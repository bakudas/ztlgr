use pulldown_cmark::{Event, Parser};
use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

pub struct PreviewPane {
    content: String,
}

impl PreviewPane {
    pub fn new() -> Self {
        Self {
            content: String::new(),
        }
    }

    pub fn set_content(&mut self, content: &str) {
        self.content = content.to_string();
    }

    /// Convert markdown to formatted ratatui Text
    fn render_markdown(&self, theme: &dyn crate::config::Theme) -> Text<'static> {
        if self.content.is_empty() {
            return Text::from("Select a note to preview");
        }

        let parser = Parser::new(&self.content);
        let mut lines = vec![];
        let mut current_line = vec![];
        let mut in_code_block = false;
        let mut in_list = false;

        for event in parser {
            match event {
                Event::Start(tag) => {
                    match tag {
                        pulldown_cmark::Tag::Heading(..) => {
                            // Heading styling
                            if !current_line.is_empty() {
                                lines.push(Line::from(current_line.clone()));
                                current_line.clear();
                            }
                        }
                        pulldown_cmark::Tag::CodeBlock(_) => {
                            in_code_block = true;
                        }
                        pulldown_cmark::Tag::List(_) => {
                            in_list = true;
                        }
                        _ => {}
                    }
                }
                Event::End(_tag) => {
                    if in_code_block {
                        in_code_block = false;
                    }
                    if in_list {
                        in_list = false;
                    }
                }
                Event::Text(text) => {
                    let style = if in_code_block {
                        Style::default().fg(theme.accent()).bg(theme.bg_secondary())
                    } else {
                        Style::default().fg(theme.fg())
                    };

                    current_line.push(Span::styled(text.to_string(), style));
                }
                Event::Code(text) => {
                    current_line.push(Span::styled(
                        text.to_string(),
                        Style::default()
                            .fg(theme.success())
                            .add_modifier(Modifier::BOLD),
                    ));
                }
                Event::SoftBreak | Event::HardBreak => {
                    if !current_line.is_empty() {
                        lines.push(Line::from(current_line.clone()));
                        current_line.clear();
                    } else {
                        lines.push(Line::from(""));
                    }
                }
                Event::Rule => {
                    if !current_line.is_empty() {
                        lines.push(Line::from(current_line.clone()));
                        current_line.clear();
                    }
                    lines.push(Line::from("───────────────────"));
                }
                _ => {
                    // Other events like html, footnotes, etc.
                }
            }
        }

        // Add remaining line
        if !current_line.is_empty() {
            lines.push(Line::from(current_line));
        }

        Text::from(lines)
    }

    pub fn draw(
        &self,
        f: &mut Frame,
        area: Rect,
        theme: &dyn crate::config::Theme,
        is_focused: bool,
    ) {
        let text = self.render_markdown(theme);

        let border_color = if is_focused {
            theme.border_highlight()
        } else {
            theme.border()
        };

        let paragraph = Paragraph::new(text)
            .block(
                Block::default()
                    .title(" Preview ")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(border_color)),
            )
            .style(Style::default().fg(theme.fg()).bg(theme.bg()))
            .wrap(Wrap { trim: true });

        f.render_widget(paragraph, area);
    }
}
