use crate::note::Note;
use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::Text,
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
        // Render notes with type headers
        let mut items: Vec<ratatui::widgets::ListItem> = Vec::new();
        let mut last_type: Option<&str> = None;

        for note in notes {
            // Get type name and icon
            let (type_name, type_icon) = match &note.note_type {
                crate::note::NoteType::Daily => ("Daily", "󰃰"),
                crate::note::NoteType::Fleeting => ("Inbox", "󰘶"),
                crate::note::NoteType::Permanent => ("Permanent", "󰐕"),
                crate::note::NoteType::Literature { .. } => ("Literature", "󰧮"),
                crate::note::NoteType::Reference { .. } => ("Reference", "󰈙"),
                crate::note::NoteType::Index => ("Index", "󰈈"),
            };

            // Add type header if this is a new type
            if last_type != Some(type_name) {
                let header_style = Style::default()
                    .fg(theme.accent())
                    .add_modifier(Modifier::BOLD);

                let header_text =
                    Text::from(format!("{} {}", type_icon, type_name)).patch_style(header_style);

                items.push(ratatui::widgets::ListItem::new(header_text));
                last_type = Some(type_name);
            }

            // Add the note
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
