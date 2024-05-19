mod app;
mod data;
mod serial_reader;
mod serial_parser;
mod ui;

use app::SerialMonitorApp;
use data::{PlotData, SerialMonitorData};
use std::path::PathBuf;
use clap::Parser;

#[derive(Parser, Debug)]
struct Args {
    #[arg(long, help = "Path to a json file containing a saved configuration")]
    config: Option<String>,

    #[arg(short, long, action, help = "Connect to the port from the configuration file")]
    connect: bool,

    #[arg(short, long, action, help = "Enable output to the console/terminal")]
    terminal: bool,

    #[arg(long, action, help = "Prevent GUI creation", requires_all = &["config", "connect"])]
    headless: bool
}

fn main() {
    let args = Args::parse();
    let mut data = SerialMonitorData::default();
    data.plots = vec![PlotData::new("Plot 1")];
    if let Some(path) = &args.config {
        data = match SerialMonitorData::deserialize(&PathBuf::from(&path)) {
            Ok(config) => config,
            Err(e) => {
                eprintln!("Could not load config from file: {} ({})", path, e.to_string());
                std::process::exit(1);
            }
        };
    }
    if !args.terminal {
        #[cfg(target_os = "windows")]
        hide_console();
    }

    if let Err(e) = SerialMonitorApp::run(data, args.config.is_some() && args.connect, args.terminal, args.headless) {
        eprintln!("{:?}", e);
        std::process::exit(1);
    }
}

#[cfg(target_os = "windows")]
fn hide_console() {
    use windows::Win32::System::Console::FreeConsole;
    unsafe {
        FreeConsole().ok();
    }
}
