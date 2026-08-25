use crate::show::types::ShowSnapshot;

pub const MAX_UNDO_STACK_SIZE: usize = 100;

#[derive(Debug, Clone, Default)]
pub struct ShowHistory {
    pub undo_stack: Vec<ShowSnapshot>,
    pub redo_stack: Vec<ShowSnapshot>,
    pub saved_snapshot: Option<ShowSnapshot>,
}

impl ShowHistory {
    pub fn reset_with_saved(&mut self, snapshot: ShowSnapshot) {
        self.undo_stack.clear();
        self.redo_stack.clear();
        self.saved_snapshot = Some(snapshot);
    }

    pub fn reset_empty(&mut self) {
        self.undo_stack.clear();
        self.redo_stack.clear();
        self.saved_snapshot = None;
    }

    pub fn push_mutation(&mut self, previous_snapshot: ShowSnapshot) {
        self.undo_stack.push(previous_snapshot);
        if self.undo_stack.len() > MAX_UNDO_STACK_SIZE {
            self.undo_stack.remove(0);
        }
        self.redo_stack.clear();
    }

    pub fn mark_saved(&mut self, current_snapshot: ShowSnapshot) {
        self.saved_snapshot = Some(current_snapshot);
    }

    pub fn is_dirty(&self, current_snapshot: &ShowSnapshot) -> bool {
        match &self.saved_snapshot {
            Some(saved) => saved != current_snapshot,
            None => true,
        }
    }

    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }
}
