use chrono::{DateTime, Utc};
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

use crate::note::{Note, NoteType};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MetadataField {
    Title,
    Type,
    Tags,
    CreatedAt,
    UpdatedAt,
}

pub struct MetadataPane {
    current_note: Option<Note>,
    selected_field: Option<MetadataField>,
    editing: bool,
}

impl MetadataPane {
    pub fn new() -> Self {
        Self {
            current_note: None,
            selected_field: None,
            editing: false,
        }
    }

    pub fn set_note(&mut self, note: Note) {
        self.current_note = Some(note);
    }

    pub fn clear(&mut self) {
        self.current_note = None;
        self.selected_field = None;
        self.editing = false;
    }

    pub fn is_editing(&self) -> bool {
        self.editing
    }

    pub fn select_next(&mut self) {
        let fields = [
            MetadataField::Title,
            MetadataField::Type,
            MetadataField::Tags,
        ];
        if let Some(current) = &self.selected_field {
            if let Some(pos) = fields.iter().position(|f| f == current) {
                if pos + 1 < fields.len() {
                    self.selected_field = Some(fields[pos + 1]);
                }
            } else {
                self.selected_field = Some(fields[0]);
            }
        } else {
            self.selected_field = Some(fields[0]);
        }
    }

    pub fn select_prev(&mut self) {
        let fields = [
            MetadataField::Title,
            MetadataField::Type,
            MetadataField::Tags,
        ];
        if let Some(current) = &self.selected_field {
            if let Some(pos) = fields.iter().position(|f| f == current) {
                if pos > 0 {
                    self.selected_field = Some(fields[pos - 1]);
                } else {
                    self.selected_field = Some(fields[fields.len() - 1]);
                }
            } else {
                self.selected_field = Some(fields[0]);
            }
        } else {
            self.selected_field = Some(fields[0]);
        }
    }

    pub fn toggle_edit(&mut self) {
        if self.selected_field.is_some() {
            self.editing = !self.editing;
        }
    }

    pub fn get_selected_field(&self) -> Option<MetadataField> {
        self.selected_field
    }

    fn format_datetime(dt: &DateTime<Utc>) -> String {
        dt.format("%Y-%m-%d %H:%M").to_string()
    }

    fn format_note_type(note_type: &NoteType) -> String {
        match note_type {
            NoteType::Daily => "Daily".to_string(),
            NoteType::Fleeting => "Fleeting".to_string(),
            NoteType::Literature { source } => {
                if source.is_empty() {
                    "Literature".to_string()
                } else {
                    format!("Literature ({})", source)
                }
            }
            NoteType::Permanent => "Permanent".to_string(),
            NoteType::Reference { url } => {
                if let Some(url) = url {
                    if url.is_empty() {
                        "Reference".to_string()
                    } else {
                        format!("Reference ({})", url)
                    }
                } else {
                    "Reference".to_string()
                }
            }
            NoteType::Index => "Index".to_string(),
        }
    }

    fn field_to_string(&self, field: MetadataField) -> String {
        if let Some(note) = &self.current_note {
            match field {
                MetadataField::Title => note.title.clone(),
                MetadataField::Type => Self::format_note_type(&note.note_type),
                MetadataField::Tags => note
                    .metadata
                    .tags
                    .as_ref()
                    .map(|t| t.join(", "))
                    .unwrap_or_default(),
                MetadataField::CreatedAt => Self::format_datetime(&note.created_at),
                MetadataField::UpdatedAt => Self::format_datetime(&note.updated_at),
            }
        } else {
            String::new()
        }
    }

    fn render_field(&self, label: &str, field: MetadataField) -> Line {
        let is_selected = self.selected_field == Some(field);
        let is_editable = matches!(
            field,
            MetadataField::Title | MetadataField::Type | MetadataField::Tags
        );

        let label_style = if is_selected {
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Gray)
        };

        let value_style = if is_selected && self.editing {
            Style::default().fg(Color::Green).bg(Color::DarkGray)
        } else if is_selected {
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::White)
        };

        let edit_indicator = if is_editable && is_selected {
            if self.editing {
                " [EDITING]"
            } else {
                " [Enter to edit]"
            }
        } else {
            ""
        };

        Line::from(vec![
            Span::styled(format!("{}: ", label), label_style),
            Span::styled(self.field_to_string(field), value_style),
            Span::styled(edit_indicator, Style::default().fg(Color::Cyan)),
        ])
    }

    pub fn draw(&self, f: &mut Frame, area: Rect, theme: &dyn crate::config::Theme) {
        let mut lines = vec![];

        if let Some(note) = &self.current_note {
            // Title
            lines.push(self.render_field("Title", MetadataField::Title));
            lines.push(Line::from(""));

            // Type
            lines.push(self.render_field("Type", MetadataField::Type));
            lines.push(Line::from(""));

            // Tags
            lines.push(self.render_field("Tags", MetadataField::Tags));
            lines.push(Line::from(""));

            // Read-only metadata
            lines.push(self.render_field("Created", MetadataField::CreatedAt));
            lines.push(self.render_field("Updated", MetadataField::UpdatedAt));
            lines.push(Line::from(""));

            // Note ID
            lines.push(Line::from(vec![
                Span::styled("ID: ", Style::default().fg(Color::Gray)),
                Span::styled(note.id.as_str(), Style::default().fg(Color::DarkGray)),
            ]));

            // Zettel ID (if present)
            if let Some(zid) = &note.zettel_id {
                lines.push(Line::from(vec![
                    Span::styled("Zettel ID: ", Style::default().fg(Color::Gray)),
                    Span::styled(zid.as_str(), Style::default().fg(Color::DarkGray)),
                ]));
            }

            // Parent ID (if present)
            if let Some(parent_id) = &note.parent_id {
                lines.push(Line::from(vec![
                    Span::styled("Parent: ", Style::default().fg(Color::Gray)),
                    Span::styled(parent_id.as_str(), Style::default().fg(Color::DarkGray)),
                ]));
            }

            // Source (for Literature notes)
            if let NoteType::Literature { source } = &note.note_type {
                if !source.is_empty() {
                    lines.push(Line::from(""));
                    lines.push(Line::from(vec![
                        Span::styled("Source: ", Style::default().fg(Color::Gray)),
                        Span::styled(source, Style::default().fg(Color::Cyan)),
                    ]));
                }
            }

            // URL (for Reference notes)
            if let NoteType::Reference { url } = &note.note_type {
                if let Some(url) = url {
                    if !url.is_empty() {
                        lines.push(Line::from(""));
                        lines.push(Line::from(vec![
                            Span::styled("URL: ", Style::default().fg(Color::Gray)),
                            Span::styled(url, Style::default().fg(Color::Cyan)),
                        ]));
                    }
                }
            }

            // Aliases (if present)
            if let Some(aliases) = &note.metadata.aliases {
                if !aliases.is_empty() {
                    lines.push(Line::from(""));
                    lines.push(Line::from(vec![
                        Span::styled("Aliases: ", Style::default().fg(Color::Gray)),
                        Span::styled(aliases.join(", "), Style::default().fg(Color::White)),
                    ]));
                }
            }

            // Custom metadata
            if !note.metadata.custom.is_empty() {
                lines.push(Line::from(""));
                lines.push(Line::from(vec![Span::styled(
                    "Custom Fields:",
                    Style::default().fg(Color::Gray),
                )]));
                for (key, value) in &note.metadata.custom {
                    lines.push(Line::from(vec![
                        Span::styled(format!("  {}: ", key), Style::default().fg(Color::Gray)),
                        Span::styled(
                            serde_json::to_string(value).unwrap_or_default(),
                            Style::default().fg(Color::White),
                        ),
                    ]));
                }
            }
        } else {
            lines.push(Line::from("Select a note to view metadata"));
        }

        // Help text
        if self.current_note.is_some() {
            lines.push(Line::from(""));
            lines.push(Line::from(vec![Span::styled(
                "j/k: Navigate  |  Enter: Edit  |  Esc: Cancel",
                Style::default().fg(Color::DarkGray),
            )]));
        }

        let paragraph = Paragraph::new(lines)
            .block(
                Block::default()
                    .title(" Metadata ")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(theme.border())),
            )
            .style(Style::default().fg(theme.fg()).bg(theme.bg()))
            .wrap(Wrap { trim: true });

        f.render_widget(paragraph, area);
    }
}
