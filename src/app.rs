use crate::data::{ConnectionConfig, SerialMonitorData};
use crate::serial_reader::{SerialConfig, SerialError, SerialReader, StartMode};
use crate::ui::SerialMonitorUI;

const WIN_WIDTH: f32 = 1280.0;
const WIN_HEIGHT: f32 = 720.0;

#[derive(Default)]
pub struct SerialMonitorApp {
    data: SerialMonitorData,
    ui: Option<SerialMonitorUI>,

    reader: Option<SerialReader>
}

impl SerialMonitorApp {
    pub fn run(data: SerialMonitorData) -> Result<(), String> {
        let native_opts = eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default()
                .with_inner_size(egui::Vec2::new(WIN_WIDTH, WIN_HEIGHT))
                .with_min_inner_size(egui::Vec2::new(WIN_WIDTH, WIN_HEIGHT)),
            ..Default::default()
        };
        eframe::run_native(
            "SerialMonitor",
            native_opts,
            Box::new(|ctx| {
                let ui = SerialMonitorUI::new(ctx);
                let app = SerialMonitorApp {
                    data,
                    ui: Some(ui),
                    ..Default::default()
                };
                Box::new(app)
            }),
        )
        .map_err(|e| e.to_string())
    }

    pub fn update(&mut self) {
        self.reset_port_if_missing();
    }

    pub fn reset_port_if_missing(&mut self) -> bool {
        if !self.available_devices().contains(&self.data.conn_config.port) {
            self.data.conn_config.port = String::from(ConnectionConfig::NO_PORT);
            return true;
        }
        false
    }

    pub fn conn_config(&mut self) -> &mut ConnectionConfig {
        &mut self.data.conn_config
    }

    pub fn available_devices(&self) -> Vec<String> {
        match serialport::available_ports() {
            Ok(ports) => ports.iter().map(|n| n.port_name.to_owned()).collect(),
            Err(_) => vec![]
        }
    }

    pub fn can_connect(&self) -> bool {
        self.data.conn_config.port != ConnectionConfig::NO_PORT
    }

    pub fn is_connected(&self) -> bool {
        match &self.reader {
            Some(reader) => reader.is_open(),
            None => false
        }
    }

    pub fn connect_current(&mut self) -> Result<(), SerialError> {
        let mut reader = SerialReader::new(SerialConfig::from(self.data.conn_config.clone()));
        reader.open(self.data.conn_config.dtr)?;
        reader.begin_read(StartMode::from(self.data.conn_config.clone()))?;
        self.reader = Some(reader);
        Ok(())
    }

    pub fn disconnect_current(&mut self) {
        if let Some(reader) = self.reader.take() {
            std::mem::drop(reader);
        }
    }
}

impl eframe::App for SerialMonitorApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        self.update();
        if let Some(mut ui) = self.ui.take() {
            ui.update(ctx, frame, self);
            self.ui = Some(ui);
        }
    }
}
