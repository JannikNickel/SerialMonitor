use crate::serial_reader::{FlowCtrl, Parity, SerialConfig, StartMode};
use std::{fmt::Display, fs::File, io::Write, path::PathBuf, sync::atomic::{AtomicUsize, Ordering}, time::Duration};
use serde::{Serialize, Deserialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct ConnectionConfig {
    pub port: String,
    pub baud_rate: u32,
    pub data_bits: u8,
    pub parity: Parity,
    pub stop_bits: u8,
    pub flow_ctrl: FlowCtrl,
    pub dtr: bool,
    pub start_mode: StartMode,
    pub start_delay: u32,
    pub start_msg: String
}

impl Default for ConnectionConfig {
    fn default() -> Self {
        Self {
            port: String::from(ConnectionConfig::NO_PORT),
            baud_rate: 9600,
            data_bits: 8,
            parity: Parity::None,
            stop_bits: 1,
            flow_ctrl: FlowCtrl::None,
            dtr: true,
            start_mode: StartMode::Delay(Duration::ZERO),
            start_delay: 1000,
            start_msg: String::from("Start")
        }
    }
}

impl From<ConnectionConfig> for SerialConfig {
    fn from(value: ConnectionConfig) -> Self {
        SerialConfig {
            port: value.port,
            baud_rate: value.baud_rate,
            data_bits: value.data_bits,
            parity: value.parity,
            stop_bits: value.stop_bits,
            flow_ctrl: value.flow_ctrl,
            timeout: Duration::ZERO
        }
    }
}

impl From<ConnectionConfig> for StartMode {
    fn from(value: ConnectionConfig) -> Self {
        match value.start_mode {
            Self::Immediate => StartMode::Immediate,
            Self::Delay(_) => StartMode::Delay(Duration::from_millis(value.start_delay as u64)),
            Self::Message(_) => StartMode::Message(value.start_msg)
        }
    }
}

impl ConnectionConfig {
    pub const NO_PORT: &'static str = "-";
}

#[derive(PartialEq, Clone, Copy, Debug, Serialize, Deserialize)]
pub enum PlotMode {
    Continous,
    Cyclic
}

impl Display for PlotMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(PartialEq, Clone, Copy, Debug, Serialize, Deserialize)]
pub enum PlotScaleMode {
    Auto,
    AutoMax,
    Manual
}

impl Display for PlotScaleMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Serialize, Deserialize)]
pub struct PlotConfig {
    pub mode: PlotMode,
    pub window: f64,
    pub scale_mode: PlotScaleMode,
    pub y_min: f64,
    pub y_max: f64
}

impl Default for PlotConfig {
    fn default() -> Self {
        Self { 
            mode: PlotMode::Continous,
            window: 5.0,
            scale_mode: PlotScaleMode::Auto,
            y_min: 0.0,
            y_max: 1.0
        }
    }
}

#[derive(Default, Serialize, Deserialize)]
pub struct InputSlot {
    pub index: usize,
    pub name: String,
    pub color: [f32; 3],
    #[serde(skip)]
    pub value: f64
}

#[derive(Serialize, Deserialize)]
pub struct PlotData {
    pub id: usize,
    pub name: String,
    pub hidden: Vec<usize>,
    pub height: f32,
    pub console: bool
}

static PLOT_ID: AtomicUsize = AtomicUsize::new(1);

impl PlotData {
    pub fn new(name: &str) -> Self {
        Self {
            id: PLOT_ID.fetch_add(1, Ordering::SeqCst),
            name: name.to_owned(),
            hidden: Vec::new(),
            height: 256.0,
            console: false
        }
    }

    pub fn console() -> Self {
        Self {
            id: PLOT_ID.fetch_add(1, Ordering::SeqCst),
            name: String::from("Console"),
            hidden: Vec::new(),
            height: 192.0,
            console: true
        }
    }

    pub fn update_internal_ids(plots: &Vec<PlotData>) {
        if let Some(max) = plots.iter().max_by_key(|n| n.id) {
            PLOT_ID.store(max.id + 1, Ordering::SeqCst);
        }
    }
}

#[derive(Default, Serialize, Deserialize)]
pub struct SerialMonitorData {
    pub conn_config: ConnectionConfig,
    pub plot_config: PlotConfig,
    pub inp_slots: Vec<InputSlot>,
    pub plots: Vec<PlotData>
}

impl SerialMonitorData {
    pub fn serialize(path: &PathBuf, data: &SerialMonitorData) -> std::io::Result<()> {
        let config = serde_json::to_string_pretty(data)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        let mut file = File::create(&path)?;
        file.write_all(config.as_bytes())?;
        Ok(())
    }

    pub fn deserialize(path: &PathBuf) -> Result<SerialMonitorData, std::io::Error> {
        let file = File::open(path)?;
        let config: SerialMonitorData = serde_json::from_reader(&file)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        Ok(config)
    }
}
