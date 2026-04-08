/// Force-directed graph layout using the Fruchterman-Reingold algorithm.
///
/// This module takes a `GraphData` and iteratively adjusts node positions
/// using repulsive forces between all nodes and attractive forces along edges.
use super::types::GraphData;

/// Configuration for the force-directed layout.
#[derive(Debug, Clone)]
pub struct LayoutConfig {
    /// Number of iterations to run
    pub iterations: usize,
    /// Ideal edge length (optimal distance between connected nodes)
    pub ideal_length: f64,
    /// Cooling factor: temperature decreases each iteration
    pub cooling_factor: f64,
    /// Initial temperature (maximum displacement per iteration)
    pub initial_temperature: f64,
    /// Minimum distance to prevent division by zero
    pub min_distance: f64,
}

impl Default for LayoutConfig {
    fn default() -> Self {
        Self {
            iterations: 100,
            ideal_length: 30.0,
            cooling_factor: 0.95,
            initial_temperature: 50.0,
            min_distance: 0.1,
        }
    }
}

/// Run the Fruchterman-Reingold force-directed layout algorithm on the graph.
///
/// Modifies node positions (x, y) in place within `graph.nodes`.
pub fn force_directed_layout(graph: &mut GraphData, config: &LayoutConfig) {
    let n = graph.nodes.len();
    if n <= 1 {
        // Single node or empty graph: center it
        if n == 1 {
            graph.nodes[0].x = 0.0;
            graph.nodes[0].y = 0.0;
        }
        return;
    }

    let k = config.ideal_length;
    let k_squared = k * k;
    let mut temperature = config.initial_temperature;

    for _iter in 0..config.iterations {
        // Compute displacements
        let mut dx = vec![0.0_f64; n];
        let mut dy = vec![0.0_f64; n];

        // Repulsive forces: all pairs
        for i in 0..n {
            for j in (i + 1)..n {
                let delta_x = graph.nodes[i].x - graph.nodes[j].x;
                let delta_y = graph.nodes[i].y - graph.nodes[j].y;
                let dist = (delta_x * delta_x + delta_y * delta_y)
                    .sqrt()
                    .max(config.min_distance);

                // Repulsive force: k^2 / d
                let force = k_squared / dist;
                let fx = (delta_x / dist) * force;
                let fy = (delta_y / dist) * force;

                dx[i] += fx;
                dy[i] += fy;
                dx[j] -= fx;
                dy[j] -= fy;
            }
        }

        // Attractive forces: edges only
        for edge in &graph.edges {
            let si = edge.source_idx;
            let ti = edge.target_idx;

            let delta_x = graph.nodes[si].x - graph.nodes[ti].x;
            let delta_y = graph.nodes[si].y - graph.nodes[ti].y;
            let dist = (delta_x * delta_x + delta_y * delta_y)
                .sqrt()
                .max(config.min_distance);

            // Attractive force: d^2 / k
            let force = (dist * dist) / k;
            let fx = (delta_x / dist) * force;
            let fy = (delta_y / dist) * force;

            dx[si] -= fx;
            dy[si] -= fy;
            dx[ti] += fx;
            dy[ti] += fy;
        }

        // Apply displacements, limited by temperature
        for i in 0..n {
            let disp = (dx[i] * dx[i] + dy[i] * dy[i])
                .sqrt()
                .max(config.min_distance);
            let scale = temperature.min(disp) / disp;
            graph.nodes[i].x += dx[i] * scale;
            graph.nodes[i].y += dy[i] * scale;
        }

        // Cool down
        temperature *= config.cooling_factor;
    }

    // Center the graph around origin
    center_graph(graph);
}

/// Center the graph so the centroid is at (0, 0).
fn center_graph(graph: &mut GraphData) {
    if graph.nodes.is_empty() {
        return;
    }
    let n = graph.nodes.len() as f64;
    let cx: f64 = graph.nodes.iter().map(|node| node.x).sum::<f64>() / n;
    let cy: f64 = graph.nodes.iter().map(|node| node.y).sum::<f64>() / n;
    for node in &mut graph.nodes {
        node.x -= cx;
        node.y -= cy;
    }
}

/// Compute the bounding box of all node positions.
///
/// Returns (min_x, min_y, max_x, max_y).
pub fn bounding_box(graph: &GraphData) -> (f64, f64, f64, f64) {
    if graph.nodes.is_empty() {
        return (0.0, 0.0, 0.0, 0.0);
    }
    let mut min_x = f64::MAX;
    let mut min_y = f64::MAX;
    let mut max_x = f64::MIN;
    let mut max_y = f64::MIN;
    for node in &graph.nodes {
        if node.x < min_x {
            min_x = node.x;
        }
        if node.y < min_y {
            min_y = node.y;
        }
        if node.x > max_x {
            max_x = node.x;
        }
        if node.y > max_y {
            max_y = node.y;
        }
    }
    (min_x, min_y, max_x, max_y)
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

    #[test]
    fn test_layout_empty_graph() {
        let mut graph = GraphData::empty();
        let config = LayoutConfig::default();
        force_directed_layout(&mut graph, &config);
        assert!(graph.nodes.is_empty());
    }

    #[test]
    fn test_layout_single_node_centers() {
        let mut graph = GraphData {
            nodes: vec![make_node("a", 100.0, -50.0)],
            edges: vec![],
        };
        let config = LayoutConfig::default();
        force_directed_layout(&mut graph, &config);
        assert_eq!(graph.nodes[0].x, 0.0);
        assert_eq!(graph.nodes[0].y, 0.0);
    }

    #[test]
    fn test_layout_two_disconnected_nodes_repel() {
        let mut graph = GraphData {
            nodes: vec![make_node("a", 0.0, 0.0), make_node("b", 1.0, 0.0)],
            edges: vec![],
        };
        let config = LayoutConfig {
            iterations: 50,
            ..Default::default()
        };
        force_directed_layout(&mut graph, &config);
        // They should be farther apart due to repulsion
        let dist = ((graph.nodes[0].x - graph.nodes[1].x).powi(2)
            + (graph.nodes[0].y - graph.nodes[1].y).powi(2))
        .sqrt();
        assert!(dist > 1.0, "Disconnected nodes should repel: dist={}", dist);
    }

    #[test]
    fn test_layout_connected_nodes_reach_equilibrium() {
        let mut graph = GraphData {
            nodes: vec![make_node("a", -100.0, 0.0), make_node("b", 100.0, 0.0)],
            edges: vec![GraphEdge {
                source_idx: 0,
                target_idx: 1,
                link_type: "reference".to_string(),
            }],
        };
        let config = LayoutConfig {
            iterations: 200,
            ..Default::default()
        };
        force_directed_layout(&mut graph, &config);
        // Connected nodes should converge to a reasonable distance
        let dist = ((graph.nodes[0].x - graph.nodes[1].x).powi(2)
            + (graph.nodes[0].y - graph.nodes[1].y).powi(2))
        .sqrt();
        // Should be near the ideal length (30.0), within a reasonable margin
        assert!(
            dist < 200.0,
            "Connected nodes should not be extremely far: dist={}",
            dist
        );
        assert!(
            dist > 1.0,
            "Connected nodes should not overlap: dist={}",
            dist
        );
    }

    #[test]
    fn test_layout_graph_is_centered() {
        let mut graph = GraphData {
            nodes: vec![
                make_node("a", 100.0, 100.0),
                make_node("b", 200.0, 200.0),
                make_node("c", 300.0, 300.0),
            ],
            edges: vec![],
        };
        let config = LayoutConfig::default();
        force_directed_layout(&mut graph, &config);
        let cx: f64 = graph.nodes.iter().map(|n| n.x).sum::<f64>() / 3.0;
        let cy: f64 = graph.nodes.iter().map(|n| n.y).sum::<f64>() / 3.0;
        assert!(cx.abs() < 0.01, "Center X should be ~0: {}", cx);
        assert!(cy.abs() < 0.01, "Center Y should be ~0: {}", cy);
    }

    #[test]
    fn test_layout_triangle_graph() {
        let mut graph = GraphData {
            nodes: vec![
                make_node("a", 0.0, 0.0),
                make_node("b", 10.0, 0.0),
                make_node("c", 5.0, 10.0),
            ],
            edges: vec![
                GraphEdge {
                    source_idx: 0,
                    target_idx: 1,
                    link_type: "reference".to_string(),
                },
                GraphEdge {
                    source_idx: 1,
                    target_idx: 2,
                    link_type: "reference".to_string(),
                },
                GraphEdge {
                    source_idx: 2,
                    target_idx: 0,
                    link_type: "reference".to_string(),
                },
            ],
        };
        let config = LayoutConfig {
            iterations: 200,
            ..Default::default()
        };
        force_directed_layout(&mut graph, &config);
        // All three nodes should be distinct positions
        for i in 0..3 {
            for j in (i + 1)..3 {
                let dist = ((graph.nodes[i].x - graph.nodes[j].x).powi(2)
                    + (graph.nodes[i].y - graph.nodes[j].y).powi(2))
                .sqrt();
                assert!(dist > 1.0, "Nodes {} and {} overlap: dist={}", i, j, dist);
            }
        }
    }

    #[test]
    fn test_bounding_box_empty() {
        let graph = GraphData::empty();
        let (min_x, min_y, max_x, max_y) = bounding_box(&graph);
        assert_eq!(min_x, 0.0);
        assert_eq!(min_y, 0.0);
        assert_eq!(max_x, 0.0);
        assert_eq!(max_y, 0.0);
    }

    #[test]
    fn test_bounding_box_single_node() {
        let graph = GraphData {
            nodes: vec![make_node("a", 5.0, -3.0)],
            edges: vec![],
        };
        let (min_x, min_y, max_x, max_y) = bounding_box(&graph);
        assert_eq!(min_x, 5.0);
        assert_eq!(min_y, -3.0);
        assert_eq!(max_x, 5.0);
        assert_eq!(max_y, -3.0);
    }

    #[test]
    fn test_bounding_box_multiple_nodes() {
        let graph = GraphData {
            nodes: vec![
                make_node("a", -10.0, 5.0),
                make_node("b", 20.0, -15.0),
                make_node("c", 0.0, 30.0),
            ],
            edges: vec![],
        };
        let (min_x, min_y, max_x, max_y) = bounding_box(&graph);
        assert_eq!(min_x, -10.0);
        assert_eq!(min_y, -15.0);
        assert_eq!(max_x, 20.0);
        assert_eq!(max_y, 30.0);
    }

    #[test]
    fn test_center_graph_shifts_centroid() {
        let mut graph = GraphData {
            nodes: vec![make_node("a", 10.0, 20.0), make_node("b", 30.0, 40.0)],
            edges: vec![],
        };
        center_graph(&mut graph);
        let cx: f64 = graph.nodes.iter().map(|n| n.x).sum::<f64>() / 2.0;
        let cy: f64 = graph.nodes.iter().map(|n| n.y).sum::<f64>() / 2.0;
        assert!(cx.abs() < 0.01);
        assert!(cy.abs() < 0.01);
    }

    #[test]
    fn test_layout_config_default() {
        let config = LayoutConfig::default();
        assert_eq!(config.iterations, 100);
        assert_eq!(config.ideal_length, 30.0);
        assert!((config.cooling_factor - 0.95).abs() < 0.001);
        assert_eq!(config.initial_temperature, 50.0);
    }

    #[test]
    fn test_layout_preserves_node_metadata() {
        let mut graph = GraphData {
            nodes: vec![GraphNode {
                id: "test-id".to_string(),
                title: "My Title".to_string(),
                note_type: "literature".to_string(),
                outgoing_count: 5,
                incoming_count: 3,
                x: 10.0,
                y: 20.0,
            }],
            edges: vec![],
        };
        let config = LayoutConfig::default();
        force_directed_layout(&mut graph, &config);
        assert_eq!(graph.nodes[0].id, "test-id");
        assert_eq!(graph.nodes[0].title, "My Title");
        assert_eq!(graph.nodes[0].note_type, "literature");
        assert_eq!(graph.nodes[0].outgoing_count, 5);
        assert_eq!(graph.nodes[0].incoming_count, 3);
    }
}
