use crate::note::Note;
use ratatui::{
    layout::Rect,
    style::Style,
    text::Text,
    widgets::{Block, Borders},
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
    ) {
        use ratatui::widgets::List;

        let items: Vec<ratatui::widgets::ListItem> = notes
            .iter()
            .map(|note| {
                let is_selected = Some(note.id.as_str()) == selected;

                let style = if is_selected {
                    theme.selected_style()
                } else {
                    Style::default().fg(theme.fg()).bg(theme.bg())
                };

                let note_style = Style::default().fg(theme.note_color(&note.note_type));

                let zettel_id = note
                    .zettel_id
                    .as_ref()
                    .map(|z| format!("[{}] ", z.as_str()))
                    .unwrap_or_default();

                let text =
                    Text::from(format!("{}{}", zettel_id, note.title)).patch_style(note_style);

                ratatui::widgets::ListItem::new(text).style(style)
            })
            .collect();

        let list = List::new(items).block(
            Block::default()
                .title(" Notes ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.border())),
        );

        f.render_widget(list, area);
    }
}
