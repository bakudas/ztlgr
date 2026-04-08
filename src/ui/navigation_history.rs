use std::collections::VecDeque;

/// Represents a navigation point in the link history
#[derive(Debug, Clone)]
pub struct NavigationPoint {
    pub note_id: String,
    pub note_title: String,
    /// Cursor position in the note when navigated away
    pub cursor_pos: usize,
}

/// Navigation history for link following (Enter to follow, Ctrl+O to go back)
pub struct NavigationHistory {
    /// Stack of previously visited notes
    history: VecDeque<NavigationPoint>,
    /// Maximum number of steps to keep in history
    max_size: usize,
}

impl NavigationHistory {
    pub fn new(max_size: usize) -> Self {
        Self {
            history: VecDeque::with_capacity(max_size),
            max_size,
        }
    }

    /// Push a new point into history (when navigating forward)
    pub fn push(&mut self, point: NavigationPoint) {
        self.history.push_front(point);
        if self.history.len() > self.max_size {
            self.history.pop_back();
        }
    }

    /// Pop the last visited note (go back)
    pub fn pop(&mut self) -> Option<NavigationPoint> {
        self.history.pop_front()
    }

    /// Get the most recent navigation point without removing it
    pub fn peek(&self) -> Option<&NavigationPoint> {
        self.history.front()
    }

    /// Check if there's history to go back to
    pub fn has_history(&self) -> bool {
        !self.history.is_empty()
    }

    /// Clear all history
    pub fn clear(&mut self) {
        self.history.clear();
    }

    /// Get current history size
    pub fn len(&self) -> usize {
        self.history.len()
    }

    /// Check if history is empty
    pub fn is_empty(&self) -> bool {
        self.history.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_navigation_history_new() {
        let history = NavigationHistory::new(10);
        assert_eq!(history.len(), 0);
        assert!(!history.has_history());
    }

    #[test]
    fn test_navigation_history_push() {
        let mut history = NavigationHistory::new(10);
        history.push(NavigationPoint {
            note_id: "note-1".to_string(),
            note_title: "Note 1".to_string(),
            cursor_pos: 42,
        });
        assert_eq!(history.len(), 1);
        assert!(history.has_history());
    }

    #[test]
    fn test_navigation_history_pop() {
        let mut history = NavigationHistory::new(10);
        history.push(NavigationPoint {
            note_id: "note-1".to_string(),
            note_title: "Note 1".to_string(),
            cursor_pos: 42,
        });

        let popped = history.pop();
        assert!(popped.is_some());
        assert_eq!(popped.unwrap().note_id, "note-1");
        assert!(!history.has_history());
    }

    #[test]
    fn test_navigation_history_peek() {
        let mut history = NavigationHistory::new(10);
        history.push(NavigationPoint {
            note_id: "note-1".to_string(),
            note_title: "Note 1".to_string(),
            cursor_pos: 42,
        });

        let peeked = history.peek();
        assert!(peeked.is_some());
        assert_eq!(peeked.unwrap().note_id, "note-1");
        // After peek, history should still have the item
        assert!(history.has_history());
    }

    #[test]
    fn test_navigation_history_max_size() {
        let mut history = NavigationHistory::new(2);

        for i in 1..=5 {
            history.push(NavigationPoint {
                note_id: format!("note-{}", i),
                note_title: format!("Note {}", i),
                cursor_pos: i * 10,
            });
        }

        // History should not exceed max_size
        assert_eq!(history.len(), 2);

        // Should keep the most recent entries
        let point = history.pop().unwrap();
        assert_eq!(point.note_id, "note-5");
    }

    #[test]
    fn test_navigation_history_clear() {
        let mut history = NavigationHistory::new(10);
        history.push(NavigationPoint {
            note_id: "note-1".to_string(),
            note_title: "Note 1".to_string(),
            cursor_pos: 42,
        });

        history.clear();
        assert!(!history.has_history());
        assert_eq!(history.len(), 0);
    }

    #[test]
    fn test_navigation_history_lifo_order() {
        let mut history = NavigationHistory::new(10);

        for i in 1..=3 {
            history.push(NavigationPoint {
                note_id: format!("note-{}", i),
                note_title: format!("Note {}", i),
                cursor_pos: i * 10,
            });
        }

        // Should pop in reverse order (LIFO)
        assert_eq!(history.pop().unwrap().note_id, "note-3");
        assert_eq!(history.pop().unwrap().note_id, "note-2");
        assert_eq!(history.pop().unwrap().note_id, "note-1");
    }
}
