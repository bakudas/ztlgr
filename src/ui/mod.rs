mod app;
pub mod link_following;
pub mod navigation_history;
mod widgets;

pub use app::App;
pub use link_following::detect_link_at_cursor;
pub use navigation_history::{NavigationHistory, NavigationPoint};
pub use widgets::{NoteEditor, NoteList, PreviewPane, StatusBar};
