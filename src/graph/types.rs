//! Graph data types for knowledge graph visualization.

/// A node in the knowledge graph, representing a note.
#[derive(Debug, Clone)]
pub struct GraphNode {
    /// Unique note ID
    pub id: String,
    /// Note title (displayed as label)
    pub title: String,
    /// Note type (permanent, fleeting, literature, daily)
    pub note_type: String,
    /// Number of outgoing links from this note
    pub outgoing_count: usize,
    /// Number of incoming links to this note
    pub incoming_count: usize,
    /// Current X position in the layout (world coordinates)
    pub x: f64,
    /// Current Y position in the layout (world coordinates)
    pub y: f64,
}

impl GraphNode {
    /// Total number of connections (outgoing + incoming).
    pub fn degree(&self) -> usize {
        self.outgoing_count + self.incoming_count
    }
}

/// An edge in the knowledge graph, representing a link between notes.
#[derive(Debug, Clone)]
pub struct GraphEdge {
    /// Index into GraphData.nodes for the source
    pub source_idx: usize,
    /// Index into GraphData.nodes for the target
    pub target_idx: usize,
    /// Link type (reference, embed, etc.)
    pub link_type: String,
}

/// The full graph dataset: nodes + edges, ready for layout and rendering.
#[derive(Debug, Clone)]
pub struct GraphData {
    pub nodes: Vec<GraphNode>,
    pub edges: Vec<GraphEdge>,
}

impl GraphData {
    /// Create an empty graph.
    pub fn empty() -> Self {
        Self {
            nodes: Vec::new(),
            edges: Vec::new(),
        }
    }

    /// Build a GraphData from raw database results.
    ///
    /// `raw_nodes`: Vec of (note_id, title, note_type, outgoing_count, incoming_count)
    /// `raw_edges`: Vec of (source_id, source_title, target_id, target_title, link_type)
    pub fn from_db(
        raw_nodes: Vec<(String, String, String, usize, usize)>,
        raw_edges: Vec<(String, String, String, String, String)>,
    ) -> Self {
        use std::collections::HashMap;

        if raw_nodes.is_empty() {
            return Self::empty();
        }

        // Build nodes with initial random-ish positions based on index
        let mut id_to_idx: HashMap<String, usize> = HashMap::new();
        let node_count = raw_nodes.len();
        let nodes: Vec<GraphNode> = raw_nodes
            .into_iter()
            .enumerate()
            .map(|(i, (id, title, note_type, out_count, in_count))| {
                id_to_idx.insert(id.clone(), i);
                // Distribute nodes in a circle initially
                let angle = 2.0 * std::f64::consts::PI * (i as f64) / (node_count as f64);
                let radius = 50.0;
                GraphNode {
                    id,
                    title,
                    note_type,
                    outgoing_count: out_count,
                    incoming_count: in_count,
                    x: radius * angle.cos(),
                    y: radius * angle.sin(),
                }
            })
            .collect();

        // Build edges, skipping any with missing node IDs
        let edges: Vec<GraphEdge> = raw_edges
            .into_iter()
            .filter_map(
                |(source_id, _source_title, target_id, _target_title, link_type)| {
                    let source_idx = id_to_idx.get(&source_id)?;
                    let target_idx = id_to_idx.get(&target_id)?;
                    Some(GraphEdge {
                        source_idx: *source_idx,
                        target_idx: *target_idx,
                        link_type,
                    })
                },
            )
            .collect();

        Self { nodes, edges }
    }

    /// Returns true if the graph has no nodes.
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    /// Returns the number of nodes.
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    /// Returns the number of edges.
    pub fn edge_count(&self) -> usize {
        self.edges.len()
    }

    /// Find a node index by note ID.
    pub fn find_node_by_id(&self, note_id: &str) -> Option<usize> {
        self.nodes.iter().position(|n| n.id == note_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_graph_node_degree() {
        let node = GraphNode {
            id: "abc".to_string(),
            title: "Test".to_string(),
            note_type: "permanent".to_string(),
            outgoing_count: 3,
            incoming_count: 2,
            x: 0.0,
            y: 0.0,
        };
        assert_eq!(node.degree(), 5);
    }

    #[test]
    fn test_graph_node_degree_zero() {
        let node = GraphNode {
            id: "abc".to_string(),
            title: "Orphan".to_string(),
            note_type: "fleeting".to_string(),
            outgoing_count: 0,
            incoming_count: 0,
            x: 0.0,
            y: 0.0,
        };
        assert_eq!(node.degree(), 0);
    }

    #[test]
    fn test_graph_data_empty() {
        let graph = GraphData::empty();
        assert!(graph.is_empty());
        assert_eq!(graph.node_count(), 0);
        assert_eq!(graph.edge_count(), 0);
    }

    #[test]
    fn test_graph_data_from_db_empty() {
        let graph = GraphData::from_db(vec![], vec![]);
        assert!(graph.is_empty());
    }

    #[test]
    fn test_graph_data_from_db_nodes_only() {
        let raw_nodes = vec![
            ("id1".into(), "Note One".into(), "permanent".into(), 0, 0),
            ("id2".into(), "Note Two".into(), "fleeting".into(), 0, 0),
        ];
        let graph = GraphData::from_db(raw_nodes, vec![]);
        assert_eq!(graph.node_count(), 2);
        assert_eq!(graph.edge_count(), 0);
        assert_eq!(graph.nodes[0].title, "Note One");
        assert_eq!(graph.nodes[1].title, "Note Two");
    }

    #[test]
    fn test_graph_data_from_db_with_edges() {
        let raw_nodes = vec![
            ("id1".into(), "Alpha".into(), "permanent".into(), 1, 0),
            ("id2".into(), "Beta".into(), "permanent".into(), 0, 1),
        ];
        let raw_edges = vec![(
            "id1".into(),
            "Alpha".into(),
            "id2".into(),
            "Beta".into(),
            "reference".into(),
        )];
        let graph = GraphData::from_db(raw_nodes, raw_edges);
        assert_eq!(graph.node_count(), 2);
        assert_eq!(graph.edge_count(), 1);
        assert_eq!(graph.edges[0].source_idx, 0);
        assert_eq!(graph.edges[0].target_idx, 1);
        assert_eq!(graph.edges[0].link_type, "reference");
    }

    #[test]
    fn test_graph_data_from_db_skips_invalid_edges() {
        let raw_nodes = vec![("id1".into(), "Only".into(), "permanent".into(), 0, 0)];
        // Edge references id2 which doesn't exist in nodes
        let raw_edges = vec![(
            "id1".into(),
            "Only".into(),
            "id_nonexistent".into(),
            "Ghost".into(),
            "reference".into(),
        )];
        let graph = GraphData::from_db(raw_nodes, raw_edges);
        assert_eq!(graph.node_count(), 1);
        assert_eq!(graph.edge_count(), 0); // Edge skipped
    }

    #[test]
    fn test_graph_data_initial_positions_circular() {
        let raw_nodes = vec![
            ("id1".into(), "A".into(), "permanent".into(), 0, 0),
            ("id2".into(), "B".into(), "permanent".into(), 0, 0),
            ("id3".into(), "C".into(), "permanent".into(), 0, 0),
            ("id4".into(), "D".into(), "permanent".into(), 0, 0),
        ];
        let graph = GraphData::from_db(raw_nodes, vec![]);
        // First node should be at angle 0 → (50, 0)
        assert!((graph.nodes[0].x - 50.0).abs() < 0.01);
        assert!((graph.nodes[0].y - 0.0).abs() < 0.01);
        // All nodes should be at radius ~50
        for node in &graph.nodes {
            let r = (node.x * node.x + node.y * node.y).sqrt();
            assert!((r - 50.0).abs() < 0.01);
        }
    }

    #[test]
    fn test_graph_data_find_node_by_id() {
        let raw_nodes = vec![
            ("id1".into(), "A".into(), "permanent".into(), 0, 0),
            ("id2".into(), "B".into(), "permanent".into(), 0, 0),
        ];
        let graph = GraphData::from_db(raw_nodes, vec![]);
        assert_eq!(graph.find_node_by_id("id1"), Some(0));
        assert_eq!(graph.find_node_by_id("id2"), Some(1));
        assert_eq!(graph.find_node_by_id("id_missing"), None);
    }

    #[test]
    fn test_graph_data_from_db_preserves_types() {
        let raw_nodes = vec![
            ("id1".into(), "Daily".into(), "daily".into(), 2, 3),
            ("id2".into(), "Literature".into(), "literature".into(), 0, 1),
        ];
        let graph = GraphData::from_db(raw_nodes, vec![]);
        assert_eq!(graph.nodes[0].note_type, "daily");
        assert_eq!(graph.nodes[0].outgoing_count, 2);
        assert_eq!(graph.nodes[0].incoming_count, 3);
        assert_eq!(graph.nodes[1].note_type, "literature");
    }
}
