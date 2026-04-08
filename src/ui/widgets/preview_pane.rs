use pulldown_cmark::{CodeBlockKind, Event, HeadingLevel, Options, Parser, Tag, TagEnd};
use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use regex::Regex;

/// Maximum nesting depth for blockquotes to prevent rendering issues
const MAX_BLOCKQUOTE_DEPTH: usize = 5;
/// Indent per list nesting level (in spaces)
const LIST_INDENT: usize = 2;

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

    /// Word-aware text wrapping for styled spans.
    /// Breaks at word boundaries when possible, falls back to character-level wrapping.
    fn wrap_spans(spans: &[Span<'static>], wrap_width: usize) -> Vec<Line<'static>> {
        if spans.is_empty() {
            return vec![Line::from("")];
        }

        let full_text: String = spans.iter().map(|s| s.content.as_ref()).collect();
        if full_text.chars().count() <= wrap_width {
            return vec![Line::from(spans.to_vec())];
        }

        // Flatten all spans into (char, Style) pairs
        let styled_chars: Vec<(char, Style)> = spans
            .iter()
            .flat_map(|span| span.content.chars().map(move |c| (c, span.style)))
            .collect();

        let mut result: Vec<Line<'static>> = Vec::new();
        let mut line_start = 0;

        while line_start < styled_chars.len() {
            let remaining = styled_chars.len() - line_start;
            if remaining <= wrap_width {
                // Last chunk fits entirely
                let line = build_line_from_styled_chars(&styled_chars[line_start..]);
                result.push(line);
                break;
            }

            // Find the last space within wrap_width for word-boundary wrapping
            let end = line_start + wrap_width;
            let mut break_at = None;
            for i in (line_start..end).rev() {
                if styled_chars[i].0 == ' ' {
                    break_at = Some(i);
                    break;
                }
            }

            match break_at {
                Some(pos) => {
                    // Break at word boundary (include content up to the space)
                    let line = build_line_from_styled_chars(&styled_chars[line_start..pos]);
                    result.push(line);
                    line_start = pos + 1; // Skip the space
                }
                None => {
                    // No space found, break at wrap_width (character-level)
                    let line = build_line_from_styled_chars(&styled_chars[line_start..end]);
                    result.push(line);
                    line_start = end;
                }
            }
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

        // Pre-process wiki-links: [[target]] or [[target|label]]
        // Convert them to markdown links so pulldown-cmark handles them
        let content = preprocess_wiki_links(&self.content);

        // Enable all markdown extensions
        let mut options = Options::empty();
        options.insert(Options::ENABLE_TABLES);
        options.insert(Options::ENABLE_FOOTNOTES);
        options.insert(Options::ENABLE_STRIKETHROUGH);
        options.insert(Options::ENABLE_TASKLISTS);
        options.insert(Options::ENABLE_HEADING_ATTRIBUTES);
        options.insert(Options::ENABLE_SMART_PUNCTUATION);

        let parser = Parser::new_ext(&content, options);

        let mut lines: Vec<Vec<Span<'static>>> = vec![];
        let mut current_spans: Vec<Span<'static>> = vec![];

        // State tracking
        let mut in_code_block = false;
        #[allow(unused_assignments)]
        let mut code_block_lang = String::new();
        let mut in_heading = false;
        let mut heading_level = HeadingLevel::H1;
        let mut in_emphasis = false;
        let mut in_strong = false;
        let mut in_strikethrough = false;
        let mut in_link = false;
        let mut link_url = String::new();
        let mut is_wiki_link = false;
        let mut in_image = false;
        let mut image_url = String::new();

        // List state stack (supports nested lists)
        let mut list_stack: Vec<ListState> = vec![];
        let mut list_item_started = false;

        // Blockquote state
        let mut blockquote_depth: usize = 0;

        // Table state
        let mut _in_table = false;
        let mut table_alignments: Vec<pulldown_cmark::Alignment> = vec![];
        let mut in_table_head = false;
        let mut table_row_cells: Vec<Vec<Span<'static>>> = vec![];
        let mut in_table_cell = false;

        // Footnote state
        let mut in_footnote_def = false;
        let mut footnote_label = String::new();

        let push_current = |spans: &mut Vec<Span<'static>>, out: &mut Vec<Vec<Span<'static>>>| {
            if !spans.is_empty() {
                out.push(std::mem::take(spans));
            }
        };

        for event in parser {
            match event {
                // ── Start tags ──────────────────────────────────────
                Event::Start(tag) => match tag {
                    Tag::Heading {
                        level,
                        id: _,
                        classes: _,
                        attrs: _,
                    } => {
                        push_current(&mut current_spans, &mut lines);
                        in_heading = true;
                        heading_level = level;

                        // Add heading prefix indicator
                        let prefix = match level {
                            HeadingLevel::H1 => "# ",
                            HeadingLevel::H2 => "## ",
                            HeadingLevel::H3 => "### ",
                            HeadingLevel::H4 => "#### ",
                            HeadingLevel::H5 => "##### ",
                            HeadingLevel::H6 => "###### ",
                        };
                        current_spans.push(Span::styled(
                            prefix.to_string(),
                            Style::default()
                                .fg(theme.fg_secondary())
                                .add_modifier(Modifier::BOLD),
                        ));
                    }
                    Tag::Paragraph => {
                        // Add blockquote prefix if inside a blockquote
                        if blockquote_depth > 0 && !in_footnote_def {
                            let prefix = build_blockquote_prefix(blockquote_depth, theme);
                            current_spans.extend(prefix);
                        }
                    }
                    Tag::BlockQuote(_kind) => {
                        push_current(&mut current_spans, &mut lines);
                        blockquote_depth = (blockquote_depth + 1).min(MAX_BLOCKQUOTE_DEPTH);
                    }
                    Tag::CodeBlock(kind) => {
                        push_current(&mut current_spans, &mut lines);
                        in_code_block = true;
                        code_block_lang = match kind {
                            CodeBlockKind::Fenced(lang) => lang.to_string(),
                            CodeBlockKind::Indented => String::new(),
                        };
                        // Top border of code block
                        let header = if !code_block_lang.is_empty() {
                            format!("┌─ {} ", code_block_lang)
                        } else {
                            "┌──".to_string()
                        };
                        let border_fill = "─".repeat(width.saturating_sub(header.len() + 5).max(1));
                        lines.push(vec![Span::styled(
                            format!("{}{}", header, border_fill),
                            Style::default().fg(theme.fg_secondary()),
                        )]);
                    }
                    Tag::List(Some(start_number)) => {
                        if list_stack.is_empty() {
                            push_current(&mut current_spans, &mut lines);
                        }
                        list_stack.push(ListState::Ordered(start_number));
                        list_item_started = false;
                    }
                    Tag::List(None) => {
                        if list_stack.is_empty() {
                            push_current(&mut current_spans, &mut lines);
                        }
                        list_stack.push(ListState::Unordered);
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
                    Tag::Strikethrough => {
                        in_strikethrough = true;
                    }
                    Tag::Link {
                        dest_url, title: _, ..
                    } => {
                        in_link = true;
                        link_url = dest_url.to_string();
                        is_wiki_link = link_url.starts_with("zettel://");
                        if is_wiki_link {
                            current_spans
                                .push(Span::styled("[[", Style::default().fg(theme.info())));
                        } else {
                            current_spans
                                .push(Span::styled("[", Style::default().fg(theme.info())));
                        }
                    }
                    Tag::Image {
                        dest_url, title: _, ..
                    } => {
                        in_image = true;
                        image_url = dest_url.to_string();
                    }
                    Tag::Table(alignments) => {
                        push_current(&mut current_spans, &mut lines);
                        _in_table = true;
                        table_alignments = alignments;
                    }
                    Tag::TableHead => {
                        in_table_head = true;
                        table_row_cells.clear();
                    }
                    Tag::TableRow => {
                        table_row_cells.clear();
                    }
                    Tag::TableCell => {
                        in_table_cell = true;
                        // Cell content will be accumulated in current_spans
                    }
                    Tag::FootnoteDefinition(label) => {
                        push_current(&mut current_spans, &mut lines);
                        in_footnote_def = true;
                        footnote_label = label.to_string();
                        // Add footnote definition marker
                        current_spans.push(Span::styled(
                            format!("[^{}]: ", footnote_label),
                            Style::default()
                                .fg(theme.accent_secondary())
                                .add_modifier(Modifier::BOLD),
                        ));
                    }
                    _ => {}
                },

                // ── End tags ────────────────────────────────────────
                Event::End(tag) => match tag {
                    TagEnd::Heading(_) => {
                        push_current(&mut current_spans, &mut lines);
                        lines.push(vec![]); // blank line after heading
                        in_heading = false;
                    }
                    TagEnd::Paragraph => {
                        push_current(&mut current_spans, &mut lines);
                        if !in_footnote_def {
                            lines.push(vec![]); // blank line after paragraph
                        }
                    }
                    TagEnd::BlockQuote(_) => {
                        push_current(&mut current_spans, &mut lines);
                        blockquote_depth = blockquote_depth.saturating_sub(1);
                        if blockquote_depth == 0 {
                            lines.push(vec![]); // blank line after outermost blockquote
                        }
                    }
                    TagEnd::CodeBlock => {
                        in_code_block = false;
                        // Bottom border of code block
                        let border = "─".repeat(width.saturating_sub(5).max(10));
                        lines.push(vec![Span::styled(
                            format!("└{}", border),
                            Style::default().fg(theme.fg_secondary()),
                        )]);
                        lines.push(vec![]);
                    }
                    TagEnd::List(_) => {
                        list_stack.pop();
                        if list_stack.is_empty() {
                            push_current(&mut current_spans, &mut lines);
                            lines.push(vec![]); // blank line after top-level list
                        }
                    }
                    TagEnd::Item => {
                        push_current(&mut current_spans, &mut lines);
                        // Increment ordered list counter
                        if let Some(ListState::Ordered(n)) = list_stack.last_mut() {
                            *n += 1;
                        }
                        list_item_started = false;
                    }
                    TagEnd::Emphasis => {
                        in_emphasis = false;
                    }
                    TagEnd::Strong => {
                        in_strong = false;
                    }
                    TagEnd::Strikethrough => {
                        in_strikethrough = false;
                    }
                    TagEnd::Link => {
                        in_link = false;
                        if is_wiki_link {
                            current_spans
                                .push(Span::styled("]]", Style::default().fg(theme.info())));
                        } else {
                            current_spans
                                .push(Span::styled("]", Style::default().fg(theme.info())));
                            if !link_url.is_empty() {
                                current_spans.push(Span::styled(
                                    format!("({})", link_url),
                                    Style::default().fg(theme.fg_secondary()),
                                ));
                            }
                        }
                        is_wiki_link = false;
                    }
                    TagEnd::Image => {
                        in_image = false;
                        // Render image placeholder
                        current_spans.push(Span::styled(
                            format!(" [{}]", image_url),
                            Style::default().fg(theme.fg_secondary()),
                        ));
                    }
                    TagEnd::Table => {
                        _in_table = false;
                        table_alignments.clear();
                        lines.push(vec![]);
                    }
                    TagEnd::TableHead => {
                        // Render header row
                        let rendered = render_table_row(
                            &table_row_cells,
                            &table_alignments,
                            width,
                            theme,
                            true,
                        );
                        lines.extend(rendered);
                        table_row_cells.clear();
                        in_table_head = false;
                    }
                    TagEnd::TableRow => {
                        if !in_table_head {
                            let rendered = render_table_row(
                                &table_row_cells,
                                &table_alignments,
                                width,
                                theme,
                                false,
                            );
                            lines.extend(rendered);
                        }
                        table_row_cells.clear();
                    }
                    TagEnd::TableCell => {
                        in_table_cell = false;
                        // Save cell content
                        table_row_cells.push(std::mem::take(&mut current_spans));
                    }
                    TagEnd::FootnoteDefinition => {
                        push_current(&mut current_spans, &mut lines);
                        in_footnote_def = false;
                        footnote_label.clear();
                        lines.push(vec![]);
                    }
                    _ => {}
                },

                // ── Text content ────────────────────────────────────
                Event::Text(text) => {
                    if in_image {
                        // Render image alt text as placeholder
                        current_spans.push(Span::styled(
                            format!("[IMG: {}]", text),
                            Style::default()
                                .fg(theme.warning())
                                .add_modifier(Modifier::ITALIC),
                        ));
                        continue;
                    }

                    let style = compute_text_style(
                        theme,
                        in_code_block,
                        in_heading,
                        heading_level,
                        in_strong,
                        in_emphasis,
                        in_strikethrough,
                        in_link,
                        in_table_head,
                    );

                    if in_code_block {
                        // Render each code line separately with a gutter marker
                        for (i, code_line) in text.split('\n').enumerate() {
                            if i > 0 {
                                push_current(&mut current_spans, &mut lines);
                            }
                            current_spans.push(Span::styled(
                                "│ ",
                                Style::default().fg(theme.fg_secondary()),
                            ));
                            current_spans.push(Span::styled(code_line.to_string(), style));
                        }
                    } else if in_table_cell {
                        current_spans.push(Span::styled(text.to_string(), style));
                    } else {
                        // Handle list item prefix
                        if !list_stack.is_empty() && !list_item_started {
                            let indent_level = list_stack.len().saturating_sub(1);
                            let indent = " ".repeat(indent_level * LIST_INDENT);

                            let marker = match list_stack.last() {
                                Some(ListState::Ordered(n)) => {
                                    format!("{}. ", n)
                                }
                                Some(ListState::Unordered) => {
                                    // Different bullet styles for nesting levels
                                    let bullet = match indent_level % 3 {
                                        0 => "•",
                                        1 => "◦",
                                        _ => "▸",
                                    };
                                    format!("{} ", bullet)
                                }
                                None => String::new(),
                            };

                            if !marker.is_empty() {
                                current_spans.push(Span::styled(
                                    format!("{}{}", indent, marker),
                                    Style::default().fg(theme.fg_secondary()),
                                ));
                            }
                            list_item_started = true;
                        }

                        // Add blockquote prefix for text lines
                        if blockquote_depth > 0 && current_spans.is_empty() && !in_footnote_def {
                            let prefix = build_blockquote_prefix(blockquote_depth, theme);
                            current_spans.extend(prefix);
                        }

                        current_spans.push(Span::styled(text.to_string(), style));
                    }
                }

                // ── Inline code ─────────────────────────────────────
                Event::Code(text) => {
                    let style = Style::default().fg(theme.accent()).bg(theme.bg_secondary());
                    if !in_code_block {
                        current_spans.push(Span::styled(format!("`{}`", text), style));
                    } else {
                        current_spans.push(Span::styled(text.to_string(), style));
                    }
                }

                // ── Breaks ──────────────────────────────────────────
                Event::SoftBreak => {
                    current_spans.push(Span::raw(" "));
                }
                Event::HardBreak => {
                    push_current(&mut current_spans, &mut lines);
                }

                // ── Horizontal rule ─────────────────────────────────
                Event::Rule => {
                    push_current(&mut current_spans, &mut lines);
                    lines.push(vec![Span::styled(
                        "─".repeat(width.saturating_sub(4).max(10)),
                        Style::default().fg(theme.border()),
                    )]);
                    lines.push(vec![]);
                }

                // ── Task list marker ────────────────────────────────
                Event::TaskListMarker(checked) => {
                    let indent_level = list_stack.len().saturating_sub(1);
                    let indent = " ".repeat(indent_level * LIST_INDENT);

                    let (marker, color) = if checked {
                        ("[x]", theme.success())
                    } else {
                        ("[ ]", theme.fg_secondary())
                    };

                    current_spans.push(Span::styled(
                        format!("{}{} ", indent, marker),
                        Style::default().fg(color),
                    ));
                    list_item_started = true;
                }

                // ── Footnote reference ──────────────────────────────
                Event::FootnoteReference(label) => {
                    current_spans.push(Span::styled(
                        format!("[^{}]", label),
                        Style::default()
                            .fg(theme.accent_secondary())
                            .add_modifier(Modifier::BOLD),
                    ));
                }

                // ── HTML (inline and block) ─────────────────────────
                Event::Html(html) | Event::InlineHtml(html) => {
                    current_spans.push(Span::styled(
                        html.to_string(),
                        Style::default().fg(theme.fg_dim()),
                    ));
                }

                _ => {}
            }
        }

        push_current(&mut current_spans, &mut lines);

        // Apply word-aware wrapping
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

// ── Helper types ─────────────────────────────────────────────────

#[derive(Debug, Clone)]
enum ListState {
    Ordered(u64),
    Unordered,
}

// ── Helper functions ─────────────────────────────────────────────

/// Build a Line from a slice of (char, Style) pairs, coalescing runs of the same style.
fn build_line_from_styled_chars(chars: &[(char, Style)]) -> Line<'static> {
    if chars.is_empty() {
        return Line::from("");
    }

    let mut spans: Vec<Span<'static>> = Vec::new();
    let mut current_style = chars[0].1;
    let mut current_text = String::new();

    for &(c, style) in chars {
        if style == current_style {
            current_text.push(c);
        } else {
            if !current_text.is_empty() {
                spans.push(Span::styled(current_text.clone(), current_style));
                current_text.clear();
            }
            current_style = style;
            current_text.push(c);
        }
    }

    if !current_text.is_empty() {
        spans.push(Span::styled(current_text, current_style));
    }

    Line::from(spans)
}

/// Compute the text style based on current rendering context.
fn compute_text_style(
    theme: &dyn crate::config::Theme,
    in_code_block: bool,
    in_heading: bool,
    heading_level: HeadingLevel,
    in_strong: bool,
    in_emphasis: bool,
    in_strikethrough: bool,
    in_link: bool,
    in_table_head: bool,
) -> Style {
    if in_code_block {
        return Style::default().fg(theme.accent()).bg(theme.bg_secondary());
    }

    if in_heading {
        let base = Style::default()
            .fg(theme.link())
            .add_modifier(Modifier::BOLD);
        return match heading_level {
            HeadingLevel::H1 => base.bg(theme.bg_highlight()),
            HeadingLevel::H2 => base,
            _ => Style::default()
                .fg(theme.link())
                .add_modifier(Modifier::BOLD),
        };
    }

    if in_table_head {
        return Style::default()
            .fg(theme.accent())
            .add_modifier(Modifier::BOLD);
    }

    let mut style = Style::default().fg(theme.fg());

    if in_strong {
        style = style.fg(theme.success()).add_modifier(Modifier::BOLD);
    }

    if in_emphasis {
        style = style
            .fg(if in_strong {
                theme.success()
            } else {
                theme.warning()
            })
            .add_modifier(Modifier::ITALIC);
    }

    if in_strikethrough {
        style = style.add_modifier(Modifier::CROSSED_OUT);
    }

    if in_link {
        style = Style::default()
            .fg(theme.link())
            .add_modifier(Modifier::UNDERLINED);
    }

    style
}

/// Build the blockquote prefix spans (e.g., "│ " for depth 1, "│ │ " for depth 2).
fn build_blockquote_prefix(depth: usize, theme: &dyn crate::config::Theme) -> Vec<Span<'static>> {
    let mut spans = Vec::new();
    let depth = depth.min(MAX_BLOCKQUOTE_DEPTH);
    for i in 0..depth {
        let color = match i % 3 {
            0 => theme.accent(),
            1 => theme.info(),
            _ => theme.warning(),
        };
        spans.push(Span::styled("│ ", Style::default().fg(color)));
    }
    spans
}

/// Render a table row as styled lines, including separator for header.
fn render_table_row(
    cells: &[Vec<Span<'static>>],
    alignments: &[pulldown_cmark::Alignment],
    total_width: usize,
    theme: &dyn crate::config::Theme,
    is_header: bool,
) -> Vec<Vec<Span<'static>>> {
    let mut result = Vec::new();

    let num_cols = cells.len().max(1);
    let available_width = total_width.saturating_sub(4); // account for borders
    let col_width = if num_cols > 0 {
        available_width.saturating_sub(num_cols + 1) / num_cols // +1 for separators
    } else {
        available_width
    };
    let col_width = col_width.max(3);

    // Build the row
    let mut row_spans: Vec<Span<'static>> = vec![];
    row_spans.push(Span::styled("│", Style::default().fg(theme.border())));

    for (i, cell) in cells.iter().enumerate() {
        let cell_text: String = cell.iter().map(|s| s.content.as_ref()).collect();
        let alignment = alignments
            .get(i)
            .copied()
            .unwrap_or(pulldown_cmark::Alignment::None);
        let padded = align_text(&cell_text, col_width, alignment);

        let style = if is_header {
            Style::default()
                .fg(theme.accent())
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(theme.fg())
        };

        row_spans.push(Span::styled(format!(" {} ", padded), style));
        row_spans.push(Span::styled("│", Style::default().fg(theme.border())));
    }

    result.push(row_spans);

    // Add separator line below header
    if is_header {
        let mut sep_spans: Vec<Span<'static>> = vec![];
        sep_spans.push(Span::styled("├", Style::default().fg(theme.border())));
        for (i, _) in cells.iter().enumerate() {
            let alignment = alignments
                .get(i)
                .copied()
                .unwrap_or(pulldown_cmark::Alignment::None);
            let sep = match alignment {
                pulldown_cmark::Alignment::Left => format!(":{}─", "─".repeat(col_width)),
                pulldown_cmark::Alignment::Right => format!("{}─:", "─".repeat(col_width)),
                pulldown_cmark::Alignment::Center => {
                    format!(":{}:", "─".repeat(col_width))
                }
                _ => format!("─{}─", "─".repeat(col_width)),
            };
            sep_spans.push(Span::styled(sep, Style::default().fg(theme.border())));
            if i < cells.len() - 1 {
                sep_spans.push(Span::styled("┼", Style::default().fg(theme.border())));
            }
        }
        sep_spans.push(Span::styled("┤", Style::default().fg(theme.border())));
        result.push(sep_spans);
    }

    result
}

/// Align text within a given width according to the alignment directive.
fn align_text(text: &str, width: usize, alignment: pulldown_cmark::Alignment) -> String {
    let text_len = text.chars().count();
    if text_len >= width {
        return text.chars().take(width).collect();
    }
    let padding = width - text_len;
    match alignment {
        pulldown_cmark::Alignment::Right => format!("{}{}", " ".repeat(padding), text),
        pulldown_cmark::Alignment::Center => {
            let left = padding / 2;
            let right = padding - left;
            format!("{}{}{}", " ".repeat(left), text, " ".repeat(right))
        }
        _ => format!("{}{}", text, " ".repeat(padding)),
    }
}

/// Pre-process wiki-links ([[target]] or [[target|label]]) into a format
/// pulldown-cmark can parse as standard links, with a custom scheme to
/// identify them later during rendering.
fn preprocess_wiki_links(content: &str) -> String {
    let re = Regex::new(r"\[\[([^\]|]+)(?:\|([^\]]+))?\]\]").unwrap();
    re.replace_all(content, |caps: &regex::Captures| {
        let target = &caps[1];
        let label = caps.get(2).map_or(target, |m| m.as_str());
        format!("[{}](zettel://{})", label, target)
    })
    .to_string()
}

// ── Tests ────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::style::Color;

    // ── Test theme implementation ───────────────────────────────
    struct TestTheme;

    impl crate::config::Theme for TestTheme {
        fn name(&self) -> &str {
            "test"
        }
        fn bg(&self) -> Color {
            Color::Black
        }
        fn bg_secondary(&self) -> Color {
            Color::DarkGray
        }
        fn bg_highlight(&self) -> Color {
            Color::Gray
        }
        fn fg(&self) -> Color {
            Color::White
        }
        fn fg_secondary(&self) -> Color {
            Color::Gray
        }
        fn fg_dim(&self) -> Color {
            Color::DarkGray
        }
        fn accent(&self) -> Color {
            Color::Cyan
        }
        fn accent_secondary(&self) -> Color {
            Color::Magenta
        }
        fn success(&self) -> Color {
            Color::Green
        }
        fn warning(&self) -> Color {
            Color::Yellow
        }
        fn error(&self) -> Color {
            Color::Red
        }
        fn info(&self) -> Color {
            Color::Blue
        }
        fn note_daily(&self) -> Color {
            Color::Cyan
        }
        fn note_fleeting(&self) -> Color {
            Color::Yellow
        }
        fn note_literature(&self) -> Color {
            Color::Magenta
        }
        fn note_permanent(&self) -> Color {
            Color::Green
        }
        fn note_reference(&self) -> Color {
            Color::Blue
        }
        fn note_index(&self) -> Color {
            Color::Red
        }
        fn link(&self) -> Color {
            Color::Cyan
        }
        fn tag(&self) -> Color {
            Color::Magenta
        }
        fn border(&self) -> Color {
            Color::Gray
        }
        fn border_highlight(&self) -> Color {
            Color::White
        }
    }

    // Helper to render markdown to text representation
    fn render(content: &str) -> String {
        let pane = PreviewPane {
            content: content.to_string(),
            scroll: 0,
        };
        let theme = TestTheme;
        let text = pane.render_markdown(&theme, 80);
        text.lines
            .iter()
            .map(|line| {
                line.spans
                    .iter()
                    .map(|s| s.content.as_ref())
                    .collect::<String>()
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    // Helper to get styled spans for detailed inspection
    fn render_spans(content: &str) -> Vec<Vec<(String, Style)>> {
        let pane = PreviewPane {
            content: content.to_string(),
            scroll: 0,
        };
        let theme = TestTheme;
        let text = pane.render_markdown(&theme, 80);
        text.lines
            .iter()
            .map(|line| {
                line.spans
                    .iter()
                    .map(|s| (s.content.to_string(), s.style))
                    .collect()
            })
            .collect()
    }

    // ── Basic rendering ─────────────────────────────────────────

    #[test]
    fn test_empty_content() {
        let pane = PreviewPane::new();
        let theme = TestTheme;
        let text = pane.render_markdown(&theme, 80);
        let content: String = text
            .lines
            .iter()
            .flat_map(|l| l.spans.iter().map(|s| s.content.to_string()))
            .collect();
        assert_eq!(content, "Select a note to preview");
    }

    #[test]
    fn test_plain_text_paragraph() {
        let output = render("Hello world");
        assert!(output.contains("Hello world"));
    }

    // ── Headings ────────────────────────────────────────────────

    #[test]
    fn test_heading_h1() {
        let output = render("# Title");
        assert!(output.contains("# "));
        assert!(output.contains("Title"));
    }

    #[test]
    fn test_heading_h2() {
        let output = render("## Subtitle");
        assert!(output.contains("## "));
        assert!(output.contains("Subtitle"));
    }

    #[test]
    fn test_heading_h3() {
        let output = render("### Section");
        assert!(output.contains("### "));
        assert!(output.contains("Section"));
    }

    #[test]
    fn test_heading_levels_styled() {
        let spans = render_spans("# H1 Title");
        let first_line = &spans[0];
        // Should have prefix and title, both bold
        assert!(first_line.iter().any(|(text, _)| text == "# "));
        assert!(first_line.iter().any(|(text, style)| {
            text == "H1 Title" && style.add_modifier.contains(Modifier::BOLD)
        }));
    }

    #[test]
    fn test_heading_h1_has_bg_highlight() {
        let spans = render_spans("# Big Title");
        let first_line = &spans[0];
        let title_span = first_line.iter().find(|(text, _)| text == "Big Title");
        assert!(title_span.is_some());
        let (_, style) = title_span.unwrap();
        // H1 should have background highlight
        assert!(style.bg.is_some());
    }

    // ── Bold / Italic / Strikethrough ───────────────────────────

    #[test]
    fn test_bold_text() {
        let spans = render_spans("**bold text**");
        let has_bold = spans.iter().any(|line| {
            line.iter().any(|(text, style)| {
                text == "bold text" && style.add_modifier.contains(Modifier::BOLD)
            })
        });
        assert!(has_bold);
    }

    #[test]
    fn test_italic_text() {
        let spans = render_spans("*italic text*");
        let has_italic = spans.iter().any(|line| {
            line.iter().any(|(text, style)| {
                text == "italic text" && style.add_modifier.contains(Modifier::ITALIC)
            })
        });
        assert!(has_italic);
    }

    #[test]
    fn test_bold_italic_combined() {
        let spans = render_spans("***bold italic***");
        let has_both = spans.iter().any(|line| {
            line.iter().any(|(_, style)| {
                style.add_modifier.contains(Modifier::BOLD)
                    && style.add_modifier.contains(Modifier::ITALIC)
            })
        });
        assert!(has_both);
    }

    #[test]
    fn test_strikethrough() {
        let spans = render_spans("~~deleted~~");
        let has_strikethrough = spans.iter().any(|line| {
            line.iter().any(|(text, style)| {
                text == "deleted" && style.add_modifier.contains(Modifier::CROSSED_OUT)
            })
        });
        assert!(has_strikethrough);
    }

    // ── Inline code ─────────────────────────────────────────────

    #[test]
    fn test_inline_code() {
        let output = render("Use `cargo build` to compile");
        assert!(output.contains("`cargo build`"));
    }

    #[test]
    fn test_inline_code_style() {
        let spans = render_spans("Run `test`");
        let has_code = spans.iter().any(|line| {
            line.iter()
                .any(|(text, style)| text == "`test`" && style.bg.is_some())
        });
        assert!(has_code);
    }

    // ── Code blocks ─────────────────────────────────────────────

    #[test]
    fn test_code_block_with_language() {
        let md = "```rust\nfn main() {}\n```";
        let output = render(md);
        assert!(output.contains("rust"));
        assert!(output.contains("fn main() {}"));
    }

    #[test]
    fn test_code_block_borders() {
        let md = "```\nhello\n```";
        let output = render(md);
        assert!(output.contains("┌"));
        assert!(output.contains("└"));
        assert!(output.contains("│ "));
    }

    #[test]
    fn test_code_block_gutter() {
        let md = "```\nline1\nline2\n```";
        let spans = render_spans(md);
        let gutter_count = spans
            .iter()
            .filter(|line| line.iter().any(|(text, _)| text == "│ "))
            .count();
        assert!(gutter_count >= 2);
    }

    // ── Links ───────────────────────────────────────────────────

    #[test]
    fn test_markdown_link() {
        let output = render("[Click here](https://example.com)");
        assert!(output.contains("["));
        assert!(output.contains("Click here"));
        assert!(output.contains("(https://example.com)"));
    }

    #[test]
    fn test_link_style() {
        let spans = render_spans("[link](http://url.com)");
        let has_underlined_link = spans.iter().any(|line| {
            line.iter().any(|(text, style)| {
                text == "link" && style.add_modifier.contains(Modifier::UNDERLINED)
            })
        });
        assert!(has_underlined_link);
    }

    // ── Wiki links ──────────────────────────────────────────────

    #[test]
    fn test_wiki_link_simple() {
        let output = render("See [[my-note]]");
        assert!(output.contains("[["));
        assert!(output.contains("my-note"));
        assert!(output.contains("]]"));
    }

    #[test]
    fn test_wiki_link_with_label() {
        let output = render("See [[target|Display Text]]");
        assert!(output.contains("[["));
        assert!(output.contains("Display Text"));
        assert!(output.contains("]]"));
    }

    #[test]
    fn test_preprocess_wiki_links() {
        let result = preprocess_wiki_links("Link to [[my-note]]");
        assert_eq!(result, "Link to [my-note](zettel://my-note)");
    }

    #[test]
    fn test_preprocess_wiki_links_with_label() {
        let result = preprocess_wiki_links("Link to [[target|label]]");
        assert_eq!(result, "Link to [label](zettel://target)");
    }

    // ── Lists ───────────────────────────────────────────────────

    #[test]
    fn test_unordered_list() {
        let md = "- Item 1\n- Item 2\n- Item 3";
        let output = render(md);
        assert!(output.contains("• "));
        assert!(output.contains("Item 1"));
        assert!(output.contains("Item 2"));
        assert!(output.contains("Item 3"));
    }

    #[test]
    fn test_ordered_list() {
        let md = "1. First\n2. Second\n3. Third";
        let output = render(md);
        assert!(output.contains("1. "));
        assert!(output.contains("First"));
        assert!(output.contains("2. "));
        assert!(output.contains("Second"));
    }

    #[test]
    fn test_nested_list() {
        let md = "- Outer\n  - Inner\n    - Deep";
        let output = render(md);
        assert!(output.contains("• "));
        assert!(output.contains("Outer"));
        assert!(output.contains("Inner"));
        // Should have nested bullet styles
        assert!(output.contains("◦") || output.contains("▸"));
    }

    // ── Task lists ──────────────────────────────────────────────

    #[test]
    fn test_task_list_unchecked() {
        let md = "- [ ] Not done";
        let output = render(md);
        assert!(output.contains("[ ]"));
        assert!(output.contains("Not done"));
    }

    #[test]
    fn test_task_list_checked() {
        let md = "- [x] Done";
        let output = render(md);
        assert!(output.contains("[x]"));
        assert!(output.contains("Done"));
    }

    #[test]
    fn test_task_list_mixed() {
        let md = "- [x] Task 1\n- [ ] Task 2\n- [x] Task 3";
        let output = render(md);
        assert!(output.contains("[x]"));
        assert!(output.contains("[ ]"));
        assert!(output.contains("Task 1"));
        assert!(output.contains("Task 2"));
    }

    // ── Blockquotes ─────────────────────────────────────────────

    #[test]
    fn test_blockquote_simple() {
        let md = "> This is a quote";
        let output = render(md);
        assert!(output.contains("│ "));
        assert!(output.contains("This is a quote"));
    }

    #[test]
    fn test_blockquote_nested() {
        let md = "> Level 1\n>> Level 2";
        let output = render(md);
        assert!(output.contains("Level 1"));
        assert!(output.contains("Level 2"));
        // Should have multiple "│ " prefixes for nested blockquotes
        let double_bar_count = output.matches("│ ").count();
        assert!(double_bar_count >= 2);
    }

    // ── Tables ──────────────────────────────────────────────────

    #[test]
    fn test_table_basic() {
        let md = "| Name | Age |\n|------|-----|\n| Alice | 30 |\n| Bob | 25 |";
        let output = render(md);
        assert!(output.contains("Name"));
        assert!(output.contains("Age"));
        assert!(output.contains("Alice"));
        assert!(output.contains("Bob"));
        assert!(output.contains("│"));
    }

    #[test]
    fn test_table_separator() {
        let md = "| A | B |\n|---|---|\n| 1 | 2 |";
        let output = render(md);
        // Should have separator line characters
        assert!(output.contains("─"));
        assert!(output.contains("├"));
        assert!(output.contains("┤"));
    }

    #[test]
    fn test_table_alignment() {
        let md = "| Left | Center | Right |\n|:-----|:------:|------:|\n| a | b | c |";
        let output = render(md);
        assert!(output.contains("Left"));
        assert!(output.contains("Center"));
        assert!(output.contains("Right"));
    }

    // ── Images ──────────────────────────────────────────────────

    #[test]
    fn test_image_placeholder() {
        let md = "![Alt text](image.png)";
        let output = render(md);
        assert!(output.contains("[IMG: Alt text]"));
    }

    #[test]
    fn test_image_with_url() {
        let md = "![photo](https://example.com/photo.jpg)";
        let output = render(md);
        assert!(output.contains("[IMG: photo]"));
        assert!(output.contains("[https://example.com/photo.jpg]"));
    }

    // ── Footnotes ───────────────────────────────────────────────

    #[test]
    fn test_footnote_reference() {
        let md = "Text with footnote[^1]\n\n[^1]: This is the footnote";
        let output = render(md);
        assert!(output.contains("[^1]"));
    }

    #[test]
    fn test_footnote_definition() {
        let md = "Something[^note]\n\n[^note]: Definition here";
        let output = render(md);
        assert!(output.contains("Definition here"));
    }

    // ── Horizontal rule ─────────────────────────────────────────

    #[test]
    fn test_horizontal_rule() {
        let md = "Above\n\n---\n\nBelow";
        let output = render(md);
        assert!(output.contains("─"));
        assert!(output.contains("Above"));
        assert!(output.contains("Below"));
    }

    // ── Word wrapping ───────────────────────────────────────────

    #[test]
    fn test_wrap_at_word_boundary() {
        let spans = vec![Span::raw(
            "Hello world this is a test of word wrapping behavior",
        )];
        let lines = PreviewPane::wrap_spans(&spans, 20);
        // Should break at word boundary, not in the middle of a word
        for line in &lines {
            let text: String = line.spans.iter().map(|s| s.content.as_ref()).collect();
            // No word should be split across lines (unless single word > width)
            assert!(text.chars().count() <= 20, "Line too long: '{}'", text);
        }
    }

    #[test]
    fn test_wrap_short_text_no_wrap() {
        let spans = vec![Span::raw("Short")];
        let lines = PreviewPane::wrap_spans(&spans, 80);
        assert_eq!(lines.len(), 1);
    }

    #[test]
    fn test_wrap_empty_spans() {
        let lines = PreviewPane::wrap_spans(&[], 80);
        assert_eq!(lines.len(), 1);
    }

    #[test]
    fn test_wrap_preserves_styles() {
        let spans = vec![
            Span::styled("Hello ", Style::default().fg(Color::Red)),
            Span::styled(
                "world this is long text that wraps",
                Style::default().fg(Color::Blue),
            ),
        ];
        let lines = PreviewPane::wrap_spans(&spans, 15);
        assert!(lines.len() > 1);
        // Styles should be preserved in wrapped lines
        let first_line_has_red = lines[0]
            .spans
            .iter()
            .any(|s| s.style.fg == Some(Color::Red));
        assert!(first_line_has_red);
    }

    // ── Scroll behavior ─────────────────────────────────────────

    #[test]
    fn test_scroll_up_at_zero() {
        let mut pane = PreviewPane::new();
        pane.scroll_up();
        assert_eq!(pane.scroll, 0);
    }

    #[test]
    fn test_scroll_down() {
        let mut pane = PreviewPane::new();
        pane.scroll_down();
        assert_eq!(pane.scroll, 1);
        pane.scroll_down();
        assert_eq!(pane.scroll, 2);
    }

    #[test]
    fn test_scroll_up() {
        let mut pane = PreviewPane::new();
        pane.scroll_down();
        pane.scroll_down();
        pane.scroll_up();
        assert_eq!(pane.scroll, 1);
    }

    #[test]
    fn test_scroll_to_top() {
        let mut pane = PreviewPane::new();
        pane.scroll = 50;
        pane.scroll_to_top();
        assert_eq!(pane.scroll, 0);
    }

    #[test]
    fn test_scroll_to_bottom() {
        let mut pane = PreviewPane::new();
        pane.scroll_to_bottom(100);
        assert_eq!(pane.scroll, 99);
    }

    #[test]
    fn test_set_content_resets_scroll() {
        let mut pane = PreviewPane::new();
        pane.scroll = 50;
        pane.set_content("New content");
        assert_eq!(pane.scroll, 0);
        assert_eq!(pane.content, "New content");
    }

    // ── Complex markdown ────────────────────────────────────────

    #[test]
    fn test_complex_markdown_document() {
        let md = r#"# Title

This is a paragraph with **bold** and *italic* text.

## Section 1

- Item 1
- Item 2
  - Nested item

> A quote

```rust
fn main() {}
```

| Col1 | Col2 |
|------|------|
| a    | b    |

---

[Link](http://example.com)

- [x] Done
- [ ] Not done

Text with footnote[^1]

[^1]: Footnote content
"#;
        let output = render(md);
        // Verify all major elements are present
        assert!(output.contains("# "));
        assert!(output.contains("Title"));
        assert!(output.contains("bold"));
        assert!(output.contains("italic"));
        assert!(output.contains("## "));
        assert!(output.contains("Section 1"));
        assert!(output.contains("• "));
        assert!(output.contains("│ "));
        assert!(output.contains("rust"));
        assert!(output.contains("fn main() {}"));
        assert!(output.contains("Col1"));
        assert!(output.contains("─"));
        assert!(output.contains("Link"));
        assert!(output.contains("[x]"));
        assert!(output.contains("[ ]"));
    }

    // ── Helper functions ────────────────────────────────────────

    #[test]
    fn test_align_text_left() {
        let result = align_text("hi", 10, pulldown_cmark::Alignment::Left);
        assert_eq!(result, "hi        ");
    }

    #[test]
    fn test_align_text_right() {
        let result = align_text("hi", 10, pulldown_cmark::Alignment::Right);
        assert_eq!(result, "        hi");
    }

    #[test]
    fn test_align_text_center() {
        let result = align_text("hi", 10, pulldown_cmark::Alignment::Center);
        assert_eq!(result, "    hi    ");
    }

    #[test]
    fn test_align_text_truncate() {
        let result = align_text("hello world", 5, pulldown_cmark::Alignment::Left);
        assert_eq!(result, "hello");
    }

    #[test]
    fn test_build_blockquote_prefix() {
        let theme = TestTheme;
        let prefix = build_blockquote_prefix(1, &theme);
        assert_eq!(prefix.len(), 1);
        assert_eq!(prefix[0].content.as_ref(), "│ ");
    }

    #[test]
    fn test_build_blockquote_prefix_nested() {
        let theme = TestTheme;
        let prefix = build_blockquote_prefix(3, &theme);
        assert_eq!(prefix.len(), 3);
        for span in &prefix {
            assert_eq!(span.content.as_ref(), "│ ");
        }
    }

    #[test]
    fn test_build_line_from_styled_chars() {
        let chars = vec![
            ('H', Style::default().fg(Color::Red)),
            ('i', Style::default().fg(Color::Red)),
            (' ', Style::default().fg(Color::Blue)),
            ('!', Style::default().fg(Color::Blue)),
        ];
        let line = build_line_from_styled_chars(&chars);
        assert_eq!(line.spans.len(), 2);
        assert_eq!(line.spans[0].content.as_ref(), "Hi");
        assert_eq!(line.spans[1].content.as_ref(), " !");
    }

    #[test]
    fn test_build_line_from_styled_chars_empty() {
        let line = build_line_from_styled_chars(&[]);
        // Empty input produces an empty line (Line::from("") has one empty span)
        let text: String = line.spans.iter().map(|s| s.content.as_ref()).collect();
        assert!(text.is_empty());
    }

    #[test]
    fn test_default_impl() {
        let pane = PreviewPane::default();
        assert_eq!(pane.content, "");
        assert_eq!(pane.scroll, 0);
    }
}
