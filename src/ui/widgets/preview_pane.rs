use pulldown_cmark::{Event, HeadingLevel, Parser, Tag};
use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph},
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

    fn wrap_spans(spans: &[Span<'static>], wrap_width: usize) -> Vec<Line<'static>> {
        if spans.is_empty() {
            return vec![Line::from("")];
        }

        let full_text: String = spans.iter().map(|s| s.content.as_ref()).collect();
        if full_text.chars().count() <= wrap_width {
            return vec![Line::from(spans.to_vec())];
        }

        let mut result = Vec::new();
        let mut current_spans: Vec<Span<'static>> = Vec::new();
        let mut current_len = 0;

        for span in spans {
            let chars: Vec<char> = span.content.chars().collect();
            let mut pos = 0;

            while pos < chars.len() {
                let remaining = wrap_width - current_len;
                if remaining == 0 {
                    if !current_spans.is_empty() {
                        result.push(Line::from(current_spans.clone()));
                        current_spans.clear();
                    }
                    current_len = 0;
                }

                let take = remaining.min(chars.len() - pos);
                let chunk: String = chars[pos..pos + take].iter().collect();
                current_spans.push(Span::styled(chunk, span.style));
                current_len += take;
                pos += take;
            }
        }

        if !current_spans.is_empty() {
            result.push(Line::from(current_spans));
        }

        if result.is_empty() {
            result.push(Line::from(""));
        }

        result
    }

    fn render_markdown(&self, theme: &dyn crate::config::Theme, width: usize) -> Text<'static> {
        if self.content.is_empty() {
            return Text::from("Select a note to preview");
        }

        let parser = Parser::new(&self.content);
        let mut lines: Vec<Vec<Span<'static>>> = vec![];
        let mut current_spans: Vec<Span<'static>> = vec![];
        let mut in_code_block = false;
        #[allow(unused_assignments)]
        let mut code_block_lang = String::new();
        let mut in_list = false;
        let mut list_number = 0;
        let mut in_heading = false;
        let mut heading_level = HeadingLevel::H1;
        let mut in_emphasis = false;
        let mut in_strong = false;
        let mut in_link = false;
        let mut link_url = String::new();
        let mut list_item_started = false;

        let push_current = |spans: &mut Vec<Span<'static>>, out: &mut Vec<Vec<Span<'static>>>| {
            if !spans.is_empty() {
                out.push(std::mem::take(spans));
            }
        };

        for event in parser {
            match event {
                Event::Start(tag) => match tag {
                    Tag::Heading(level, _, _) => {
                        push_current(&mut current_spans, &mut lines);
                        in_heading = true;
                        heading_level = level;
                    }
                    Tag::Paragraph => {}
                    Tag::CodeBlock(kind) => {
                        push_current(&mut current_spans, &mut lines);
                        in_code_block = true;
                        code_block_lang = match kind {
                            pulldown_cmark::CodeBlockKind::Fenced(lang) => lang.to_string(),
                            pulldown_cmark::CodeBlockKind::Indented => String::new(),
                        };
                        if !code_block_lang.is_empty() {
                            lines.push(vec![Span::styled(
                                format!("```{}", code_block_lang),
                                Style::default().fg(theme.fg_secondary()),
                            )]);
                        } else {
                            lines.push(vec![Span::styled(
                                "```",
                                Style::default().fg(theme.fg_secondary()),
                            )]);
                        }
                    }
                    Tag::List(Some(start_number)) => {
                        in_list = true;
                        list_number = start_number;
                        list_item_started = false;
                    }
                    Tag::List(None) => {
                        in_list = true;
                        list_number = 0;
                        list_item_started = false;
                    }
                    Tag::Item => {
                        push_current(&mut current_spans, &mut lines);
                        list_item_started = false;
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
                        current_spans.push(Span::styled("[", Style::default().fg(theme.info())));
                    }
                    _ => {}
                },
                Event::End(tag) => match tag {
                    Tag::Heading(_, _, _) => {
                        push_current(&mut current_spans, &mut lines);
                        lines.push(vec![]);
                        in_heading = false;
                    }
                    Tag::Paragraph => {
                        push_current(&mut current_spans, &mut lines);
                        lines.push(vec![]);
                    }
                    Tag::CodeBlock(_) => {
                        in_code_block = false;
                        lines.push(vec![Span::styled(
                            "```",
                            Style::default().fg(theme.fg_secondary()),
                        )]);
                        lines.push(vec![]);
                    }
                    Tag::List(_) => {
                        in_list = false;
                        lines.push(vec![]);
                    }
                    Tag::Item => {
                        push_current(&mut current_spans, &mut lines);
                        list_number += 1;
                        list_item_started = false;
                    }
                    Tag::Emphasis => {
                        in_emphasis = false;
                    }
                    Tag::Strong => {
                        in_strong = false;
                    }
                    Tag::Link(_link_type, _dest, _title) => {
                        in_link = false;
                        current_spans.push(Span::styled("]", Style::default().fg(theme.info())));
                        if !link_url.is_empty() {
                            current_spans.push(Span::styled(
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
                        let base = Style::default()
                            .fg(theme.link())
                            .add_modifier(Modifier::BOLD);
                        match heading_level {
                            HeadingLevel::H1 => base.bg(theme.bg_highlight()),
                            _ => base,
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

                    if in_list && !list_item_started {
                        let marker = if list_number > 0 {
                            format!("{}. ", list_number)
                        } else {
                            "• ".to_string()
                        };
                        current_spans.push(Span::styled(
                            marker,
                            Style::default().fg(theme.fg_secondary()),
                        ));
                        list_item_started = true;
                    }

                    current_spans.push(Span::styled(text.to_string(), style));
                }
                Event::Code(text) => {
                    if !in_code_block {
                        current_spans.push(Span::styled(
                            format!("`{}`", text),
                            Style::default().fg(theme.accent()).bg(theme.bg_secondary()),
                        ));
                    } else {
                        current_spans.push(Span::styled(
                            text.to_string(),
                            Style::default().fg(theme.accent()).bg(theme.bg_secondary()),
                        ));
                    }
                }
                Event::SoftBreak => {
                    current_spans.push(Span::raw(" "));
                }
                Event::HardBreak => {
                    push_current(&mut current_spans, &mut lines);
                }
                Event::Rule => {
                    push_current(&mut current_spans, &mut lines);
                    lines.push(vec![Span::styled(
                        "─".repeat(width.saturating_sub(4).max(10)),
                        Style::default().fg(theme.border()),
                    )]);
                    lines.push(vec![]);
                }
                _ => {}
            }
        }

        push_current(&mut current_spans, &mut lines);

        let wrap_width = width.saturating_sub(4).max(20);
        let mut wrapped_lines: Vec<Line> = Vec::new();

        for spans in lines {
            let wrapped = Self::wrap_spans(&spans, wrap_width);
            wrapped_lines.extend(wrapped);
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
            .scroll((self.scroll, 0));

        f.render_widget(paragraph, area);
    }
}

impl Default for PreviewPane {
    fn default() -> Self {
        Self::new()
    }
}
