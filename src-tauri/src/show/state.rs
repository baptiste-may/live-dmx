use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Emitter};

use crate::output::LiveEngineRunner;
use crate::show::history::ShowHistory;
use crate::show::types::ShowData;

pub struct AppState {
    pub show: Arc<Mutex<ShowData>>,
    pub history: Mutex<ShowHistory>,
    pub output: Arc<LiveEngineRunner>,
}

impl Default for AppState {
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
    pub fn reset_live_engine(&self) -> Result<(), String> {
        let mut live = self.output.engine.lock().map_err(|e| e.to_string())?;
        live.reset_all();
        Ok(())
    }

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

pub fn emit_show_changed(app: &AppHandle, show: &ShowData) {
    let _ = app.emit("show-state-changed", show);
}
