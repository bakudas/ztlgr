/// GraphView widget: renders the knowledge graph using ratatui's Canvas.
///
/// Draws edges as lines, nodes as circles, and labels as text.
/// Supports pan, zoom, and node selection.
use ratatui::{
    layout::Rect,
    style::{Color, Style},
    widgets::{
        block::Title,
        canvas::{Canvas, Circle, Line},
        Block, Borders,
    },
    Frame,
};

use crate::config::Theme;
use crate::graph::{bounding_box, GraphData};

/// View state for the graph visualization.
#[derive(Debug, Clone)]
pub struct GraphState {
    /// The graph data (nodes + edges with positions)
    pub data: GraphData,
    /// Pan center X (world coordinates)
    pub view_x: f64,
    /// Pan center Y (world coordinates)
    pub view_y: f64,
    /// Zoom level (1.0 = default, higher = more zoomed in)
    pub zoom: f64,
    /// Currently selected node index
    pub selected_node: Option<usize>,
    /// Note ID that triggered graph mode (for centering)
    #[allow(dead_code)]
    pub center_note_id: Option<String>,
}

impl GraphState {
    /// Create a new GraphState centered on the given note.
    pub fn new(data: GraphData, center_note_id: Option<String>) -> Self {
        let mut state = Self {
            data,
            view_x: 0.0,
            view_y: 0.0,
            zoom: 1.0,
            selected_node: None,
            center_note_id: center_note_id.clone(),
        };
        // Center on the specified note if it exists
        if let Some(ref note_id) = center_note_id {
            if let Some(idx) = state.data.find_node_by_id(note_id) {
                state.view_x = state.data.nodes[idx].x;
                state.view_y = state.data.nodes[idx].y;
                state.selected_node = Some(idx);
            }
        }
        state
    }

    /// Pan the view by delta amounts.
    pub fn pan(&mut self, dx: f64, dy: f64) {
        let pan_speed = 5.0 / self.zoom;
        self.view_x += dx * pan_speed;
        self.view_y += dy * pan_speed;
    }

    /// Zoom in (increase zoom level).
    pub fn zoom_in(&mut self) {
        self.zoom = (self.zoom * 1.2).min(10.0);
    }

    /// Zoom out (decrease zoom level).
    pub fn zoom_out(&mut self) {
        self.zoom = (self.zoom / 1.2).max(0.1);
    }

    /// Center view on the currently selected node.
    pub fn center_on_selected(&mut self) {
        if let Some(idx) = self.selected_node {
            if idx < self.data.nodes.len() {
                self.view_x = self.data.nodes[idx].x;
                self.view_y = self.data.nodes[idx].y;
            }
        }
    }

    /// Select next node (wrapping).
    pub fn select_next(&mut self) {
        if self.data.nodes.is_empty() {
            return;
        }
        self.selected_node = Some(match self.selected_node {
            Some(idx) => (idx + 1) % self.data.nodes.len(),
            None => 0,
        });
    }

    /// Select previous node (wrapping).
    pub fn select_prev(&mut self) {
        if self.data.nodes.is_empty() {
            return;
        }
        self.selected_node = Some(match self.selected_node {
            Some(0) => self.data.nodes.len() - 1,
            Some(idx) => idx - 1,
            None => self.data.nodes.len() - 1,
        });
    }

    /// Get the selected node's note ID, if any.
    pub fn selected_note_id(&self) -> Option<&str> {
        self.selected_node
            .and_then(|idx| self.data.nodes.get(idx))
            .map(|n| n.id.as_str())
    }

    /// Fit the entire graph in view.
    pub fn fit_to_view(&mut self) {
        let (min_x, min_y, max_x, max_y) = bounding_box(&self.data);
        self.view_x = (min_x + max_x) / 2.0;
        self.view_y = (min_y + max_y) / 2.0;
        // Adjust zoom to fit — we'll refine in draw based on actual rect
        let width = (max_x - min_x).max(1.0);
        let height = (max_y - min_y).max(1.0);
        let span = width.max(height);
        // Default canvas spans ~100 units, so scale accordingly
        self.zoom = 100.0 / (span + 20.0); // 20 units margin
        self.zoom = self.zoom.clamp(0.1, 10.0);
    }
}

/// Get the color for a note type string using the theme.
fn note_type_color(note_type: &str, theme: &dyn Theme) -> Color {
    match note_type {
        "daily" => theme.note_daily(),
        "fleeting" => theme.note_fleeting(),
        "literature" => theme.note_literature(),
        "permanent" => theme.note_permanent(),
        "reference" => theme.note_reference(),
        "index" => theme.note_index(),
        _ => theme.fg(),
    }
}

/// Draw the knowledge graph onto the frame.
pub fn draw_graph(
    f: &mut Frame,
    area: Rect,
    state: &GraphState,
    theme: &dyn Theme,
    show_labels: bool,
) {
    if state.data.is_empty() {
        // Draw empty state message
        let block = Block::default()
            .borders(Borders::ALL)
            .title(Title::from(" Graph "))
            .border_style(Style::default().fg(theme.border()))
            .style(Style::default().bg(theme.bg()));
        let inner = block.inner(area);
        f.render_widget(block, area);

        // Center "No notes to display" message
        if inner.width > 20 && inner.height > 2 {
            let msg = "No notes to display. Create some notes first!";
            let x = inner.x + (inner.width.saturating_sub(msg.len() as u16)) / 2;
            let y = inner.y + inner.height / 2;
            let span = ratatui::text::Span::styled(msg, Style::default().fg(theme.fg_dim()));
            f.render_widget(span, Rect::new(x, y, msg.len() as u16, 1));
        }
        return;
    }

    // Compute view bounds based on zoom and pan
    let half_w = (area.width as f64) / (2.0 * state.zoom);
    let half_h = (area.height as f64) / (2.0 * state.zoom);
    let x_min = state.view_x - half_w;
    let x_max = state.view_x + half_w;
    let y_min = state.view_y - half_h;
    let y_max = state.view_y + half_h;

    let node_count = state.data.node_count();
    let edge_count = state.data.edge_count();

    let title = if let Some(idx) = state.selected_node {
        let name = &state.data.nodes[idx].title;
        format!(
            " Graph [{} nodes, {} edges] - Selected: {} ",
            node_count, edge_count, name
        )
    } else {
        format!(" Graph [{} nodes, {} edges] ", node_count, edge_count)
    };

    let selected_idx = state.selected_node;
    let edge_color = theme.fg_dim();
    let label_color = theme.fg();
    let selected_color = theme.accent();
    let border_color = theme.border_highlight();

    // Clone data references for the closure
    let nodes = &state.data.nodes;
    let edges = &state.data.edges;

    let canvas = Canvas::default()
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(Title::from(title))
                .border_style(Style::default().fg(border_color))
                .style(Style::default().bg(theme.bg())),
        )
        .marker(ratatui::symbols::Marker::Braille)
        .x_bounds([x_min, x_max])
        .y_bounds([y_min, y_max])
        .paint(move |ctx| {
            // Layer 1: Draw edges
            for edge in edges {
                let src = &nodes[edge.source_idx];
                let tgt = &nodes[edge.target_idx];
                ctx.draw(&Line {
                    x1: src.x,
                    y1: src.y,
                    x2: tgt.x,
                    y2: tgt.y,
                    color: edge_color,
                });
            }

            // Layer 2: Draw nodes
            for (i, node) in nodes.iter().enumerate() {
                let is_selected = selected_idx == Some(i);
                let color = if is_selected {
                    selected_color
                } else {
                    note_type_color(&node.note_type, theme)
                };

                // Node size based on degree (min 1.0, max 3.0)
                let radius = 1.0 + (node.degree() as f64).min(4.0) * 0.5;
                let radius = if is_selected { radius + 0.5 } else { radius };

                ctx.draw(&Circle {
                    x: node.x,
                    y: node.y,
                    radius,
                    color,
                });
            }

            // Layer 3: Draw labels
            if show_labels {
                for (i, node) in nodes.iter().enumerate() {
                    let is_selected = selected_idx == Some(i);
                    let color = if is_selected {
                        selected_color
                    } else {
                        label_color
                    };

                    // Truncate long titles
                    let label = if node.title.len() > 20 {
                        format!("{}...", &node.title[..17])
                    } else {
                        node.title.clone()
                    };

                    // Offset label above the node
                    ctx.print(
                        node.x,
                        node.y + 2.0,
                        ratatui::text::Span::styled(label, Style::default().fg(color)),
                    );
                }
            }
        });

    f.render_widget(canvas, area);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::types::{GraphEdge, GraphNode};

    fn make_node(id: &str, x: f64, y: f64) -> GraphNode {
        GraphNode {
            id: id.to_string(),
            title: id.to_string(),
            note_type: "permanent".to_string(),
            outgoing_count: 0,
            incoming_count: 0,
            x,
            y,
        }
    }

    fn make_graph() -> GraphData {
        GraphData {
            nodes: vec![
                make_node("a", 0.0, 0.0),
                make_node("b", 10.0, 0.0),
                make_node("c", 5.0, 10.0),
            ],
            edges: vec![GraphEdge {
                source_idx: 0,
                target_idx: 1,
                link_type: "reference".to_string(),
            }],
        }
    }

    #[test]
    fn test_graph_state_new_empty() {
        let state = GraphState::new(GraphData::empty(), None);
        assert!(state.data.is_empty());
        assert_eq!(state.view_x, 0.0);
        assert_eq!(state.view_y, 0.0);
        assert_eq!(state.zoom, 1.0);
        assert!(state.selected_node.is_none());
    }

    #[test]
    fn test_graph_state_new_with_center() {
        let graph = make_graph();
        let state = GraphState::new(graph, Some("b".to_string()));
        assert_eq!(state.view_x, 10.0); // centered on node "b"
        assert_eq!(state.view_y, 0.0);
        assert_eq!(state.selected_node, Some(1)); // "b" is index 1
    }

    #[test]
    fn test_graph_state_new_with_missing_center() {
        let graph = make_graph();
        let state = GraphState::new(graph, Some("nonexistent".to_string()));
        assert_eq!(state.view_x, 0.0); // fallback to origin
        assert!(state.selected_node.is_none());
    }

    #[test]
    fn test_graph_state_pan() {
        let mut state = GraphState::new(make_graph(), None);
        state.pan(1.0, 0.0); // pan right
        assert!(state.view_x > 0.0);
        assert_eq!(state.view_y, 0.0);

        state.pan(0.0, -1.0); // pan down
        assert!(state.view_y < 0.0);
    }

    #[test]
    fn test_graph_state_zoom_in() {
        let mut state = GraphState::new(make_graph(), None);
        let initial_zoom = state.zoom;
        state.zoom_in();
        assert!(state.zoom > initial_zoom);
    }

    #[test]
    fn test_graph_state_zoom_out() {
        let mut state = GraphState::new(make_graph(), None);
        let initial_zoom = state.zoom;
        state.zoom_out();
        assert!(state.zoom < initial_zoom);
    }

    #[test]
    fn test_graph_state_zoom_clamps_max() {
        let mut state = GraphState::new(make_graph(), None);
        for _ in 0..100 {
            state.zoom_in();
        }
        assert!(state.zoom <= 10.0);
    }

    #[test]
    fn test_graph_state_zoom_clamps_min() {
        let mut state = GraphState::new(make_graph(), None);
        for _ in 0..100 {
            state.zoom_out();
        }
        assert!(state.zoom >= 0.1);
    }

    #[test]
    fn test_graph_state_select_next() {
        let mut state = GraphState::new(make_graph(), None);
        assert!(state.selected_node.is_none());

        state.select_next();
        assert_eq!(state.selected_node, Some(0));

        state.select_next();
        assert_eq!(state.selected_node, Some(1));

        state.select_next();
        assert_eq!(state.selected_node, Some(2));

        state.select_next(); // wraps
        assert_eq!(state.selected_node, Some(0));
    }

    #[test]
    fn test_graph_state_select_prev() {
        let mut state = GraphState::new(make_graph(), None);
        state.select_prev(); // from None, goes to last
        assert_eq!(state.selected_node, Some(2));

        state.select_prev();
        assert_eq!(state.selected_node, Some(1));

        state.select_prev();
        assert_eq!(state.selected_node, Some(0));

        state.select_prev(); // wraps
        assert_eq!(state.selected_node, Some(2));
    }

    #[test]
    fn test_graph_state_select_next_empty() {
        let mut state = GraphState::new(GraphData::empty(), None);
        state.select_next();
        assert!(state.selected_node.is_none());
    }

    #[test]
    fn test_graph_state_selected_note_id() {
        let mut state = GraphState::new(make_graph(), None);
        assert!(state.selected_note_id().is_none());

        state.select_next(); // select "a"
        assert_eq!(state.selected_note_id(), Some("a"));

        state.select_next(); // select "b"
        assert_eq!(state.selected_note_id(), Some("b"));
    }

    #[test]
    fn test_graph_state_center_on_selected() {
        let mut state = GraphState::new(make_graph(), None);
        state.selected_node = Some(2); // "c" at (5.0, 10.0)
        state.center_on_selected();
        assert_eq!(state.view_x, 5.0);
        assert_eq!(state.view_y, 10.0);
    }

    #[test]
    fn test_graph_state_center_on_selected_no_selection() {
        let mut state = GraphState::new(make_graph(), None);
        state.view_x = 100.0;
        state.center_on_selected(); // no-op when nothing selected
        assert_eq!(state.view_x, 100.0);
    }

    #[test]
    fn test_graph_state_fit_to_view() {
        let mut state = GraphState::new(make_graph(), None);
        state.view_x = 999.0;
        state.view_y = -999.0;
        state.zoom = 0.01;
        state.fit_to_view();
        // Should center on midpoint and adjust zoom
        assert!(state.zoom >= 0.1);
        assert!(state.zoom <= 10.0);
        // View should be centered between nodes
        assert!(state.view_x < 20.0);
        assert!(state.view_y < 20.0);
    }

    #[test]
    fn test_note_type_color_mapping() {
        // This test just verifies the function doesn't panic for valid types
        // We can't test actual Color values without a concrete theme
        struct TestTheme;
        impl Theme for TestTheme {
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
                Color::Blue
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
                Color::Yellow
            }
            fn note_fleeting(&self) -> Color {
                Color::Green
            }
            fn note_literature(&self) -> Color {
                Color::Blue
            }
            fn note_permanent(&self) -> Color {
                Color::Cyan
            }
            fn note_reference(&self) -> Color {
                Color::Magenta
            }
            fn note_index(&self) -> Color {
                Color::Red
            }
            fn link(&self) -> Color {
                Color::Cyan
            }
            fn tag(&self) -> Color {
                Color::Yellow
            }
            fn border(&self) -> Color {
                Color::Gray
            }
            fn border_highlight(&self) -> Color {
                Color::White
            }
        }

        let theme = TestTheme;
        assert_eq!(note_type_color("daily", &theme), Color::Yellow);
        assert_eq!(note_type_color("fleeting", &theme), Color::Green);
        assert_eq!(note_type_color("literature", &theme), Color::Blue);
        assert_eq!(note_type_color("permanent", &theme), Color::Cyan);
        assert_eq!(note_type_color("reference", &theme), Color::Magenta);
        assert_eq!(note_type_color("index", &theme), Color::Red);
        assert_eq!(note_type_color("unknown", &theme), Color::White); // fallback to fg
    }
}
