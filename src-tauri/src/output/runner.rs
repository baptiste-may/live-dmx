use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use crate::output::engine::LiveEngine;
use crate::show::ShowData;

pub struct LiveEngineRunner {
    pub engine: Arc<Mutex<LiveEngine>>,
    pub running: Arc<AtomicBool>,
}

impl Default for LiveEngineRunner {
    fn default() -> Self {
        Self::new()
    }
}

impl LiveEngineRunner {
    pub fn new() -> Self {
        Self {
            engine: Arc::new(Mutex::new(LiveEngine::new())),
            running: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn start_loop(&self, show_state: Arc<Mutex<ShowData>>) {
        if self.running.swap(true, Ordering::SeqCst) {
            return;
        }

        let engine = self.engine.clone();
        let running = self.running.clone();

        thread::spawn(move || {
            let tick_rate = Duration::from_micros(22727); // 44Hz standard DMX rate

            while running.load(Ordering::SeqCst) {
                let start = Instant::now();

                let show_snapshot = {
                    if let Ok(show) = show_state.lock() {
                        show.clone()
                    } else {
                        ShowData::default()
                    }
                };

                if let Ok(mut eng) = engine.lock() {
                    eng.compute_dmx(&show_snapshot);
                }

                let elapsed = start.elapsed();
                if elapsed < tick_rate {
                    thread::sleep(tick_rate - elapsed);
                }
            }
        });
    }

    pub fn stop_loop(&self) {
        self.running.store(false, Ordering::SeqCst);
    }
}
