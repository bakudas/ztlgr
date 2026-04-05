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

use super::widgets::modals::note_type_selector::NoteType as SelectorNoteType;
use super::widgets::{
    Command, CommandContext, CommandExecutor, CommandParser, CommandResult, ConfirmationAction,
    ConfirmationModal, CreateNoteAction, CreateNoteModal, MetadataPane, NoteEditor, NoteList,
    NoteTypeAction, NoteTypeSelector, PreviewPane, SearchResult, SearchState, StatusBar,
};

/// Sanitize filename by removing invalid characters
fn sanitize_filename(name: &str) -> String {
    let mut filename = name.to_string();
    // Replace spaces with hyphens
    filename = filename.replace(' ', "-");
    // Remove invalid characters
    filename.retain(|c| c.is_alphanumeric() || c == '-' || c == '_');
    // Limit length
    if filename.len() > 100 {
        filename = filename[..100].to_string();
    }
    filename
}
use crate::config::Config;
use crate::db::Database;
use crate::error::Result;
use crate::note::Note;
use crate::storage::{MarkdownStorage, Storage};

#[derive(Debug, Clone, Copy, PartialEq)]
enum RightPanel {
    Preview,
    Metadata,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Panel {
    NoteList,
    Editor,
    Right, // Preview or Metadata
}

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
    metadata_pane: MetadataPane,
    status_bar: StatusBar,

    // Modal state
    current_modal: Option<CurrentModal>,
    pending_note_type: Option<SelectorNoteType>, // Store selected note type during creation

    // Search state
    search_state: SearchState,

    // Layout
    right_panel: RightPanel,
    focused_panel: Panel,
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

        let mut app = Self {
            config,
            db,
            mode: Mode::Normal,
            selected_note: None,
            notes,
            command_buffer: String::new(),
            note_list: NoteList::new(),
            note_editor: NoteEditor::new(),
            preview_pane: PreviewPane::new(),
            metadata_pane: MetadataPane::new(),
            status_bar: StatusBar::new(),
            current_modal: None,
            pending_note_type: None,
            search_state: SearchState::new(),
            right_panel: RightPanel::Preview,
            focused_panel: Panel::NoteList,
            show_preview: true,
            running: true,
        };

        // Load the first note if available
        if !app.notes.is_empty() {
            app.selected_note = Some(app.notes[0].id.as_str().to_string());
            app.load_note();
        }

        Ok(app)
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

        let chunks = if self.show_preview {
            Layout::default()
                .direction(Direction::Horizontal)
                .margin(1)
                .constraints([
                    Constraint::Min(15),
                    Constraint::Min(20),
                    Constraint::Min(15),
                ])
                .split(f.size())
        } else {
            Layout::default()
                .direction(Direction::Horizontal)
                .margin(1)
                .constraints([Constraint::Min(15), Constraint::Min(20)])
                .split(f.size())
        };

        let theme_ref: &dyn crate::config::Theme = theme.as_ref();
        self.note_list.draw(
            f,
            chunks[0],
            &self.notes,
            theme_ref,
            self.selected_note.as_deref(),
            self.focused_panel == Panel::NoteList,
        );
        self.note_editor.draw(
            f,
            chunks[1],
            theme_ref,
            self.mode,
            self.focused_panel == Panel::Editor,
            &self.db,
        );

        if self.show_preview {
            match self.right_panel {
                RightPanel::Preview => self.preview_pane.draw(
                    f,
                    chunks[2],
                    theme_ref,
                    self.focused_panel == Panel::Right,
                ),
                RightPanel::Metadata => self.metadata_pane.draw(
                    f,
                    chunks[2],
                    theme_ref,
                    self.focused_panel == Panel::Right,
                ),
            }
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
                                self.pending_note_type = Some(note_type);

                                // Daily notes are created automatically with date as title
                                if matches!(note_type, SelectorNoteType::Daily) {
                                    // Check if daily note already exists for today
                                    let today = chrono::Utc::now().format("%Y-%m-%d").to_string();
                                    let daily_exists = self.notes.iter().any(|n| {
                                        matches!(n.note_type, crate::note::NoteType::Daily)
                                            && n.created_at.format("%Y-%m-%d").to_string() == today
                                    });

                                    if daily_exists {
                                        self.status_bar
                                            .set_message("Daily note already exists for today!");
                                        self.current_modal = None;
                                        self.pending_note_type = None;
                                    } else {
                                        // Create daily note automatically
                                        let title = format!("Daily Note - {}", today);
                                        let note_type = crate::note::NoteType::Daily;
                                        self.create_note_with_details(
                                            title,
                                            String::new(),
                                            note_type,
                                            true,
                                        );
                                        self.pending_note_type = None;
                                        self.current_modal = None;
                                        self.mode = Mode::Insert;
                                    }
                                } else {
                                    // For other note types, show the CreateNoteModal
                                    let create_modal =
                                        CreateNoteModal::new().with_note_type(note_type.label());
                                    self.current_modal =
                                        Some(CurrentModal::CreateNote(create_modal));
                                    self.status_bar.set_message(&format!(
                                        "Creating {} note - Enter title",
                                        note_type.label()
                                    ));
                                }
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
                                // Use the stored note type or default to Fleeting
                                let note_type = self
                                    .pending_note_type
                                    .map(|t| match t {
                                        SelectorNoteType::Daily => crate::note::NoteType::Daily,
                                        SelectorNoteType::Fleeting => {
                                            crate::note::NoteType::Fleeting
                                        }
                                        SelectorNoteType::Permanent => {
                                            crate::note::NoteType::Permanent
                                        }
                                        SelectorNoteType::Literature => {
                                            crate::note::NoteType::Literature {
                                                source: String::new(),
                                            }
                                        }
                                    })
                                    .unwrap_or(crate::note::NoteType::Fleeting); // Default to Fleeting

                                // Check if daily note already exists for today
                                if matches!(note_type, crate::note::NoteType::Daily) {
                                    let today = chrono::Utc::now().format("%Y-%m-%d").to_string();
                                    let daily_exists = self.notes.iter().any(|n| {
                                        matches!(n.note_type, crate::note::NoteType::Daily)
                                            && n.created_at.format("%Y-%m-%d").to_string() == today
                                    });

                                    if daily_exists {
                                        self.status_bar
                                            .set_message("Daily note already exists for today!");
                                        self.pending_note_type = None;
                                        self.current_modal = None;
                                        return Ok(());
                                    }
                                }

                                // Create note with template option
                                let use_template = modal.use_template();
                                self.create_note_with_details(title, tags, note_type, use_template);
                                self.pending_note_type = None; // Clear after use
                                self.current_modal = None;
                                self.mode = Mode::Insert;
                            }
                            CreateNoteAction::Cancelled => {
                                self.pending_note_type = None; // Clear on cancel
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
        if self.focused_panel == Panel::Editor {
            self.handle_editor_normal_mode(key);
            return;
        }

        match key.code {
            KeyCode::Char('j') => {
                if self.focused_panel == Panel::Right && self.show_preview {
                    match self.right_panel {
                        RightPanel::Preview => self.preview_pane.scroll_down(),
                        RightPanel::Metadata => self.next_note(),
                    }
                } else {
                    self.next_note()
                }
            }
            KeyCode::Char('k') => {
                if self.focused_panel == Panel::Right && self.show_preview {
                    match self.right_panel {
                        RightPanel::Preview => self.preview_pane.scroll_up(),
                        RightPanel::Metadata => self.prev_note(),
                    }
                } else {
                    self.prev_note()
                }
            }
            KeyCode::Char('h') => self.prev_panel(),
            KeyCode::Char('l') => self.next_panel(),
            KeyCode::Char('g') => self.goto_top(),
            KeyCode::Char('G') => self.goto_bottom(),

            KeyCode::Char('i') => {
                self.mode = Mode::Insert;
                self.focused_panel = Panel::Editor;
            }
            KeyCode::Char('n') => self.new_note(),
            KeyCode::Char('d') => self.delete_note(),
            KeyCode::Char('r') if key.modifiers == KeyModifiers::CONTROL => self.rename_note(),
            KeyCode::Char('s') if key.modifiers == KeyModifiers::CONTROL => {
                self.save_current_note()
            }
            KeyCode::Char('/') => self.mode = Mode::Search,
            KeyCode::Char(':') => self.mode = Mode::Command,
            KeyCode::Char('v') => self.mode = Mode::Graph,

            KeyCode::Enter => self.follow_link(),
            KeyCode::Char('o') => self.open_link_under_cursor(),
            KeyCode::Char('b') => self.go_back(),

            KeyCode::Char('p') => self.toggle_preview(),
            KeyCode::Char('m') => self.toggle_metadata(),

            _ => {}
        }
    }

    fn handle_editor_normal_mode(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('i') => {
                self.mode = Mode::Insert;
            }
            KeyCode::Char('a') => {
                self.note_editor.move_cursor_right();
                self.mode = Mode::Insert;
            }
            KeyCode::Char('A') => {
                self.note_editor.move_cursor_end();
                self.mode = Mode::Insert;
            }
            KeyCode::Char('o') => {
                self.note_editor.move_cursor_end();
                self.note_editor.insert_newline();
                self.mode = Mode::Insert;
            }
            KeyCode::Char('O') => {
                self.note_editor.move_cursor_home();
                self.note_editor.insert_newline();
                self.note_editor.move_cursor_up();
                self.mode = Mode::Insert;
            }

            KeyCode::Char('h') => self.note_editor.move_cursor_left(),
            KeyCode::Char('j') => self.note_editor.move_cursor_down(),
            KeyCode::Char('k') => self.note_editor.move_cursor_up(),
            KeyCode::Char('l') => self.note_editor.move_cursor_right(),

            KeyCode::Char('w') => self.note_editor.move_cursor_word_forward(),
            KeyCode::Char('b') => self.note_editor.move_cursor_word_back(),

            KeyCode::Char('0') => self.note_editor.move_cursor_home(),
            KeyCode::Char('$') => self.note_editor.move_cursor_end(),

            KeyCode::Char('g') => self.note_editor.move_cursor_top(),
            KeyCode::Char('G') => self.note_editor.move_cursor_bottom(),

            KeyCode::Char('x') => self.note_editor.delete_next_char(),
            KeyCode::Char('X') => self.note_editor.delete_prev_char(),
            KeyCode::Char('d') => {
                self.note_editor.delete_line();
            }
            KeyCode::Char('D') => {
                self.note_editor.delete_to_end_of_line();
            }

            KeyCode::Char('u') => self.note_editor.undo(),
            KeyCode::Char('r') if key.modifiers == KeyModifiers::CONTROL => {
                self.note_editor.redo();
            }

            KeyCode::Char('y') => {
                self.note_editor.copy();
            }
            KeyCode::Char('p') => {
                self.note_editor.paste();
            }

            KeyCode::Char('H') => self.prev_panel(),
            KeyCode::Char('L') => self.next_panel(),

            KeyCode::Esc => {
                self.focused_panel = Panel::NoteList;
            }

            _ => {}
        }
    }

    fn handle_insert_mode(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc => {
                // Always save when exiting insert mode
                self.save_current_note();
                self.mode = Mode::Normal;
            }
            KeyCode::Char('s') if key.modifiers == KeyModifiers::CONTROL => {
                self.save_current_note();
            }
            _ => {
                // Pass to editor
                self.note_editor.handle_key(key);
                self.note_editor.update_autocomplete_db(&self.db);

                // Update preview in real-time if preview is visible
                if self.show_preview && matches!(self.right_panel, RightPanel::Preview) {
                    let content = self.note_editor.get_content();
                    self.preview_pane.set_content(&content);
                }
            }
        }
    }

    fn handle_search_mode(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc => {
                self.mode = Mode::Normal;
                self.search_state.clear();
            }
            KeyCode::Enter => {
                self.perform_search();
                // Stay in search mode to show results
            }
            KeyCode::Up => {
                self.search_state.results.select_prev();
            }
            KeyCode::Down => {
                self.search_state.results.select_next();
            }
            _ => {
                // Handle input characters
                self.search_state.input.handle_key(key);
                // Auto-search as user types
                self.perform_search();
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
        let cmd_str = self.command_buffer.trim();

        // Parse the command
        let command = CommandParser::parse(cmd_str);

        // Create context for the executor
        let context = if let Some(selected) = &self.selected_note.clone() {
            if self.notes.iter().any(|n| n.id.as_str() == selected) {
                CommandContext::with_note(selected.clone())
            } else {
                CommandContext::new()
            }
        } else {
            CommandContext::new()
        };

        // Execute the command and get result
        let result = CommandExecutor::execute(&command, &context);

        match result {
            CommandResult::Success(msg) => {
                // Handle specific command side effects
                match command {
                    Command::Help => {
                        self.status_bar.set_message(&msg);
                    }
                    Command::Rename(new_title) => {
                        if let Some(selected) = &self.selected_note.clone() {
                            if let Some(note) =
                                self.notes.iter_mut().find(|n| n.id.as_str() == selected)
                            {
                                note.title = new_title;
                                note.updated_at = chrono::Utc::now();

                                // Save to database
                                if self.db.update_note(note).is_ok() {
                                    tracing::info!("Note renamed: {} -> {}", selected, note.title);
                                    self.status_bar.set_message(&msg);
                                } else {
                                    self.status_bar.set_message("Error renaming note");
                                }
                            }
                        }
                    }
                    Command::Move(folder) => {
                        if let Some(selected) = &self.selected_note.clone() {
                            if let Some(note) =
                                self.notes.iter_mut().find(|n| n.id.as_str() == selected)
                            {
                                // Update note type based on folder
                                note.note_type = match folder.as_str() {
                                    "daily" => crate::note::NoteType::Daily,
                                    "fleeting" => crate::note::NoteType::Fleeting,
                                    "permanent" => crate::note::NoteType::Permanent,
                                    "literature" => crate::note::NoteType::Literature {
                                        source: String::new(),
                                    },
                                    "index" => crate::note::NoteType::Index,
                                    "reference" => crate::note::NoteType::Reference { url: None },
                                    _ => note.note_type.clone(),
                                };
                                note.updated_at = chrono::Utc::now();

                                // Save to database
                                if self.db.update_note(note).is_ok() {
                                    tracing::info!("Note moved to: {}", folder);
                                    self.status_bar.set_message(&msg);
                                } else {
                                    self.status_bar.set_message("Error moving note");
                                }
                            }
                        }
                    }
                    Command::Tag(tags) => {
                        if let Some(selected) = &self.selected_note.clone() {
                            if let Some(note) =
                                self.notes.iter_mut().find(|n| n.id.as_str() == selected)
                            {
                                // Add tags to note metadata
                                if note.metadata.tags.is_none() {
                                    note.metadata.tags = Some(Vec::new());
                                }
                                if let Some(ref mut note_tags) = note.metadata.tags {
                                    for tag in tags {
                                        if !note_tags.contains(&tag) {
                                            note_tags.push(tag);
                                        }
                                    }
                                }
                                note.updated_at = chrono::Utc::now();

                                // Save to database
                                if self.db.update_note(note).is_ok() {
                                    tracing::info!("Tags added to note: {}", selected);
                                    self.status_bar.set_message(&msg);
                                } else {
                                    self.status_bar.set_message("Error adding tags");
                                }
                            }
                        }
                    }
                    Command::Delete => {
                        // Show confirmation modal for delete
                        self.current_modal = Some(CurrentModal::Confirmation(
                            ConfirmationModal::new("Delete Note", "current note"),
                        ));
                        self.status_bar.set_message("Delete confirmation shown");
                    }
                    Command::Export(format) => {
                        self.status_bar
                            .set_message(&format!("Export to {} not yet implemented", format));
                    }
                    Command::Quit => {
                        // Save before quitting
                        if self.config.editor.auto_save_interval > 0 {
                            self.save_current_note();
                        }
                        self.running = false;
                    }
                    Command::Unknown(_) => {
                        // Error case handled below
                    }
                }
            }
            CommandResult::Error(msg) => {
                self.status_bar.set_message(&msg);
            }
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
        // Save current note before switching
        self.save_current_note();

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
        // Save current note before switching
        self.save_current_note();

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
        match self.focused_panel {
            Panel::NoteList => {
                // Already at leftmost panel
            }
            Panel::Editor => {
                self.focused_panel = Panel::NoteList;
            }
            Panel::Right => {
                self.focused_panel = Panel::Editor;
            }
        }
    }

    fn next_panel(&mut self) {
        if !self.show_preview {
            // Can only navigate to note list and editor
            match self.focused_panel {
                Panel::NoteList => {
                    self.focused_panel = Panel::Editor;
                }
                Panel::Editor => {
                    self.focused_panel = Panel::NoteList;
                }
                Panel::Right => {
                    // Should not happen when preview is hidden
                }
            }
        } else {
            // All three panels available
            match self.focused_panel {
                Panel::NoteList => {
                    self.focused_panel = Panel::Editor;
                }
                Panel::Editor => {
                    self.focused_panel = Panel::Right;
                }
                Panel::Right => {
                    self.focused_panel = Panel::NoteList;
                }
            }
        }
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
        self.status_bar
            .set_message("Select note type (D/F/P/L for quick select)");
    }

    fn create_note_with_details(
        &mut self,
        title: String,
        _tags: String,
        note_type: crate::note::NoteType,
        use_template: bool,
    ) {
        // Save current note before creating new one
        self.save_current_note();

        // Create note with specified type and template option
        let note = Note::with_template(title, note_type.clone(), use_template);

        if let Ok(id) = self.db.create_note(&note) {
            // Refresh notes list from database to ensure sync
            if let Ok(updated_notes) = self.db.list_notes(50, 0) {
                self.notes = updated_notes;
            } else {
                // Fallback: just insert the new note at the beginning
                self.notes.insert(0, note);
            }

            self.selected_note = Some(id.as_str().to_string());
            self.load_note();
            self.mode = Mode::Insert; // Switch to insert mode immediately
            self.status_bar
                .set_message(&format!("{} note created successfully!", note_type));
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
        // Reset focus to editor when toggling preview
        if self.focused_panel == Panel::Right {
            self.focused_panel = Panel::Editor;
        }
    }

    fn toggle_metadata(&mut self) {
        if self.show_preview {
            // Toggle between preview and metadata
            self.right_panel = match self.right_panel {
                RightPanel::Preview => RightPanel::Metadata,
                RightPanel::Metadata => {
                    // When switching back to preview, update with current content
                    let content = self.note_editor.get_content();
                    self.preview_pane.set_content(&content);
                    RightPanel::Preview
                }
            };
        }
    }

    fn load_note(&mut self) {
        if let Some(selected) = &self.selected_note {
            if let Ok(id) = crate::note::NoteId::parse(selected) {
                if let Ok(Some(note)) = self.db.get_note(&id) {
                    self.note_editor.set_content(&note.content);
                    self.preview_pane.set_content(&note.content);
                    self.metadata_pane.set_note(note);
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
                    if self.db.update_note(note).is_ok() {
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

                        // Determine subdirectory based on note type
                        let subdir = match &note.note_type {
                            crate::note::NoteType::Daily => "daily",
                            crate::note::NoteType::Fleeting => "inbox",
                            crate::note::NoteType::Literature { .. } => "literature",
                            crate::note::NoteType::Permanent => "permanent",
                            crate::note::NoteType::Reference { .. } => "reference",
                            crate::note::NoteType::Index => "index",
                        };

                        // Generate filename from zettel_id + title (or just title)
                        let filename = note
                            .zettel_id
                            .as_ref()
                            .map(|z| format!("{}-{}", z.as_str(), sanitize_filename(&note.title)))
                            .unwrap_or_else(|| sanitize_filename(&note.title));

                        let file_path = vault_path.join(subdir).join(format!("{}.md", filename));
                        let markdown_storage = MarkdownStorage::new();

                        if markdown_storage.write_note(note, &file_path).is_ok() {
                            tracing::info!(
                                "Note saved: {} (db + file at {})",
                                selected,
                                file_path.display()
                            );
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
        let query = self.search_state.input.query().to_string(); // Clone to avoid borrow issues

        if query.trim().is_empty() {
            self.search_state.results.clear();
            self.status_bar.set_message("Search: (type to search)");
            return;
        }

        self.search_state.set_searching(true);

        // Use FTS5 to search notes
        match self.db.search_notes(&query, 20) {
            Ok(search_results) => {
                self.search_state.results.clear();

                for result in search_results {
                    let excerpt = if result.content.len() > 100 {
                        format!("{}...", &result.content[..100])
                    } else {
                        result.content.clone()
                    };

                    let search_result = SearchResult {
                        note_id: result.id.to_string(),
                        title: result.title.clone(),
                        excerpt,
                        score: 0.85, // Default score for now
                    };
                    self.search_state.results.add_result(search_result);
                }

                let count = self.search_state.results.count();
                self.status_bar.set_message(&format!(
                    "Search: '{}' - {} results (↑↓ navigate, Enter to open, Esc to cancel)",
                    query, count
                ));
            }
            Err(_e) => {
                self.search_state.results.clear();
                self.status_bar
                    .set_message(&format!("Search error for: '{}'", query));
            }
        }

        self.search_state.set_searching(false);
    }
}
