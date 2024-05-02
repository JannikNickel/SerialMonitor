use crate::serial_reader::{FlowCtrl, Parity, StartMode};

pub struct ConnectionConfig {
    pub port: String,
    pub baud_rate: u32,
    pub data_bits: u32,
    pub parity: Parity,
    pub stop_bits: u32,
    pub flow_ctrl: FlowCtrl,
    pub dtr: bool,
    pub start_mode: StartMode,
    pub start_delay: u32,
    pub start_msg: String
}

impl Default for ConnectionConfig {
    fn default() -> Self {
        Self {
            port: String::from("-"),
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

#[derive(Default)]
pub struct SerialMonitorData {
    pub conn_config: ConnectionConfig
}
