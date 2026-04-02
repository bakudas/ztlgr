use std::sync::Arc;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    Terminal,
};

use crate::config::Config;
use crate::db::Database;
use crate::note::Note;
use crate::error::Result;
use super::widgets::{NoteList, NoteEditor, PreviewPane, StatusBar};

pub struct App {
    config: Config,
    db: Arc<Database>,
    
    // UI state
    mode: Mode,
    selected_note: Option<String>,
    notes: Vec<Note>,
    
    // Widgets
    note_list: NoteList,
    note_editor: NoteEditor,
    preview_pane: PreviewPane,
    status_bar: StatusBar,
    
    // Layout
    show_preview: bool,
    running: bool,
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
        let theme = config.get_theme();
        
        // Load initial notes
        let notes = db.list_notes(50, 0)?;
        
        Ok(Self {
            config,
            db,
            mode: Mode::Normal,
            selected_note: None,
            notes,
            note_list: NoteList::new(),
            note_editor: NoteEditor::new(),
            preview_pane: PreviewPane::new(),
            status_bar: StatusBar::new(),
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
        while self.running {
            // Draw UI
            terminal.draw(|f| self.draw(f))?;
            
            // Handle events
            if let Event::Key(key) = event::read()? {
                self.handle_key(key)?;
            }
        }
        
        // Restore terminal
        disable_raw_mode()?;
        execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )?;
        
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
        self.note_list.draw(f, chunks[0], &self.notes, theme_ref, self.selected_note.as_deref());
        self.note_editor.draw(f, chunks[1], theme_ref, self.mode);
        
        if self.show_preview {
            self.preview_pane.draw(f, chunks[2], theme_ref);
        }
        
        // Status bar
        let status_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(2), Constraint::Length(1)])
            .split(f.size());
        
        self.status_bar.draw(f, status_chunks[1], theme_ref, self.mode);
    }
    
    fn handle_key(&mut self, key: KeyEvent) -> Result<()> {
        match self.mode {
            Mode::Normal => self.handle_normal_mode(key),
            Mode::Insert => self.handle_insert_mode(key),
            Mode::Search => self.handle_search_mode(key),
            Mode::Command => self.handle_command_mode(key),
            Mode::Graph => self.handle_graph_mode(key),
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
            KeyCode::Esc => self.mode = Mode::Normal,
            KeyCode::Enter => {
                //self.execute_command();
                self.mode = Mode::Normal;
            }
            _ => {
                // Pass to command bar
            }
        }
    }
    
    fn handle_graph_mode(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => self.mode = Mode::Normal,
            _ => {}
        }
    }
    
    // Actions
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
        // Create new note
        let note = Note::new("New Note".to_string(), String::new());
        if let Ok(id) = self.db.create_note(&note) {
            self.notes.insert(0, note);
            self.selected_note = Some(id.as_str().to_string());
            self.load_note();
            self.mode = Mode::Insert;
        }
    }
    
    fn delete_note(&mut self) {
        if let Some(selected) = &self.selected_note {
            if let Ok(id) = crate::note::NoteId::parse(selected) {
                if self.db.delete_note(&id).is_ok() {
                    self.notes.retain(|n| n.id.as_str() != selected);
                    self.selected_note = None;
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
                }
            }
        }
    }
    
    fn save_current_note(&mut self) {
        // Not implemented yet
    }
    
    fn perform_search(&mut self) {
        // Not implemented yet
    }
}