use pulldown_cmark::{Event, HeadingLevel, Parser, Tag};
use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

pub struct PreviewPane {
    content: String,
    scroll: u16,
}

impl PreviewPane {
    pub fn new() -> Self {
        Self {
            content: String::new(),
            scroll: 0,
        }
    }

    pub fn set_content(&mut self, content: &str) {
        self.content = content.to_string();
        self.scroll = 0;
    }

    /// Wrap text to fit within specified width
    fn wrap_line(text: &str, width: usize) -> Vec<String> {
        if width == 0 {
            return vec![text.to_string()];
        }

        let mut lines = Vec::new();
        let mut current_line = String::new();

        for word in text.split_whitespace() {
            if current_line.is_empty() {
                current_line = word.to_string();
            } else if current_line.len() + word.len() + 1 <= width {
                current_line.push(' ');
                current_line.push_str(word);
            } else {
                lines.push(current_line);
                current_line = word.to_string();
            }
        }

        if !current_line.is_empty() {
            lines.push(current_line);
        }

        if lines.is_empty() {
            lines.push(String::new());
        }

        lines
    }

    /// Convert markdown to formatted ratatui Text with proper wrapping
    fn render_markdown(&self, theme: &dyn crate::config::Theme, width: usize) -> Text<'static> {
        if self.content.is_empty() {
            return Text::from("Select a note to preview");
        }

        let parser = Parser::new(&self.content);
        let mut lines = vec![];
        let mut current_line_spans = vec![];
        let mut in_code_block = false;
        let mut code_block_lang = String::new();
        let mut in_list = false;
        let mut list_number = 0;
        let mut in_heading = false;
        let mut heading_level = HeadingLevel::H1;
        let mut in_emphasis = false;
        let mut in_strong = false;
        let mut in_link = false;
        let mut link_url = String::new();
        let mut in_paragraph = false;

        for event in parser {
            match event {
                Event::Start(tag) => match tag {
                    Tag::Heading(level, _, _) => {
                        if !current_line_spans.is_empty() {
                            lines.push(Line::from(current_line_spans.clone()));
                            current_line_spans.clear();
                        }
                        in_heading = true;
                        heading_level = level;
                    }
                    Tag::Paragraph => {
                        in_paragraph = true;
                    }
                    Tag::CodeBlock(kind) => {
                        if !current_line_spans.is_empty() {
                            lines.push(Line::from(current_line_spans.clone()));
                            current_line_spans.clear();
                        }
                        in_code_block = true;
                        code_block_lang = match kind {
                            pulldown_cmark::CodeBlockKind::Fenced(lang) => lang.to_string(),
                            pulldown_cmark::CodeBlockKind::Indented => String::new(),
                        };
                        if !code_block_lang.is_empty() {
                            lines.push(Line::from(Span::styled(
                                format!("```{}", code_block_lang),
                                Style::default().fg(theme.fg_secondary()),
                            )));
                        } else {
                            lines.push(Line::from(Span::styled(
                                "```",
                                Style::default().fg(theme.fg_secondary()),
                            )));
                        }
                    }
                    Tag::List(Some(start_number)) => {
                        in_list = true;
                        list_number = start_number;
                    }
                    Tag::List(None) => {
                        in_list = true;
                        list_number = 0;
                    }
                    Tag::Item => {
                        if !current_line_spans.is_empty() {
                            lines.push(Line::from(current_line_spans.clone()));
                            current_line_spans.clear();
                        }
                    }
                    Tag::Emphasis => {
                        in_emphasis = true;
                    }
                    Tag::Strong => {
                        in_strong = true;
                    }
                    Tag::Link(_link_type, dest, _title) => {
                        in_link = true;
                        link_url = dest.to_string();
                        current_line_spans
                            .push(Span::styled("[", Style::default().fg(theme.info())));
                    }
                    _ => {}
                },
                Event::End(tag) => match tag {
                    Tag::Heading(_, _, _) => {
                        if !current_line_spans.is_empty() {
                            lines.push(Line::from(current_line_spans.clone()));
                            current_line_spans.clear();
                        }
                        lines.push(Line::from(""));
                        in_heading = false;
                    }
                    Tag::Paragraph => {
                        if !current_line_spans.is_empty() {
                            lines.push(Line::from(current_line_spans.clone()));
                            current_line_spans.clear();
                        }
                        lines.push(Line::from(""));
                        in_paragraph = false;
                    }
                    Tag::CodeBlock(_) => {
                        in_code_block = false;
                        lines.push(Line::from(Span::styled(
                            "```",
                            Style::default().fg(theme.fg_secondary()),
                        )));
                        lines.push(Line::from(""));
                    }
                    Tag::List(_) => {
                        in_list = false;
                    }
                    Tag::Item => {
                        if !current_line_spans.is_empty() {
                            lines.push(Line::from(current_line_spans.clone()));
                            current_line_spans.clear();
                        }
                        list_number += 1;
                    }
                    Tag::Emphasis => {
                        in_emphasis = false;
                    }
                    Tag::Strong => {
                        in_strong = false;
                    }
                    Tag::Link(_link_type, _dest, _title) => {
                        in_link = false;
                        current_line_spans
                            .push(Span::styled("]", Style::default().fg(theme.info())));
                        if !link_url.is_empty() {
                            current_line_spans.push(Span::styled(
                                format!("({})", link_url),
                                Style::default().fg(theme.fg_secondary()),
                            ));
                        }
                    }
                    _ => {}
                },
                Event::Text(text) => {
                    let style = if in_code_block {
                        Style::default().fg(theme.accent()).bg(theme.bg_secondary())
                    } else if in_heading {
                        let base_style = Style::default()
                            .fg(theme.link())
                            .add_modifier(Modifier::BOLD);
                        match heading_level {
                            HeadingLevel::H1 => base_style.bg(theme.bg_highlight()),
                            HeadingLevel::H2 => base_style,
                            HeadingLevel::H3 => base_style,
                            _ => Style::default()
                                .fg(theme.link())
                                .add_modifier(Modifier::BOLD),
                        }
                    } else if in_strong && in_emphasis {
                        Style::default()
                            .fg(theme.success())
                            .add_modifier(Modifier::BOLD)
                            .add_modifier(Modifier::ITALIC)
                    } else if in_strong {
                        Style::default()
                            .fg(theme.success())
                            .add_modifier(Modifier::BOLD)
                    } else if in_emphasis {
                        Style::default()
                            .fg(theme.warning())
                            .add_modifier(Modifier::ITALIC)
                    } else if in_link {
                        Style::default()
                            .fg(theme.link())
                            .add_modifier(Modifier::UNDERLINED)
                    } else {
                        Style::default().fg(theme.fg())
                    };

                    if in_list {
                        let marker = if list_number > 0 {
                            format!("{}. ", list_number)
                        } else {
                            "• ".to_string()
                        };
                        current_line_spans.push(Span::styled(
                            marker,
                            Style::default().fg(theme.fg_secondary()),
                        ));
                    }

                    current_line_spans.push(Span::styled(text.to_string(), style));
                }
                Event::Code(text) => {
                    current_line_spans.push(Span::styled(
                        format!("`{}`", text),
                        Style::default().fg(theme.accent()).bg(theme.bg_secondary()),
                    ));
                }
                Event::SoftBreak => {
                    if in_paragraph {
                        current_line_spans.push(Span::raw(" "));
                    } else {
                        lines.push(Line::from(current_line_spans.clone()));
                        current_line_spans.clear();
                    }
                }
                Event::HardBreak => {
                    lines.push(Line::from(current_line_spans.clone()));
                    current_line_spans.clear();
                }
                Event::Rule => {
                    if !current_line_spans.is_empty() {
                        lines.push(Line::from(current_line_spans.clone()));
                        current_line_spans.clear();
                    }
                    lines.push(Line::from(Span::styled(
                        "─".repeat(width.saturating_sub(2).max(10)),
                        Style::default().fg(theme.border()),
                    )));
                    lines.push(Line::from(""));
                }
                _ => {}
            }
        }

        if !current_line_spans.is_empty() {
            lines.push(Line::from(current_line_spans));
        }

        let mut wrapped_lines = Vec::new();
        let wrap_width = width.saturating_sub(4).max(20);

        for line in lines {
            let line_text: String = line.spans.iter().map(|s| s.content.as_ref()).collect();

            if line_text.len() > wrap_width {
                let wrapped = Self::wrap_line(&line_text, wrap_width);
                for wrapped_text in wrapped {
                    wrapped_lines.push(Line::from(Span::styled(
                        wrapped_text,
                        Style::default().fg(theme.fg()),
                    )));
                }
            } else {
                wrapped_lines.push(line);
            }
        }

        Text::from(wrapped_lines)
    }

    pub fn scroll_up(&mut self) {
        if self.scroll > 0 {
            self.scroll -= 1;
        }
    }

    pub fn scroll_down(&mut self) {
        self.scroll += 1;
    }

    pub fn scroll_to_top(&mut self) {
        self.scroll = 0;
    }

    pub fn scroll_to_bottom(&mut self, total_lines: u16) {
        self.scroll = total_lines.saturating_sub(1);
    }

    pub fn draw(
        &self,
        f: &mut Frame,
        area: Rect,
        theme: &dyn crate::config::Theme,
        is_focused: bool,
    ) {
        let width = area.width as usize;
        let text = self.render_markdown(theme, width);

        let border_color = if is_focused {
            theme.border_highlight()
        } else {
            theme.border()
        };

        let title = if is_focused {
            " Preview (↑↓ scroll) "
        } else {
            " Preview "
        };

        let paragraph = Paragraph::new(text)
            .block(
                Block::default()
                    .title(title)
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(border_color)),
            )
            .style(Style::default().fg(theme.fg()).bg(theme.bg()))
            .wrap(Wrap { trim: false })
            .scroll((self.scroll, 0));

        f.render_widget(paragraph, area);
    }
}

impl Default for PreviewPane {
    fn default() -> Self {
        Self::new()
    }
}
