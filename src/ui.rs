use crate::{app::SerialMonitorApp, serial_reader::{FlowCtrl, Parity, StartMode}};
use eframe::egui;
use egui_plot::{Corner, Legend, Line, PlotPoints};
use std::{fmt::Display, time::Duration};

const SIDEPANEL_WIDTH: f32 = 225.0;
const DROPDOWN_WIDTH: f32 = 150.0;
const STATUS_RADIUS: f32 = 6.0;

const BAUD_RATES: &[u32] = &[
    300, 600, 750, 1200, 2400, 4800, 9600, 19200, 31250, 38400, 57600, 74880, 115200, 230400,
    250000, 460800, 500000, 921600, 1000000, 2000000,
];
const DATA_BITS: &[u32] = &[5, 6, 7, 8, 9];
const PARITIES: &[Parity] = &[Parity::None, Parity::Odd, Parity::Even];
const STOP_BITS: &[u32] = &[1, 2];
const FLOW_CTRLS: &[FlowCtrl] = &[FlowCtrl::None, FlowCtrl::Software, FlowCtrl::Hardware];
const START_MODES: &[StartMode] = &[StartMode::Immediate, StartMode::Delay(Duration::ZERO), StartMode::Message(String::new())];

pub struct SerialMonitorUI {
    app: SerialMonitorApp
}

impl SerialMonitorUI {
    pub fn new(app: SerialMonitorApp, context: &eframe::CreationContext<'_>) -> Self {
        Self {
            app
        }
    }
    
    fn config_panel(&mut self, ctx: &egui::Context) {
        let config = self.app.conn_config();

        egui::SidePanel::left("ConfigPanel")
            .exact_width(SIDEPANEL_WIDTH)
            .min_width(SIDEPANEL_WIDTH)
            .max_width(SIDEPANEL_WIDTH)
            .resizable(false)
            .show(ctx, |ui| {
                ui.add_space(5.0);
                let frame = egui::Frame::window(&ctx.style());
                frame.show(ui, |ui| {
                    ui.horizontal(|ui| {
                        let pos = egui::pos2(
                            ui.next_widget_position().x + STATUS_RADIUS * 1.25,
                            ui.available_height() + STATUS_RADIUS,
                        );
                        let col = egui::Color32::DARK_RED;
                        ui.painter().circle_filled(pos, STATUS_RADIUS, col);
                        ui.add_space(STATUS_RADIUS * 3.0);
                        ui.heading("Connection");
                        ui.add_space(10.0);
                        ui.add(egui::Button::new("Connect").min_size(egui::Vec2::new(86.0, 0.0)));
                    });
                    ui.separator();
    
                    static DEVICES: &[&'static str; 2] = &["COM1", "COM3"];
                    option_dropdown(ui, "Device", DEVICES, &mut config.port.as_str(), 20.0);
                    option_dropdown(ui, "Baud", BAUD_RATES, &mut config.baud_rate, 27.0);
                    ui.separator();

                    option_dropdown(ui, "Data bits", DATA_BITS, &mut config.data_bits, 8.0);
                    option_dropdown(ui, "Parity", PARITIES, &mut config.parity, 24.5);
                    option_dropdown(ui, "Stop bits", STOP_BITS, &mut config.stop_bits, 8.0);
                    option_dropdown(ui, "Flow ctrl", FLOW_CTRLS, &mut config.flow_ctrl, 10.0);
                    ui.separator();

                    option_dropdown(ui, "DTR", &[false, true], &mut config.dtr, 33.0);
                    option_dropdown(ui, "Start mode", START_MODES, &mut config.start_mode, -6.0);
                    if matches!(config.start_mode, StartMode::Delay(_)) {
                        ui.horizontal(|ui| {
                            ui.label("Delay (ms)");
                            ui.add_space(0.0);
                            ui.add(egui::DragValue::new(&mut config.start_delay).clamp_range(0..=100000).max_decimals(0));
                        });
                    }
                    else if matches!(config.start_mode, StartMode::Message(_)) {
                        ui.horizontal(|ui| {
                            ui.label("Message");
                            ui.add_space(7.0);
                            ui.add(egui::TextEdit::singleline(&mut config.start_msg).desired_width(DROPDOWN_WIDTH - ui.style().spacing.item_spacing.x));
                        });
                    }
                });
            });
    }

    fn data_panel(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            let frame = egui::Frame::window(&ctx.style());
            frame.show(ui, |ui| {
                ui.horizontal(|ui| {
                    if ui.button("Save Config").clicked() {}
                    if ui.button("Load Config").clicked() {}
                    ui.add_space(ui.available_width());
                });
            });
    
            let frame = egui::Frame::window(&ctx.style());
            frame.show(ui, |ui| {
                let graph: Vec<[f64; 2]> = vec![[0.0, 0.66], [0.33, 0.33], [0.66, 0.5], [1.0, 0.9]];
                ui.horizontal(|ui| {
                    ui.heading("Plot 1");
                    if ui.button("Reset").clicked() {}
                });
                egui_plot::Plot::new("Plot1")
                    .legend(Legend::default().position(Corner::LeftTop))
                    .height(256.0)
                    .x_axis_formatter(|grid_pt, _, _| format!("{:.2}s", grid_pt.value))
                    .y_axis_formatter(|grid_pt, _, _| format!("{:.2}", grid_pt.value))
                    .y_axis_width(3)
                    .show(ui, |ui| {
                        ui.line(Line::new(PlotPoints::from(graph)).name("line"));
                    });
            });
        });
    }
}

impl eframe::App for SerialMonitorUI {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        self.config_panel(ctx);
        self.data_panel(ctx);
    }
}

fn option_dropdown<T: PartialEq + Clone + Display>(ui: &mut egui::Ui, label: &'static str, options: &[T], value: &mut T, spacing: f32) {
    ui.horizontal(|ui| {
        ui.label(label);
        ui.add_space(spacing);
        egui::ComboBox::new(label, "")
            .selected_text(value.to_string())
            .width(DROPDOWN_WIDTH)
            .show_ui(ui, |ui| {
                for option in options {
                    ui.selectable_value(value, option.clone(), option.to_string());
                }
            });
    });
}
