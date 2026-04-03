use crate::note::{Note, NoteType};
use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, List},
    Frame,
};

pub struct NoteList {
    scroll: usize,
}

impl NoteList {
    pub fn new() -> Self {
        Self { scroll: 0 }
    }

    pub fn draw(
        &self,
        f: &mut Frame,
        area: Rect,
        notes: &[Note],
        theme: &dyn crate::config::Theme,
        selected: Option<&str>,
        is_focused: bool,
    ) {
        // Group notes by type
        let mut items: Vec<ratatui::widgets::ListItem> = Vec::new();

        // Define order of note types
        let type_order = [
            (NoteType::Daily, "Daily", "󰃰"),
            (NoteType::Fleeting, "Inbox", "󰘶"),
            (NoteType::Permanent, "Permanent", "󰐕"),
            (
                NoteType::Literature {
                    source: String::new(),
                },
                "Literature",
                "󰧮",
            ),
            (NoteType::Reference { url: None }, "Reference", "󰈙"),
            (NoteType::Index, "Index", "󰈈"),
        ];

        let mut current_y = 0;

        for (note_type, type_name, icon) in type_order.iter() {
            // Filter notes of this type
            let notes_of_type: Vec<&Note> = notes
                .iter()
                .filter(|n| match (&n.note_type, note_type) {
                    (NoteType::Daily, NoteType::Daily) => true,
                    (NoteType::Fleeting, NoteType::Fleeting) => true,
                    (NoteType::Permanent, NoteType::Permanent) => true,
                    (NoteType::Literature { .. }, NoteType::Literature { .. }) => true,
                    (NoteType::Reference { .. }, NoteType::Reference { .. }) => true,
                    (NoteType::Index, NoteType::Index) => true,
                    _ => false,
                })
                .collect();

            if notes_of_type.is_empty() {
                continue;
            }

            // Add type header
            let header_style = Style::default()
                .fg(theme.accent())
                .add_modifier(Modifier::BOLD);

            let header_text =
                Text::from(format!("{} {}", icon, type_name)).patch_style(header_style);

            items.push(ratatui::widgets::ListItem::new(header_text));
            current_y += 1;

            // Add notes of this type
            for note in notes_of_type {
                let is_selected = Some(note.id.as_str()) == selected;

                let style = if is_selected {
                    theme.selected_style()
                } else {
                    Style::default().fg(theme.fg()).bg(theme.bg())
                };

                let zettel_id = note
                    .zettel_id
                    .as_ref()
                    .map(|z| format!("[{}] ", z.as_str()))
                    .unwrap_or_default();

                let prefix = if is_selected { "▶ " } else { "  " };
                let text =
                    Text::from(format!("{}{}{}", prefix, zettel_id, note.title)).patch_style(style);

                items.push(ratatui::widgets::ListItem::new(text));
                current_y += 1;
            }
        }

        let border_color = if is_focused {
            theme.border_highlight()
        } else {
            theme.border()
        };

        let list = List::new(items).block(
            Block::default()
                .title(" Notes ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(border_color)),
        );

        f.render_widget(list, area);
    }
}
