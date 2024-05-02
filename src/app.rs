use crate::data::{ConnectionConfig, SerialMonitorData};
use crate::ui::SerialMonitorUI;

const WIN_WIDTH: f32 = 1280.0;
const WIN_HEIGHT: f32 = 720.0;

pub struct SerialMonitorApp {
    data: SerialMonitorData,
}

impl SerialMonitorApp {
    pub fn run(data: SerialMonitorData) -> Result<(), String> {
        let app = SerialMonitorApp { data };

        let native_opts = eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default()
                .with_inner_size(egui::Vec2::new(WIN_WIDTH, WIN_HEIGHT))
                .with_min_inner_size(egui::Vec2::new(WIN_WIDTH, WIN_HEIGHT)),
            ..Default::default()
        };
        eframe::run_native(
            "SerialMonitor",
            native_opts,
            Box::new(|ctx| Box::new(SerialMonitorUI::new(app, ctx))),
        )
        .map_err(|e| e.to_string())
    }

    pub fn conn_config(&mut self) -> &mut ConnectionConfig {
        &mut self.data.conn_config
    }
}
