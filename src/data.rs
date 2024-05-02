use std::time::Duration;

use crate::serial_reader::{FlowCtrl, Parity, SerialConfig, StartMode};

#[derive(Clone)]
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
            dtr: false,
            start_mode: StartMode::Immediate,
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
            timeout: Duration::from_millis(1)
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

#[derive(Default)]
pub struct SerialMonitorData {
    pub conn_config: ConnectionConfig
}
