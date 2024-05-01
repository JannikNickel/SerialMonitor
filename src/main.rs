mod serial_reader;
mod app;

use app::SerialReaderApp;

const WIN_WIDTH: f32 = 1280.0;
const WIN_HEIGHT: f32 = 720.0;

fn main() {
    let native_opts = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size(egui::Vec2::new(WIN_WIDTH, WIN_HEIGHT))
            .with_min_inner_size(egui::Vec2::new(WIN_WIDTH, WIN_HEIGHT)
        ),
        ..Default::default()
    };
    let _ = eframe::run_native(
        "SerialMonitor",
        native_opts,
        Box::new(|ctx| Box::new(SerialReaderApp::new(ctx))));
}
