use crate::data::{ConnectionConfig, InputSlot, PlotConfig, PlotData, SerialMonitorData};
use crate::serial_parser::SerialParser;
use crate::serial_reader::{SerialConfig, SerialError, SerialReader, StartMode};
use crate::ui::{Notification, NotificationType, SerialMonitorUI};
use std::collections::VecDeque;
use std::io::Write;
use std::iter::zip;
use std::time::Duration;
use egui::ecolor::rgb_from_hsv;

const WIN_WIDTH: f32 = 1280.0;
const WIN_HEIGHT: f32 = 720.0;

pub struct SerialMonitorApp {
    data: SerialMonitorData,
    ui: Option<SerialMonitorUI>,

    reader: Option<SerialReader>,
    parser: SerialParser,

    values: Vec<Vec<[f64; 2]>>,
    lines: VecDeque<String>,

    paused: bool,
    start_connected: bool,
    terminal_output: bool,
    headless: bool
}

impl SerialMonitorApp {
    pub const STORED_DURATION: f64 = 60.0;
    pub const STORED_LINES: usize = 512;

    pub fn run(data: SerialMonitorData, connect: bool, terminal_output: bool, headless: bool) -> Result<(), String> {
        let icon = image::load_from_memory(include_bytes!("../res/icon.ico")).unwrap();
        let icon = egui::IconData {
            width: icon.width(),
            height: icon.height(),
            rgba: icon.into_rgba8().into_raw()
        };
        let native_opts = eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default()
                .with_inner_size(egui::Vec2::new(WIN_WIDTH, WIN_HEIGHT))
                .with_min_inner_size(egui::Vec2::new(WIN_WIDTH, WIN_HEIGHT))
                .with_icon(icon),
            ..Default::default()
        };
        let mut app = SerialMonitorApp {
            data,
            ui: None,
            reader: None,
            parser: SerialParser::new(),
            values: Vec::new(),
            lines: VecDeque::new(),
            paused: false,
            start_connected: connect,
            terminal_output: terminal_output,
            headless: headless
        };

        if headless {
            loop {
                app.update();
            }
        }

        eframe::run_native(
            "SerialMonitor",
            native_opts,
            Box::new(move |ctx| {
                let ui = SerialMonitorUI::new(ctx);
                app.ui = Some(ui);
                Box::new(app)
            }),
        )
        .map_err(|e| e.to_string())
    }

    pub fn update(&mut self) {
        self.reset_port_if_missing();
        self.read_input();
        self.prep_input_slots(self.parser.columns());

        if self.start_connected {
            self.start_connected = false;
            if let Err(e) = self.connect_current() {
                self.error(&e.to_string());
            }
        }
    }

    pub fn reset_port_if_missing(&mut self) -> bool {
        if !self.available_devices().contains(&self.data.conn_config.port) {
            self.data.conn_config.port = String::from(ConnectionConfig::NO_PORT);
            return true;
        }
        false
    }

    fn read_input(&mut self) {
        let mut err: Option<String> = None;
        if let Some(mut reader) = self.reader.take() {
            while let Some(line) = reader.get_line() {
                match &line {
                    Ok(line) => {
                        if !self.paused {
                            match self.parser.parse_values(&line.content) {
                                Ok(values) => self.handle_input(line.t, &values),
                                Err(e) => self.warning(&e.to_string())
                            }
                            self.handle_input_line(line.t, &line.content);
                        }
                    },
                    Err(e) => {
                        err = Some(e.to_string());
                        break;
                    }
                }
            }
            self.reader = Some(reader);
        }
        if let Some(e) = err {
            self.error(&e)
        }
    }

    fn handle_input(&mut self, t: f64, values: &Vec<f64>) {
        while self.values.len() < values.len() {
            self.values.push(Vec::new());
        }
        for (l, r) in zip(&mut self.values, values) {
            l.push([t, *r]);
        }
    }

    fn handle_input_line(&mut self, t: f64, line: &String) {
        let fmt_line = format!("[{:.2}] > {}", t, line);
        if self.terminal_output {
            println!("{}", &fmt_line);
            _ = std::io::stdout().flush();
        }
        self.lines.push_back(fmt_line);
        if self.lines.len() > Self::STORED_LINES {
            self.lines.pop_front();
        }
    }

    fn prep_input_slots(&mut self, slots: usize) {
        for i in self.data.inp_slots.len()..slots {
            let col = rgb_from_hsv((i as f32 * 0.15 % 1.0, 0.8, 0.8));
            let slot = InputSlot {
                index: i,
                name: format!("Slot {}", (i + 1)),
                color: col,
                value: 0.0
            };
            self.data.inp_slots.push(slot);
        }

        for (i, slot) in self.data.inp_slots.iter_mut().enumerate() {
            if i < self.values.len() {
                slot.value = self.values[i].last().unwrap_or(&[0.0, 0.0])[1];
            }
        }
    }

    fn warning(&mut self, msg: &str) {
        if let Some(ui) = &mut self.ui {
            ui.set_notification(Notification::new(msg, Duration::from_secs(5), NotificationType::Warning), true);
        }
    }

    fn error(&mut self, msg: &str) {
        if let Some(ui) = &mut self.ui {
            ui.set_notification(Notification::new(msg, Duration::from_secs(5), NotificationType::Error), false);
        }
        if self.headless {
            eprintln!("{}", msg);
            _ = std::io::stdout().flush();
        }
        self.disconnect_current();
    }

    pub fn conn_config(&mut self) -> &mut ConnectionConfig {
        &mut self.data.conn_config
    }

    pub fn plot_config_mut(&mut self) -> &mut PlotConfig {
        &mut self.data.plot_config
    }

    pub fn plot_config(&self) -> &PlotConfig {
        &self.data.plot_config
    }

    pub fn input_slots_mut(&mut self) -> &mut Vec<InputSlot> {
        &mut self.data.inp_slots
    }

    pub fn input_slots(&self) -> &Vec<InputSlot> {
        &self.data.inp_slots
    }

    pub fn input_columns(&self) -> usize {
        self.parser.columns()
    }

    pub fn plots_mut(&mut self) -> &mut Vec<PlotData> {
        &mut self.data.plots
    }

    pub fn plots(&self) -> &Vec<PlotData> {
        &self.data.plots
    }

    pub fn raw_values(&self) -> &Vec<Vec<[f64; 2]>> {
        &self.values
    }

    pub fn console_lines(&self) -> &VecDeque<String> {
        &self.lines
    }

    pub fn available_devices(&self) -> Vec<String> {
        match serialport::available_ports() {
            Ok(ports) => ports.iter().map(|n| n.port_name.to_owned()).collect(),
            Err(_) => Vec::new()
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
        let mut config = SerialConfig::from(self.data.conn_config.clone());
        config.timeout = Duration::from_millis(50);
        let mut reader = SerialReader::new(config);
        reader.open(self.data.conn_config.dtr)?;
        reader.begin_read(StartMode::from(self.data.conn_config.clone()))?;
        self.reader = Some(reader);
        self.paused = false;
        Ok(())
    }

    pub fn disconnect_current(&mut self) {
        if let Some(reader) = self.reader.take() {
            std::mem::drop(reader);
            self.parser.reset();
            self.values.clear();
            self.paused = false;
        }
    }

    pub fn has_input(&self) -> bool {
        self.parser.columns() > 0
    }

    pub fn add_plot(&mut self) {
        let off = self.has_console() as usize;
        let index = self.data.plots.len() - off;
        self.data.plots.insert(index, PlotData::new(&format!("Plot {}", self.data.plots.len() + 1 - off)));
    }

    pub fn remove_plot(&mut self, index: usize) {
        self.data.plots.remove(index);
    }

    pub fn reset_plot(&mut self, index: usize) {
        if self.data.plots[index].console {
            self.lines.clear();
        }
    }

    pub fn add_console(&mut self) {
        if !self.has_console() {
            self.data.plots.push(PlotData::console())
        }
    }

    pub fn has_console(&self) -> bool {
        self.data.plots.iter().any(|n| n.console)
    }

    pub fn is_paused(&self) -> bool {
        self.paused
    }

    pub fn set_paused(&mut self, pause: bool) {
        self.paused = pause
    }

    pub fn zoom_enabled(&self) -> bool {
        self.is_paused()
    }

    pub fn save_config_to_file(&self) -> std::io::Result<Option<String>> {
        let file = rfd::FileDialog::new()
            .add_filter("JSON", &["json"])
            .save_file();
        if let Some(path) = file {
            SerialMonitorData::serialize(&path, &self.data)?;
            return Ok(path.into_os_string().into_string().ok());
        }
        Ok(None)
    }

    pub fn load_config_from_file(&mut self, ui: &mut SerialMonitorUI) -> std::io::Result<bool> {
        let file = rfd::FileDialog::new()
            .add_filter("JSON", &["json"])
            .pick_file();
        if let Some(path) = file {
            let config = SerialMonitorData::deserialize(&path)?;
            self.load_config(config, ui);
            return Ok(true);
        }
        Ok(false)
    }

    pub fn load_config(&mut self, config: SerialMonitorData, ui: &mut SerialMonitorUI) {
        ui.reset();
        self.disconnect_current();
        self.data = config;
        PlotData::update_internal_ids(&self.data.plots);
    }
}

impl eframe::App for SerialMonitorApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        self.update();
        if let Some(mut ui) = self.ui.take() {
            ui.update(ctx, frame, self);
            self.ui = Some(ui);
        }
        ctx.request_repaint();
    }
}
