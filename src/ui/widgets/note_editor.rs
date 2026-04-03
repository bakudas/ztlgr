use crate::ui::app::Mode;
use crossterm::event::KeyEvent;
use ratatui::{
    layout::Rect,
    style::Style,
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

pub struct NoteEditor {
    content: String,
    cursor: usize,
    line: usize,       // Current line number for cursor
    col: usize,        // Current column for cursor
    line_width: usize, // Max 80 characters per line
}

impl NoteEditor {
    pub fn new() -> Self {
        Self {
            content: String::new(),
            cursor: 0,
            line: 0,
            col: 0,
            line_width: 80,
        }
    }

    pub fn set_content(&mut self, content: &str) {
        self.content = content.to_string();
        self.cursor = content.len();
        self.update_cursor_position();
    }

    pub fn get_content(&self) -> String {
        self.content.clone()
    }

    pub fn clear(&mut self) {
        self.content.clear();
        self.cursor = 0;
        self.line = 0;
        self.col = 0;
    }

    /// Atualiza a posição do cursor (linha e coluna) baseado no offset
    fn update_cursor_position(&mut self) {
        let mut pos = 0;
        self.line = 0;
        self.col = 0;

        for ch in self.content.chars() {
            if pos >= self.cursor {
                break;
            }
            if ch == '\n' {
                self.line += 1;
                self.col = 0;
            } else {
                self.col += 1;
            }
            pos += ch.len_utf8();
        }
    }

    /// Converte texto para linhas renderizáveis com indicador de cursor
    fn render_content(&self) -> Vec<Line> {
        let mut lines = Vec::new();
        let mut current_line = String::new();
        let mut current_pos = 0;

        for ch in self.content.chars() {
            if ch == '\n' {
                // Renderizar linha com cursor se necessário
                lines.push(self.render_line(&current_line, current_pos));
                current_line.clear();
                current_pos += 1;
            } else {
                current_line.push(ch);
                current_pos += ch.len_utf8();
            }
        }

        // Renderizar última linha
        if !current_line.is_empty() || self.content.ends_with('\n') {
            lines.push(self.render_line(&current_line, current_pos));
        }

        if lines.is_empty() {
            lines.push(Line::from(""));
        }

        lines
    }

    /// Renderiza uma linha individual com cursor
    fn render_line(&self, line_text: &str, _line_start: usize) -> Line {
        if line_text.len() < self.line_width {
            Line::from(line_text.to_string())
        } else {
            // Mostrar coluna de 80 caracteres como no Vim
            Line::from(line_text.to_string())
        }
    }

    pub fn draw(&self, f: &mut Frame, area: Rect, theme: &dyn crate::config::Theme, mode: Mode) {
        let mode_text = match mode {
            Mode::Normal => "-- NORMAL --",
            Mode::Insert => "-- INSERT --",
            Mode::Search => "-- SEARCH --",
            Mode::Command => "-- COMMAND --",
            Mode::Graph => "-- GRAPH --",
        };

        // Renderizar conteúdo com linha do cursor visível
        let lines = if self.content.is_empty() {
            vec![Line::from(Span::styled(
                "Press 'i' to enter insert mode or 'n' to create a new note",
                Style::default().fg(theme.fg_dim()),
            ))]
        } else {
            self.render_content()
        };

        // Mostrar posição do cursor no título
        let title = format!(
            " Editor {} - Line {} Col {} ",
            mode_text,
            self.line + 1,
            self.col + 1
        );

        let paragraph = Paragraph::new(lines)
            .block(
                Block::default()
                    .title(title)
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(theme.border_highlight())),
            )
            .style(Style::default().fg(theme.fg()).bg(theme.bg()))
            .wrap(Wrap { trim: false });

        f.render_widget(paragraph, area);

        // Se em modo Insert, mostrar cursor
        if mode == Mode::Insert && area.height > 2 {
            // Calcular posição do cursor na tela
            let cursor_x = area.x + 1 + (self.col.min(area.width as usize - 3)) as u16;
            let cursor_y = area.y + 1 + self.line.min(area.height as usize - 3) as u16;

            // Renderizar cursor
            f.set_cursor(cursor_x, cursor_y);
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) {
        match key.code {
            crossterm::event::KeyCode::Char(c) => {
                self.content.insert(self.cursor, c);
                self.cursor += c.len_utf8();
                self.update_cursor_position();
            }
            crossterm::event::KeyCode::Enter => {
                self.content.insert(self.cursor, '\n');
                self.cursor += 1;
                self.update_cursor_position();
            }
            crossterm::event::KeyCode::Backspace => {
                if self.cursor > 0 {
                    // Encontrar o caractere anterior para remover corretamente
                    let mut remove_at = self.cursor - 1;
                    while remove_at > 0 && !self.content.is_char_boundary(remove_at) {
                        remove_at -= 1;
                    }
                    let ch = self.content.chars().rev().next().unwrap_or(' ');
                    self.content.remove(remove_at);
                    self.cursor = remove_at;
                    self.update_cursor_position();
                }
            }
            crossterm::event::KeyCode::Delete => {
                if self.cursor < self.content.len() {
                    let ch_len = self.content[self.cursor..]
                        .chars()
                        .next()
                        .map(|c| c.len_utf8())
                        .unwrap_or(1);
                    for _ in 0..ch_len {
                        self.content.remove(self.cursor);
                    }
                    self.update_cursor_position();
                }
            }
            crossterm::event::KeyCode::Left => {
                if self.cursor > 0 {
                    self.cursor -= 1;
                    // Lidar com UTF-8 multi-byte characters
                    while self.cursor > 0 && !self.content.is_char_boundary(self.cursor) {
                        self.cursor -= 1;
                    }
                    self.update_cursor_position();
                }
            }
            crossterm::event::KeyCode::Right => {
                if self.cursor < self.content.len() {
                    self.cursor += 1;
                    // Lidar com UTF-8 multi-byte characters
                    while self.cursor < self.content.len()
                        && !self.content.is_char_boundary(self.cursor)
                    {
                        self.cursor += 1;
                    }
                    self.update_cursor_position();
                }
            }
            crossterm::event::KeyCode::Home => {
                // Ir para o início da linha
                let line_start = self.content[..self.cursor]
                    .rfind('\n')
                    .map(|i| i + 1)
                    .unwrap_or(0);
                self.cursor = line_start;
                self.update_cursor_position();
            }
            crossterm::event::KeyCode::End => {
                // Ir para o final da linha
                let line_end = self.content[self.cursor..]
                    .find('\n')
                    .map(|i| self.cursor + i)
                    .unwrap_or(self.content.len());
                self.cursor = line_end;
                self.update_cursor_position();
            }
            crossterm::event::KeyCode::Up => {
                // Mover cursor para cima (se não estiver na primeira linha)
                if self.line > 0 {
                    // Encontrar início da linha atual
                    let line_start = self.content[..self.cursor]
                        .rfind('\n')
                        .map(|i| i + 1)
                        .unwrap_or(0);

                    // Encontrar início da linha anterior
                    if line_start > 0 {
                        let prev_line_start = self.content[..line_start - 1]
                            .rfind('\n')
                            .map(|i| i + 1)
                            .unwrap_or(0);

                        let prev_line_end = line_start - 1;
                        let prev_line = &self.content[prev_line_start..prev_line_end];

                        // Posicionar cursor na mesma coluna (ou final da linha se for mais curta)
                        let target_col = (self.col).min(prev_line.len());
                        self.cursor = prev_line_start + target_col;
                        self.update_cursor_position();
                    }
                }
            }
            crossterm::event::KeyCode::Down => {
                // Mover cursor para baixo (se não estiver na última linha)
                let line_start = self.content[..self.cursor]
                    .rfind('\n')
                    .map(|i| i + 1)
                    .unwrap_or(0);

                let line_end = self.content[self.cursor..]
                    .find('\n')
                    .map(|i| self.cursor + i)
                    .unwrap_or(self.content.len());

                if line_end < self.content.len() {
                    let next_line_start = line_end + 1;
                    let next_line_end = self.content[next_line_start..]
                        .find('\n')
                        .map(|i| next_line_start + i)
                        .unwrap_or(self.content.len());

                    let next_line = &self.content[next_line_start..next_line_end];
                    let target_col = (self.col).min(next_line.len());
                    self.cursor = next_line_start + target_col;
                    self.update_cursor_position();
                }
            }
            _ => {}
        }
    }
}
