use std::sync::mpsc::channel;
use h24clock::ClockApp;
mod h24clock;

// When compiling natively:
fn main() {
    // Log to stdout (if you run with `RUST_LOG=debug`).
   // tracing_subscriber::fmt::init();

    let app = ClockApp::default();
    let options = eframe::NativeOptions {
        // Let's show off that we support transparent windows
        transparent: true,
        drag_and_drop_support: true,
        ..Default::default()
    };
    eframe::run_native(Box::new(app), options);
}
