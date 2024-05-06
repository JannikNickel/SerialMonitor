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
    #[arg(long)]
    config: Option<String>,

    #[arg(short, long, action)]
    connect: bool
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
    if let Err(e) = SerialMonitorApp::run(data, args.config.is_some() && args.connect) {
        eprintln!("{:?}", e);
        std::process::exit(1);
    }
}
