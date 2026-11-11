//! Undo/redo history and dirty tracking for the show.
//!
//! Stores whole-show [`ShowSnapshot`]s rather than diffs. See
//! `docs/contributing/architecture.md`.

use crate::show::types::ShowSnapshot;

/// Maximum number of undo entries kept; the oldest is dropped beyond this.
pub const MAX_UNDO_STACK_SIZE: usize = 100;

/// Undo/redo stacks plus the snapshot matching the file on disk.
#[derive(Debug, Clone, Default)]
pub struct ShowHistory {
    /// Previous states, most recent last.
    pub undo_stack: Vec<ShowSnapshot>,
    /// States undone and available for redo, most recent last. Cleared by any new mutation.
    pub redo_stack: Vec<ShowSnapshot>,
    /// Snapshot last saved or loaded; `None` when the show has no clean state.
    pub saved_snapshot: Option<ShowSnapshot>,
}

impl ShowHistory {
    /// Clears both stacks and records `snapshot` as the clean (saved) state.
    pub fn reset_with_saved(&mut self, snapshot: ShowSnapshot) {
        self.undo_stack.clear();
        self.redo_stack.clear();
        self.saved_snapshot = Some(snapshot);
    }

    /// Clears both stacks and forgets the saved snapshot, so the show counts as dirty.
    pub fn reset_empty(&mut self) {
        self.undo_stack.clear();
        self.redo_stack.clear();
        self.saved_snapshot = None;
    }

    /// Records `previous_snapshot` (the state before a mutation) on the undo
    /// stack, dropping the oldest entry past [`MAX_UNDO_STACK_SIZE`], and clears redo.
    pub fn push_mutation(&mut self, previous_snapshot: ShowSnapshot) {
        self.undo_stack.push(previous_snapshot);
        if self.undo_stack.len() > MAX_UNDO_STACK_SIZE {
            self.undo_stack.remove(0);
        }
        self.redo_stack.clear();
    }

    /// Records `current_snapshot` as the saved state, keeping the stacks.
    pub fn mark_saved(&mut self, current_snapshot: ShowSnapshot) {
        self.saved_snapshot = Some(current_snapshot);
    }

    /// Whether `current_snapshot` differs from the saved one; always `true`
    /// when there is no saved snapshot.
    pub fn is_dirty(&self, current_snapshot: &ShowSnapshot) -> bool {
        match &self.saved_snapshot {
            Some(saved) => saved != current_snapshot,
            None => true,
        }
    }

    /// Whether there is anything to undo.
    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    /// Whether there is anything to redo.
    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }
}
