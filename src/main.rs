#![allow(dead_code)]

mod app;
mod data;
mod serial_reader;
mod serial_parser;
mod ui;

use app::SerialMonitorApp;
use data::{PlotData, SerialMonitorData};

fn main() {
    let mut data = SerialMonitorData::default();
    data.plots = vec![PlotData::new("Plot 1")];
    if let Err(e) = SerialMonitorApp::run(data) {
        println!("{:?}", e);
        std::process::exit(1);
    }
}
