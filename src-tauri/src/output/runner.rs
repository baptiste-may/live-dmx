//! Background runner that drives the live engine at 44 Hz and streams the
//! computed universes to their configured outputs.
//!
//! See `docs/contributing/dmx-output.md` for the locking rules and protocols.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use crate::output::engine::LiveEngine;
use crate::show::ShowData;

/// Owns the shared [`LiveEngine`] and the lifecycle of the 44 Hz output thread.
pub struct LiveEngineRunner {
    /// Engine shared with the Tauri commands that mutate live state.
    pub engine: Arc<Mutex<LiveEngine>>,
    /// `true` while the output thread should keep running; cleared to stop it.
    pub running: Arc<AtomicBool>,
}

impl Default for LiveEngineRunner {
    fn default() -> Self {
        Self::new()
    }
}

impl LiveEngineRunner {
    /// Creates a stopped runner with a fresh engine.
    pub fn new() -> Self {
        Self {
            engine: Arc::new(Mutex::new(LiveEngine::new())),
            running: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Spawns the output thread unless it is already running.
    ///
    /// Each tick (about 22.7 ms, 44 Hz) it locks `show_state` then the engine to
    /// compute the frames and copies the enabled universes into private send
    /// buffers; both locks are released before any network or serial I/O. Output
    /// routing follows each universe's `output` target (`artnet`, `sacn`, `osc`,
    /// `kinet`, `opc`, otherwise a physical interface name). Send errors are
    /// ignored. A poisoned show lock stops the loop. The thread is detached and
    /// is not joined.
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
            let mut send_buffers: std::collections::HashMap<u16, (String, Vec<u8>)> =
                std::collections::HashMap::new();

            while running.load(Ordering::SeqCst) {
                let start = Instant::now();

                let transmissions: Vec<(u16, String)> = match show_state.lock() {
                    Ok(show) => {
                        let configs: Vec<(u16, bool, String)> = show
                            .output
                            .universes
                            .iter()
                            .map(|u| (u.universe as u16, u.enabled, u.output.clone()))
                            .collect();

                        let mut to_send = Vec::new();
                        if let Ok(mut eng) = engine.lock() {
                            eng.compute_dmx(&show);

                            for (u_val, enabled, output) in configs {
                                if enabled && output != "none" {
                                    if let Some(buf) = eng.universe_buffers.get(&u_val) {
                                        let entry = send_buffers
                                            .entry(u_val)
                                            .or_insert_with(|| (output.clone(), vec![0u8; 512]));
                                        entry.0 = output.clone();
                                        entry.1.copy_from_slice(buf);
                                        to_send.push((u_val, output));
                                    }
                                }
                            }
                        }
                        drop(show);
                        to_send
                    }
                    Err(err) => {
                        log::error!(target: "live-dmx::engine", "live engine loop stopping: show state lock poisoned: {err}");
                        running.store(false, Ordering::SeqCst);
                        break;
                    }
                };

                // Network & Serial I/O completely outside show and engine locks!
                for (u_val, output) in transmissions {
                    if let Some((_, buf)) = send_buffers.get(&u_val) {
                        match output.as_str() {
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

                let elapsed = start.elapsed();
                if elapsed < tick_rate {
                    thread::sleep(tick_rate - elapsed);
                }
            }
        });
    }

    /// Asks the output thread to exit after its current tick; does not wait for it.
    pub fn stop_loop(&self) {
        self.running.store(false, Ordering::SeqCst);
    }
}
