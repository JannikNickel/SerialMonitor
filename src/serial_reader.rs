use serialport::{self, DataBits, SerialPort};
use std::collections::VecDeque;
use std::fmt::Display;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::sync::Mutex;
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};
use serde::{Serialize, Deserialize};

#[derive(Debug)]
pub enum SerialError {
    UnsupportedDataBits(u8),
    UnsupportedStopBits(u8),
    OpenError(String),
    WriteDtrError,
    PortNotOpen,
    AlreadyOpen,
    AlreadyReading,
    ReadError(String),
}

impl Display for SerialError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(PartialEq, Clone, Copy, Debug, Serialize, Deserialize)]
pub enum Parity {
    None,
    Odd,
    Even,
}

impl Display for Parity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(PartialEq, Clone, Copy, Debug, Serialize, Deserialize)]
pub enum FlowCtrl {
    None,
    Software,
    Hardware,
}

impl Display for FlowCtrl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(PartialEq, Clone, Debug, Serialize, Deserialize)]
pub enum StartMode {
    Immediate,
    Delay(Duration),
    Message(String),
}

impl Display for StartMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Immediate => write!(f, "Immediate"),
            Self::Delay(delay) => match *delay == Duration::ZERO {
                true => write!(f, "Delay"),
                false => write!(f, "{:?}", self)
            }
            Self::Message(msg) => match msg.is_empty() {
                true => write!(f, "Message"),
                false => write!(f, "{:?}", self)
            }
        }
    }
}

pub struct Line {
    pub t: f64,
    pub content: String,
}

pub struct SerialConfig {
    pub port: String,
    pub baud_rate: u32,
    pub data_bits: u8,
    pub parity: Parity,
    pub stop_bits: u8,
    pub flow_ctrl: FlowCtrl,
    pub timeout: Duration,
}

pub struct SerialReader {
    config: SerialConfig,
    port: Option<Box<dyn serialport::SerialPort>>,
    lines: Arc<Mutex<VecDeque<Result<Line, SerialError>>>>,
    worker_thread: Option<JoinHandle<()>>,
    stop: Arc<AtomicBool>,
}

impl SerialReader {
    pub fn new(config: SerialConfig) -> SerialReader {
        SerialReader {
            config: config,
            port: None,
            lines: Arc::new(Mutex::new(VecDeque::new())),
            worker_thread: None,
            stop: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn open(&mut self, dtr: bool) -> Result<(), SerialError> {
        if self.is_open() {
            return Err(SerialError::AlreadyOpen);
        }

        let config = &self.config;
        let port = serialport::new(&config.port, config.baud_rate)
            .data_bits(match config.data_bits {
                5 => DataBits::Five,
                6 => DataBits::Six,
                7 => DataBits::Seven,
                8 => DataBits::Eight,
                _ => return Err(SerialError::UnsupportedDataBits(config.data_bits)),
            })
            .parity(match config.parity {
                Parity::Odd => serialport::Parity::Odd,
                Parity::Even => serialport::Parity::Even,
                Parity::None => serialport::Parity::None,
            })
            .stop_bits(match config.stop_bits {
                1 => serialport::StopBits::One,
                2 => serialport::StopBits::Two,
                _ => return Err(SerialError::UnsupportedStopBits(config.stop_bits)),
            })
            .flow_control(match config.flow_ctrl {
                FlowCtrl::None => serialport::FlowControl::None,
                FlowCtrl::Software => serialport::FlowControl::Software,
                FlowCtrl::Hardware => serialport::FlowControl::Hardware
            })
            .timeout(config.timeout);

        let mut p = port
            .open()
            .map_err(|e| SerialError::OpenError(e.to_string()))?;
        p.write_data_terminal_ready(dtr)
            .map_err(|_| SerialError::WriteDtrError)?;
        self.port = Some(p);
        Ok(())
    }

    pub fn begin_read(&mut self, start_mode: StartMode) -> Result<(), SerialError> {
        if self.worker_thread.is_some() {
            return Err(SerialError::AlreadyReading);
        }

        let mut port = match self.port.take() {
            Some(p) => p,
            None => return Err(SerialError::PortNotOpen),
        };

        let lines = Arc::clone(&self.lines);
        let stop = Arc::clone(&self.stop);
        let handle = thread::spawn(move || {
            let mut line_buf = String::new();
            let start_time = Instant::now();
            let start_off = match start_mode {
                StartMode::Delay(delay) => delay.as_secs_f64(),
                _ => 0.0
            };
            let mut started = matches!(start_mode, StartMode::Immediate);
            loop {
                if stop.load(Ordering::Relaxed) {
                    break;
                }

                line_buf.clear();
                let res = read_line(&mut port, &mut line_buf);
                let line = line_buf.trim();
                let t = start_time.elapsed();

                started |= match start_mode {
                    StartMode::Immediate => true,
                    StartMode::Delay(delay) => t >= delay,
                    StartMode::Message(ref msg) => {
                        let was_started = started;
                        started |= line.ends_with(msg);
                        if !was_started {
                            continue;
                        }
                        started
                    }
                };
                if !started {
                    continue;
                }

                match res {
                    Ok(0) => break,
                    Ok(_) => {
                        if let Ok(mut locked_lines) = lines.lock() {
                            locked_lines.push_back(Ok(Line {
                                t: t.as_secs_f64() - start_off,
                                content: line.to_owned(),
                            }));
                        }
                    },
                    Err(e) => {
                        if let Ok(mut locked_lines) = lines.lock() {
                            locked_lines.push_back(Err(SerialError::ReadError(e)));
                        }
                        break;
                    }
                }
            }
        });

        self.worker_thread = Some(handle);
        Ok(())
    }

    fn stop_read(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        if let Some(handle) = self.worker_thread.take() {
            let _ = handle.join();
        }
    }

    pub fn get_line(&mut self) -> Option<Result<Line, SerialError>> {
        if let Ok(mut lines) = self.lines.lock() {
            return lines.pop_front();
        }
        None
    }

    pub fn is_open(&self) -> bool {
        self.port.is_some() || self.worker_thread.is_some()
    }
}

impl Drop for SerialReader {
    fn drop(&mut self) {
        self.stop_read();
    }
}

fn read_line(port: &mut Box<dyn SerialPort>, buf: &mut String) -> Result<usize, String> {
    let mut buffer = [b'\0'];
    let mut nread = 0;
    loop {
        let read = port.read(&mut buffer).map_err(|e| e.to_string())?;
        if read != 1 {
            return Err(String::from("Unexpected byte amount!"))
        }
        let c = match char::from_u32(buffer[0] as u32) {
            Some(c) => c,
            None => return Err(String::from("Byte is not a valid ASCII character!"))
        };
        if c == '\n' {
            return Ok(nread);
        }
        nread += 1;
        buf.push(c);
    }
}
