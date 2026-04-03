use crossterm::{
    event::{
        self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent, KeyModifiers,
        MouseEvent,
    },
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    Terminal,
};
use std::sync::Arc;

use super::widgets::{
    NoteEditor, NoteList, PreviewPane, StatusBar,
    ConfirmationModal, ConfirmationAction,
    NoteTypeSelector, NoteTypeAction,
    CreateNoteModal, CreateNoteAction,
};
use crate::config::Config;
use crate::db::Database;
use crate::error::Result;
use crate::note::Note;
use crate::storage::{MarkdownStorage, Storage};

pub struct App {
    config: Config,
    db: Arc<Database>,

    // UI state
    mode: Mode,
    selected_note: Option<String>,
    notes: Vec<Note>,
    command_buffer: String, // For command mode input

    // Widgets
    note_list: NoteList,
    note_editor: NoteEditor,
    preview_pane: PreviewPane,
    status_bar: StatusBar,

    // Modal state
    current_modal: Option<CurrentModal>,

    // Layout
    show_preview: bool,
    running: bool,
}

#[derive(Debug)]
enum CurrentModal {
    Confirmation(ConfirmationModal),
    NoteTypeSelector(NoteTypeSelector),
    CreateNote(CreateNoteModal),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Mode {
    Normal,
    Insert,
    Search,
    Command,
    Graph,
}

impl App {
    pub fn new(config: Config, db: Database) -> Result<Self> {
        let db = Arc::new(db);

        // Load initial notes
        let notes = db.list_notes(50, 0)?;

        Ok(Self {
            config,
            db,
            mode: Mode::Normal,
            selected_note: None,
            notes,
            command_buffer: String::new(),
            note_list: NoteList::new(),
            note_editor: NoteEditor::new(),
            preview_pane: PreviewPane::new(),
            status_bar: StatusBar::new(),
            current_modal: None,
            show_preview: true,
            running: true,
        })
    }

    pub async fn run(&mut self) -> Result<()> {
        // Setup terminal
        enable_raw_mode()?;
        let mut stdout = std::io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;

        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        // Main loop
        let result = self.main_loop(&mut terminal).await;

        // Always restore terminal, even if there was an error
        let _ = disable_raw_mode();
        let _ = execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        );

        result
    }

    async fn main_loop(
        &mut self,
        terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    ) -> Result<()> {
        while self.running {
            self.status_bar.tick();

            // Draw UI
            terminal.draw(|f| self.draw(f))?;

            // Handle events with timeout to prevent blocking
            if crossterm::event::poll(std::time::Duration::from_millis(100))? {
                match event::read()? {
                    Event::Key(key) => self.handle_key(key)?,
                    Event::Mouse(mouse) => self.handle_mouse(mouse)?,
                    Event::Resize(_, _) => {} // Terminal resize, redraw on next iteration
                    _ => {}
                }
            }
        }
        Ok(())
    }

    fn draw(&self, f: &mut ratatui::Frame) {
        let theme = self.config.get_theme();

        // Layout
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .margin(1)
            .constraints(if self.show_preview {
                [
                    Constraint::Percentage(self.config.ui.sidebar_width as u16),
                    Constraint::Percentage(50),
                    Constraint::Percentage(50 - self.config.ui.sidebar_width as u16),
                ]
            } else {
                [
                    Constraint::Percentage(self.config.ui.sidebar_width as u16),
                    Constraint::Percentage(100 - self.config.ui.sidebar_width as u16),
                    Constraint::Percentage(0),
                ]
            })
            .split(f.size());

        // Draw widgets
        let theme_ref: &dyn crate::config::Theme = theme.as_ref();
        self.note_list.draw(
            f,
            chunks[0],
            &self.notes,
            theme_ref,
            self.selected_note.as_deref(),
        );
        self.note_editor.draw(f, chunks[1], theme_ref, self.mode);

        if self.show_preview {
            self.preview_pane.draw(f, chunks[2], theme_ref);
        }

        // Status bar
        let status_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(2), Constraint::Length(1)])
            .split(f.size());

        self.status_bar
            .draw(f, status_chunks[1], theme_ref, self.mode);

        // Draw modal on top if present
        if let Some(modal) = &self.current_modal {
            match modal {
                CurrentModal::Confirmation(m) => m.draw(f, theme_ref),
                CurrentModal::NoteTypeSelector(m) => m.draw(f, theme_ref),
                CurrentModal::CreateNote(m) => m.draw(f, theme_ref),
            }
        }
    }

    fn handle_key(&mut self, key: KeyEvent) -> Result<()> {
        // Modal takes priority
        if self.current_modal.is_some() {
            self.handle_modal_key(key)?;
        } else {
            match self.mode {
                Mode::Normal => self.handle_normal_mode(key),
                Mode::Insert => self.handle_insert_mode(key),
                Mode::Search => self.handle_search_mode(key),
                Mode::Command => self.handle_command_mode(key),
                Mode::Graph => self.handle_graph_mode(key),
            }
        }

        Ok(())
    }

    fn handle_modal_key(&mut self, key: KeyEvent) -> Result<()> {
        if let Some(modal) = &mut self.current_modal {
            match modal {
                CurrentModal::Confirmation(modal) => {
                    if let Some(action) = modal.handle_key(key) {
                        match action {
                            ConfirmationAction::Confirm => {
                                self.confirm_delete_note();
                            }
                            ConfirmationAction::Cancel => {}
                        }
                        self.current_modal = None;
                    }
                }
                CurrentModal::NoteTypeSelector(modal) => {
                    if let Some(action) = modal.handle_key(key) {
                        match action {
                            NoteTypeAction::Selected(note_type) => {
                                self.current_modal = Some(CurrentModal::CreateNote(
                                    CreateNoteModal::new(),
                                ));
                                self.status_bar.set_message(&format!(
                                    "Create new {} note (Esc to cancel)",
                                    note_type.label()
                                ));
                            }
                            NoteTypeAction::Cancelled => {
                                self.current_modal = None;
                            }
                        }
                    }
                }
                CurrentModal::CreateNote(modal) => {
                    if let Some(action) = modal.handle_key(key) {
                        match action {
                            CreateNoteAction::Created { title, tags } => {
                                self.create_note_with_details(title, tags);
                                self.current_modal = None;
                                self.mode = Mode::Insert;
                            }
                            CreateNoteAction::Cancelled => {
                                self.current_modal = None;
                            }
                            CreateNoteAction::Error(err) => {
                                self.status_bar.set_message(&err);
                            }
                        }
                    }
                }
            }
        }
        Ok(())
    }

    fn handle_normal_mode(&mut self, key: KeyEvent) {
        match key.code {
            // Navigation
            KeyCode::Char('j') => self.next_note(),
            KeyCode::Char('k') => self.prev_note(),
            KeyCode::Char('h') => self.prev_panel(),
            KeyCode::Char('l') => self.next_panel(),
            KeyCode::Char('g') => self.goto_top(),
            KeyCode::Char('G') => self.goto_bottom(),

            // Actions
            KeyCode::Char('i') => self.mode = Mode::Insert,
            KeyCode::Char('n') => self.new_note(),
            KeyCode::Char('d') => self.delete_note(),
            KeyCode::Char('r') if key.modifiers == KeyModifiers::CONTROL => self.rename_note(),
            KeyCode::Char('s') if key.modifiers == KeyModifiers::CONTROL => {
                self.save_current_note()
            }
            KeyCode::Char('/') => self.mode = Mode::Search,
            KeyCode::Char(':') => self.mode = Mode::Command,
            KeyCode::Char('v') => self.mode = Mode::Graph,

            // Links
            KeyCode::Enter => self.follow_link(),
            KeyCode::Char('o') => self.open_link_under_cursor(),
            KeyCode::Char('b') => self.go_back(),

            // Views
            KeyCode::Char('p') => self.toggle_preview(),

            // Quit
            KeyCode::Char('q') if key.modifiers == KeyModifiers::NONE => {
                if self.config.editor.auto_save_interval > 0 {
                    self.save_current_note();
                }
                self.running = false;
            }

            _ => {}
        }
    }

    fn handle_insert_mode(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc => {
                self.mode = Mode::Normal;
                if self.config.editor.auto_save_interval > 0 {
                    self.save_current_note();
                }
            }
            KeyCode::Char('s') if key.modifiers == KeyModifiers::CONTROL => {
                self.save_current_note();
            }
            _ => {
                // Pass to editor
                self.note_editor.handle_key(key);
            }
        }
    }

    fn handle_search_mode(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc => self.mode = Mode::Normal,
            KeyCode::Enter => {
                self.perform_search();
                self.mode = Mode::Normal;
            }
            _ => {
                // Pass to search
                //self.search_bar.handle_key(key);
            }
        }
    }

    fn handle_command_mode(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc => {
                self.mode = Mode::Normal;
                self.command_buffer.clear();
            }
            KeyCode::Enter => {
                self.execute_command();
                self.mode = Mode::Normal;
                self.command_buffer.clear();
            }
            KeyCode::Backspace => {
                self.command_buffer.pop();
                self.status_bar
                    .set_message(&format!(":{}", self.command_buffer));
            }
            KeyCode::Char(c) => {
                self.command_buffer.push(c);
                self.status_bar
                    .set_message(&format!(":{}", self.command_buffer));
            }
            _ => {}
        }
    }

    fn execute_command(&mut self) {
        let cmd = self.command_buffer.trim();

        if cmd.starts_with("rename ") {
            // Extract new title from "rename <new_title>"
            let new_title = cmd.strip_prefix("rename ").unwrap_or("").trim();
            if !new_title.is_empty() && !new_title.is_empty() {
                if let Some(selected) = &self.selected_note {
                    if let Some(note) = self.notes.iter_mut().find(|n| n.id.as_str() == selected) {
                        note.title = new_title.to_string();

                        // Save to database
                        if self.db.update_note(&note).is_ok() {
                            tracing::info!("Note renamed: {} -> {}", selected, new_title);
                            self.status_bar
                                .set_message(&format!("Renamed to: {}", new_title));
                        } else {
                            self.status_bar.set_message("Error renaming note");
                        }
                    }
                } else {
                    self.status_bar.set_message("No note selected");
                }
            } else {
                self.status_bar.set_message("Usage: rename <new_title>");
            }
        } else {
            self.status_bar
                .set_message(&format!("Unknown command: {}", cmd));
        }
    }

    fn handle_graph_mode(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => self.mode = Mode::Normal,
            _ => {}
        }
    }

    fn handle_mouse(&mut self, mouse: MouseEvent) -> Result<()> {
        match mouse.kind {
            crossterm::event::MouseEventKind::Down(button) => {
                match button {
                    crossterm::event::MouseButton::Left => {
                        if let Ok((cols, _)) = crossterm::terminal::size() {
                            let sidebar_width = cols * self.config.ui.sidebar_width / 100;
                            let in_note_list_x = mouse.column > 0 && mouse.column <= sidebar_width;
                            let first_item_row = 2_u16; // margin (1) + list top border (1)

                            if in_note_list_x && mouse.row >= first_item_row {
                                let index = (mouse.row - first_item_row) as usize;
                                if let Some(note) = self.notes.get(index) {
                                    self.selected_note = Some(note.id.as_str().to_string());
                                    self.load_note();
                                }
                            }
                        }
                    }
                    crossterm::event::MouseButton::Right => {
                        // Right click context menu (future feature)
                    }
                    _ => {}
                }
            }
            crossterm::event::MouseEventKind::ScrollUp => {
                self.prev_note();
            }
            crossterm::event::MouseEventKind::ScrollDown => {
                self.next_note();
            }
            _ => {}
        }
        Ok(())
    }

    fn next_note(&mut self) {
        if let Some(selected) = &self.selected_note {
            if let Some(pos) = self.notes.iter().position(|n| n.id.as_str() == selected) {
                if pos < self.notes.len() - 1 {
                    self.selected_note = Some(self.notes[pos + 1].id.as_str().to_string());
                    self.load_note();
                }
            }
        } else if !self.notes.is_empty() {
            self.selected_note = Some(self.notes[0].id.as_str().to_string());
            self.load_note();
        }
    }

    fn prev_note(&mut self) {
        if let Some(selected) = &self.selected_note {
            if let Some(pos) = self.notes.iter().position(|n| n.id.as_str() == selected) {
                if pos > 0 {
                    self.selected_note = Some(self.notes[pos - 1].id.as_str().to_string());
                    self.load_note();
                }
            }
        }
    }

    fn prev_panel(&mut self) {
        // Not implemented yet
    }

    fn next_panel(&mut self) {
        // Not implemented yet
    }

    fn goto_top(&mut self) {
        if !self.notes.is_empty() {
            self.selected_note = Some(self.notes[0].id.as_str().to_string());
            self.load_note();
        }
    }

    fn goto_bottom(&mut self) {
        if !self.notes.is_empty() {
            self.selected_note = Some(self.notes[self.notes.len() - 1].id.as_str().to_string());
            self.load_note();
        }
    }

    fn new_note(&mut self) {
        let selector = NoteTypeSelector::new();
        self.current_modal = Some(CurrentModal::NoteTypeSelector(selector));
        self.status_bar.set_message("Select note type (D/F/P/L for quick select)");
    }

    fn create_note_with_details(&mut self, title: String, _tags: String) {
        let note = Note::new(title, String::new());

        if let Ok(id) = self.db.create_note(&note) {
            self.notes.insert(0, note);
            self.selected_note = Some(id.as_str().to_string());
            self.load_note();
            self.status_bar
                .set_message(&format!("Note created (Insert mode)"));
        } else {
            self.status_bar.set_message("Error creating note");
        }
    }

    fn delete_note(&mut self) {
        if let Some(selected) = &self.selected_note {
            if let Some(note) = self.notes.iter().find(|n| n.id.as_str() == selected) {
                let modal = ConfirmationModal::new("Delete", &note.title);
                self.current_modal = Some(CurrentModal::Confirmation(modal));
            }
        } else {
            self.status_bar.set_message("No note selected");
        }
    }

    fn confirm_delete_note(&mut self) {
        if let Some(selected) = &self.selected_note {
            if let Ok(id) = crate::note::NoteId::parse(selected) {
                if self.db.delete_note(&id).is_ok() {
                    self.notes.retain(|n| n.id.as_str() != selected);
                    self.selected_note = None;
                    self.status_bar.set_message("Note deleted ✓");
                } else {
                    self.status_bar.set_message("Error deleting note");
                }
            }
        }
    }

    fn follow_link(&mut self) {
        // Not implemented yet
    }

    fn open_link_under_cursor(&mut self) {
        // Not implemented yet
    }

    fn go_back(&mut self) {
        // Not implemented yet
    }

    fn toggle_preview(&mut self) {
        self.show_preview = !self.show_preview;
    }

    fn load_note(&mut self) {
        if let Some(selected) = &self.selected_note {
            if let Ok(id) = crate::note::NoteId::parse(selected) {
                if let Ok(Some(note)) = self.db.get_note(&id) {
                    self.note_editor.set_content(&note.content);
                    self.preview_pane.set_content(&note.content);
                }
            }
        }
    }

    fn save_current_note(&mut self) {
        if let Some(selected) = &self.selected_note {
            if let Ok(_id) = crate::note::NoteId::parse(selected) {
                // Get the current content from the editor
                let content = self.note_editor.get_content();

                // Find the note and update it
                if let Some(note) = self.notes.iter_mut().find(|n| n.id.as_str() == selected) {
                    note.content = content;

                    // Save to database
                    if self.db.update_note(&note).is_ok() {
                        // Also save to markdown file in vault
                        let vault_path = self
                            .config
                            .vault_path()
                            .map(|p| p.to_path_buf())
                            .unwrap_or_else(|| {
                                directories::ProjectDirs::from("com", "ztlgr", "ztlgr")
                                    .map(|dirs| dirs.data_dir().join("vault"))
                                    .unwrap_or_else(|| std::path::PathBuf::from("./vault"))
                            });

                        let file_path = vault_path.join(format!("{}.md", note.id));
                        let markdown_storage = MarkdownStorage::new();

                        if markdown_storage.write_note(note, &file_path).is_ok() {
                            tracing::info!("Note saved: {} (db + file)", selected);
                            self.status_bar.set_message("Note saved ✓");
                        } else {
                            tracing::warn!(
                                "Note saved to db but failed to save to file: {}",
                                selected
                            );
                            self.status_bar
                                .set_message("Note saved to db (file save failed)");
                        }
                    } else {
                        self.status_bar.set_message("Error saving note");
                    }
                }
            }
        } else {
            self.status_bar.set_message("No note selected");
        }
    }

    fn rename_note(&mut self) {
        // Switch to command mode for renaming
        self.mode = Mode::Command;
        self.command_buffer = "rename ".to_string();
        self.status_bar.set_message(":rename <new_title>");
    }

    fn perform_search(&mut self) {
        // Not implemented yet
    }
}
