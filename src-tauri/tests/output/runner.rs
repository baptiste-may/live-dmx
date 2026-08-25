use live_dmx_lib::output::LiveEngineRunner;
use live_dmx_lib::show::ShowData;
use std::sync::{Arc, Mutex};
use std::time::Duration;

#[test]
fn test_live_engine_runner_start_stop() {
    let runner = LiveEngineRunner::new();
    let show = Arc::new(Mutex::new(ShowData::default()));

    runner.start_loop(show);
    assert!(runner.running.load(std::sync::atomic::Ordering::SeqCst));

    std::thread::sleep(Duration::from_millis(50));

    runner.stop_loop();
    assert!(!runner.running.load(std::sync::atomic::Ordering::SeqCst));
}
