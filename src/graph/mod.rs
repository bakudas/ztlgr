/// Knowledge graph visualization module.
///
/// Provides data types, force-directed layout, and rendering support
/// for visualizing the Zettelkasten note graph.
pub mod layout;
pub mod types;

pub use layout::{bounding_box, force_directed_layout, LayoutConfig};
pub use types::{GraphData, GraphEdge, GraphNode};
