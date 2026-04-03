/// Representa uma ação editável (para undo/redo)
#[derive(Clone, Debug)]
pub enum EditAction {
    /// Inserir texto na posição
    Insert { pos: usize, text: String },
    /// Deletar texto no range
    Delete {
        pos: usize,
        text: String,
        len: usize,
    },
    /// Modificar seleção
    SelectionChange { start: usize, end: usize },
}

impl EditAction {
    /// Inverte a ação (para undo/redo)
    pub fn inverse(&self) -> Self {
        match self {
            EditAction::Insert { pos, text } => EditAction::Delete {
                pos: *pos,
                text: text.clone(),
                len: text.len(),
            },
            EditAction::Delete { pos, text, len: _ } => EditAction::Insert {
                pos: *pos,
                text: text.clone(),
            },
            EditAction::SelectionChange { start, end } => EditAction::SelectionChange {
                start: *end,
                end: *start,
            },
        }
    }
}

/// Gerencia histórico de ações (undo/redo)
#[derive(Debug)]
pub struct EditHistory {
    /// Stack de ações (para undo)
    undo_stack: Vec<EditAction>,
    /// Stack de ações desfeitas (para redo)
    redo_stack: Vec<EditAction>,
    /// Limite máximo de ações no histórico
    max_history: usize,
    /// Flag para saber se há mudanças não salvas
    pub dirty: bool,
}

impl EditHistory {
    pub fn new(max_history: usize) -> Self {
        Self {
            undo_stack: Vec::with_capacity(max_history),
            redo_stack: Vec::with_capacity(max_history),
            max_history,
            dirty: false,
        }
    }

    /// Adiciona uma ação ao histórico
    pub fn push(&mut self, action: EditAction) {
        // Ao fazer nova ação, limpa redo stack
        self.redo_stack.clear();

        // Limita tamanho do histórico
        if self.undo_stack.len() >= self.max_history {
            self.undo_stack.remove(0);
        }

        self.undo_stack.push(action);
        self.dirty = true;
    }

    /// Pega a ação anterior (para undo)
    pub fn undo(&mut self) -> Option<EditAction> {
        self.undo_stack.pop().map(|action| {
            self.redo_stack.push(action.clone());
            action
        })
    }

    /// Pega a ação desfeita (para redo)
    pub fn redo(&mut self) -> Option<EditAction> {
        self.redo_stack.pop().map(|action| {
            self.undo_stack.push(action.clone());
            action
        })
    }

    /// Verifica se pode fazer undo
    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    /// Verifica se pode fazer redo
    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    /// Limpa o histórico
    pub fn clear(&mut self) {
        self.undo_stack.clear();
        self.redo_stack.clear();
        self.dirty = false;
    }

    /// Marca como salvo (reseta dirty flag)
    pub fn mark_saved(&mut self) {
        self.dirty = false;
    }

    /// Retorna número de ações no undo stack
    pub fn undo_count(&self) -> usize {
        self.undo_stack.len()
    }

    /// Retorna número de ações no redo stack
    pub fn redo_count(&self) -> usize {
        self.redo_stack.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_undo_redo() {
        let mut history = EditHistory::new(100);

        let action1 = EditAction::Insert {
            pos: 0,
            text: "hello".to_string(),
        };
        history.push(action1);
        assert!(history.can_undo());
        assert!(!history.can_redo());

        let undone = history.undo();
        assert!(undone.is_some());
        assert!(!history.can_undo());
        assert!(history.can_redo());

        let redone = history.redo();
        assert!(redone.is_some());
        assert!(history.can_undo());
        assert!(!history.can_redo());
    }

    #[test]
    fn test_dirty_flag() {
        let mut history = EditHistory::new(100);

        assert!(!history.dirty);

        history.push(EditAction::Insert {
            pos: 0,
            text: "test".to_string(),
        });

        assert!(history.dirty);

        history.mark_saved();
        assert!(!history.dirty);
    }

    #[test]
    fn test_max_history() {
        let mut history = EditHistory::new(3);

        for i in 0..5 {
            history.push(EditAction::Insert {
                pos: 0,
                text: format!("text{}", i),
            });
        }

        // Apenas as últimas 3 devem estar presentes
        assert_eq!(history.undo_count(), 3);
    }

    #[test]
    fn test_redo_stack_cleared() {
        let mut history = EditHistory::new(100);

        history.push(EditAction::Insert {
            pos: 0,
            text: "a".to_string(),
        });
        history.undo();

        assert!(history.can_redo());

        history.push(EditAction::Insert {
            pos: 1,
            text: "b".to_string(),
        });

        // Redo stack deve ser limpo após nova ação
        assert!(!history.can_redo());
    }
}
