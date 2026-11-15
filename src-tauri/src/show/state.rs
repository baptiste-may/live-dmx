//! Tauri-managed application state and the show mutation path.
//!
//! All show edits go through [`AppState::apply_mutation`] (or
//! [`AppState::mutate_show`]) so they are snapshotted for undo/redo, reset the
//! live engine when needed and notify the frontend with `show-state-changed`.
//! See `docs/contributing/architecture.md`.

use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Emitter};

use crate::output::LiveEngineRunner;
use crate::show::history::ShowHistory;
use crate::show::types::{ShowData, ShowStatus};

/// Global state managed by Tauri, created once at startup.
///
/// When locking several fields, take `show` before `history`; the engine lock
/// is taken last and never held across network or serial I/O.
pub struct AppState {
    /// Current show, shared with the output loop (which locks and borrows it each tick).
    pub show: Arc<Mutex<ShowData>>,
    /// Undo/redo stacks and saved snapshot of the current show.
    pub history: Mutex<ShowHistory>,
    /// Live engine and its ~44 Hz output loop.
    pub output: Arc<LiveEngineRunner>,
}

impl Default for AppState {
    /// Creates an empty show and immediately starts the live output loop on it.
    fn default() -> Self {
        let show = Arc::new(Mutex::new(ShowData::default()));
        let runner = Arc::new(LiveEngineRunner::new());

        runner.start_loop(show.clone());

        Self {
            show,
            history: Mutex::new(ShowHistory::default()),
            output: runner,
        }
    }
}

impl AppState {
    /// Resets all live engine state (fixture values, effects, ...).
    ///
    /// Called whenever the patch or group list changes, and when a show is
    /// created, loaded, imported, unloaded or undone/redone across such changes.
    ///
    /// # Errors
    ///
    /// Returns an error if the engine mutex is poisoned.
    pub fn reset_live_engine(&self) -> Result<(), String> {
        let mut live = self.output.engine.lock().map_err(|e| e.to_string())?;
        live.reset_all();
        Ok(())
    }

    /// Runs `f` on the show as an undoable mutation, without emitting any event.
    ///
    /// The show is snapshotted first. If `f` fails, the snapshot-tracked fields are
    /// restored; changes `f` made to `show.file` or `show.status` (including dirty
    /// flags) are not rolled back. If `f` changes the snapshot, the live engine is
    /// reset when the patch or the group list differ, the previous snapshot is
    /// pushed on the undo stack (clearing redo) and `file.is_dirty`, `can_undo` and
    /// `can_redo` are recomputed.
    ///
    /// Returns the closure result and whether the show actually changed. Locks
    /// `show` then `history`.
    ///
    /// # Errors
    ///
    /// Returns the closure's error, or an error if the show, history or engine
    /// mutex is poisoned.
    pub fn mutate_show<F, R>(&self, f: F) -> Result<(R, bool), String>
    where
        F: FnOnce(&mut ShowData) -> Result<R, String>,
    {
        let mut show = self.show.lock().map_err(|e| e.to_string())?;
        let mut history = self.history.lock().map_err(|e| e.to_string())?;

        let prev_snapshot = show.snapshot();
        let result = match f(&mut show) {
            Ok(res) => res,
            Err(err) => {
                show.restore_snapshot(prev_snapshot);
                return Err(err);
            }
        };

        let current_snapshot = show.snapshot();
        let changed = prev_snapshot != current_snapshot;
        if changed {
            if prev_snapshot.patch != current_snapshot.patch
                || prev_snapshot.groups.groups != current_snapshot.groups.groups
            {
                match self.output.engine.lock() {
                    Ok(mut live) => live.reset_all(),
                    Err(err) => {
                        return Err(format!("failed to reset live engine: {err}"));
                    }
                }
            }
            history.push_mutation(prev_snapshot);
            show.file.is_dirty = history.is_dirty(&current_snapshot);
            show.file.can_undo = history.can_undo();
            show.file.can_redo = history.can_redo();
        }

        Ok((result, changed))
    }

    /// Like [`AppState::mutate_show`], then emits `show-state-changed` with the
    /// full show when the mutation changed something. This is the mutation path
    /// command handlers should use.
    ///
    /// # Errors
    ///
    /// Same as [`AppState::mutate_show`].
    pub fn apply_mutation<F, R>(&self, app: &AppHandle, f: F) -> Result<R, String>
    where
        F: FnOnce(&mut ShowData) -> Result<R, String>,
    {
        let (result, changed) = self.mutate_show(f)?;
        if changed {
            if let Ok(show) = self.show.lock() {
                emit_show_changed(app, &show);
            }
        }

        Ok(result)
    }
}

/// Emits `show-state-changed` with the full `show` as payload.
///
/// Emission failures are ignored.
pub fn emit_show_changed(app: &AppHandle, show: &ShowData) {
    let _ = app.emit("show-state-changed", show);
}

/// Emits a partial `show-state-changed` payload `{ status, progress, statusMessage }`
/// used by long operations (load, save, MVR import) to drive the progress UI.
///
/// `progress` is a percentage (0-100). Emission failures are ignored.
pub fn emit_progress(app: &AppHandle, status: ShowStatus, progress: u8, message: &str) {
    let payload = serde_json::json!({
        "status": status,
        "progress": progress,
        "statusMessage": message,
    });
    let _ = app.emit("show-state-changed", payload);
}
