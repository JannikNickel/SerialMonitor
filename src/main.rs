#![allow(dead_code)]

mod app;
mod data;
mod serial_reader;
mod ui;

use app::SerialMonitorApp;
use data::SerialMonitorData;

fn main() {
    let data = SerialMonitorData::default();
    if let Err(e) = SerialMonitorApp::run(data) {
        println!("{:?}", e);
        std::process::exit(1);
    }
}
