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
            log::info!(target: "live-dmx::engine", "live engine loop started at 44Hz");
            let tick_rate = Duration::from_micros(22727); // 44Hz standard DMX rate
            let mut artnet_sender = crate::output::protocols::ArtNetSender::new();
            let mut sacn_sender = crate::output::protocols::SacnSender::new().ok();
            let mut osc_sender = crate::output::protocols::OscSender::new();
            let mut kinet_sender = crate::output::protocols::KinetSender::new();
            let mut opc_sender = crate::output::protocols::OpcSender::new();
            let mut physical_sender = crate::output::interfaces::PhysicalDmxSender::new();

            while running.load(Ordering::SeqCst) {
                let start = Instant::now();

                match show_state.lock() {
                    Ok(show) => {
                        let show_snapshot = show.clone();
                        drop(show);
                        if let Ok(mut eng) = engine.lock() {
                            eng.compute_dmx(&show_snapshot);

                            for universe_cfg in &show_snapshot.output.universes {
                                if !universe_cfg.enabled {
                                    continue;
                                }
                                let u_val = universe_cfg.universe as u16;
                                if let Some(buf) = eng.universe_buffers.get(&u_val) {
                                    match universe_cfg.output.as_str() {
                                        "none" => {}
                                        "artnet" => artnet_sender.send_universe(u_val, buf),
                                        "sacn" => {
                                            if let Some(sender) = &mut sacn_sender {
                                                let _ = sender.send_universe(u_val, buf);
                                            }
                                        }
                                        "osc" => osc_sender.send_universe(u_val, buf),
                                        "kinet" => kinet_sender.send_universe(u_val, buf),
                                        "opc" => {
                                            let _ = opc_sender.send_universe(u_val, buf);
                                        }
                                        target => {
                                            let _ = physical_sender.send_to_target(target, buf);
                                        }
                                    }
                                }
                            }
                        }
                    }
                    Err(err) => {
                        log::error!(target: "live-dmx::engine", "live engine loop stopping: show state lock poisoned: {err}");
                        running.store(false, Ordering::SeqCst);
                        break;
                    }
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
