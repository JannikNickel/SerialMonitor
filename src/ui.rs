use crate::app::SerialMonitorApp;
use crate::data::PlotMode;
use crate::serial_reader::{FlowCtrl, Parity, StartMode};
use eframe::egui;
use egui::emath::Numeric;
use egui::{Align, Align2, Color32, Layout, Ui};
use egui_plot::{Corner, Legend, Line, PlotPoints};
use std::fmt::Display;
use std::ops::RangeInclusive;
use std::time::{Duration, Instant};

const SIDEPANEL_WIDTH: f32 = 225.0;
const DROPDOWN_WIDTH: f32 = 150.0;
const STATUS_RADIUS: f32 = 6.0;

const BAUD_RATES: &[u32] = &[
    300, 600, 750, 1200, 2400, 4800, 9600, 19200, 31250, 38400, 57600, 74880, 115200, 230400,
    250000, 460800, 500000, 921600, 1000000, 2000000,
];
const DATA_BITS: &[u8] = &[5, 6, 7, 8, 9];
const PARITIES: &[Parity] = &[Parity::None, Parity::Odd, Parity::Even];
const STOP_BITS: &[u8] = &[1, 2];
const FLOW_CTRLS: &[FlowCtrl] = &[FlowCtrl::None, FlowCtrl::Software, FlowCtrl::Hardware];
const START_MODES: &[StartMode] = &[
    StartMode::Immediate,
    StartMode::Delay(Duration::ZERO),
    StartMode::Message(String::new()),
];
const PLOT_MODES: &[PlotMode] = &[PlotMode::Continous, PlotMode::Redraw];

struct Notification {
    pub start: Instant,
    pub duration: Duration,
    pub text: String
}

impl Notification {
    pub fn new(text: &str, duration: Duration) -> Self {
        Self {
            start: Instant::now(),
            duration: duration,
            text: text.to_owned()
        }
    }
}

pub struct SerialMonitorUI {
    notification: Option<Notification>
}

impl SerialMonitorUI {
    pub fn new(_context: &eframe::CreationContext<'_>) -> Self {
        Self {
            notification: None
        }
    }

    pub fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame, app: &mut SerialMonitorApp) {
        self.config_panel(ctx, app);
        self.data_panel(ctx);
        self.notification(ctx);
    }

    fn config_panel(&mut self, ctx: &egui::Context, app: &mut SerialMonitorApp) {
        egui::SidePanel::left("ConnPanel")
            .exact_width(SIDEPANEL_WIDTH)
            .min_width(SIDEPANEL_WIDTH)
            .max_width(SIDEPANEL_WIDTH)
            .resizable(false)
            .show(ctx, |ui| {
                self.conn_panel(ctx, ui, app);
                self.plot_panel(ctx, ui, app);
            });
    }

    fn conn_panel(&mut self, ctx: &egui::Context, ui: &mut Ui, app: &mut SerialMonitorApp) {
        ui.add_space(5.0);
        let frame = egui::Frame::window(&ctx.style());
        frame.show(ui, |ui| {
            ui.horizontal(|ui| {
                let pos = egui::pos2(
                    ui.next_widget_position().x + STATUS_RADIUS * 1.25,
                    ui.available_height() + STATUS_RADIUS * 0.85,
                );
                let col = match app.is_connected() {
                    true => egui::Color32::DARK_GREEN,
                    false => egui::Color32::DARK_RED
                };
                ui.painter().circle_filled(pos, STATUS_RADIUS, col);
                ui.add_space(STATUS_RADIUS * 3.0);
                ui.heading("Connection");
                ui.add_space(10.0);
                let connect_btn_text = match app.is_connected() {
                    true => "Disconnect",
                    false => "Connect"
                };
                let connect_resp = ui.add_enabled(
                    app.can_connect(),
                    egui::Button::new(connect_btn_text).min_size(egui::Vec2::new(86.0, 0.0)));
                if connect_resp.clicked() {
                    if !app.is_connected() {
                        if let Err(e) = app.connect_current() {
                            self.notification = Some(Notification::new(format!("Could not connect! ({})", e.to_string()).as_str(), Duration::from_secs(5)));
                        }
                    } else {
                        app.disconnect_current();
                    }
                }
            });
            ui.separator();

            ui.set_enabled(!app.is_connected());
            let devices = app.available_devices();
            let config = app.conn_config();
            option_dropdown(ui, "Device", devices.as_slice(), &mut config.port, 20.0);
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
                drag_value(ui, "Delay (ms)", &mut config.start_delay, 0.0, 0..=100000, 0, "ms");
            } else if matches!(config.start_mode, StartMode::Message(_)) {
                ui.horizontal(|ui| {
                    ui.label("Message");
                    ui.add_space(7.0);
                    ui.add(egui::TextEdit::singleline(&mut config.start_msg).desired_width(DROPDOWN_WIDTH - ui.style().spacing.item_spacing.x));
                });
            }
        });
    }

    fn plot_panel(&mut self, ctx: &egui::Context, ui: &mut Ui, app: &mut SerialMonitorApp) {
        ui.add_space(5.0);
        let frame = egui::Frame::window(&ctx.style());
        frame.show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.heading("Plot Settings");
                ui.add_space(ui.available_width());
            });
            ui.separator();

            let config = app.plot_config();
            option_dropdown(ui, "Mode", PLOT_MODES, &mut config.mode, 24.0);
            drag_value(ui, "Window (s)", &mut config.window, -3.5, 0.0..=100000.0, 2, "s");
        });
    }

    fn data_panel(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            let frame = egui::Frame::window(&ctx.style());
            frame.show(ui, |ui| {
                ui.horizontal(|ui| {
                    if ui.button("Add Plot").clicked() {}
                    if ui.button("Add Console").clicked() {}
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

    fn notification(&mut self, ctx: &egui::Context) {
        if let Some(notification) = &self.notification {
            if notification.start.elapsed() > notification.duration {
                self.notification = None;
                return;
            }

            let frame = egui::Frame::popup(&ctx.style()).fill(Color32::from_rgb(105, 25, 19));
            egui::Window::new("")
                .fixed_pos([ctx.available_rect().center().x, 75.0])
                .collapsible(false)
                .movable(false)
                .resizable(false)
                .frame(frame)
                .title_bar(false)
                .pivot(Align2::CENTER_CENTER)
                .default_pos(egui::pos2(0.0, 0.0))
                .show(ctx, |ui| {
                    ui.heading(notification.text.as_str());
                });
        }
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

fn drag_value<T: Numeric>(ui: &mut egui::Ui, label: &'static str, value: &mut T, spacing: f32, range: RangeInclusive<T>, decimals: usize, suffix: &str) {
    ui.horizontal_top(|ui| {
        ui.label(label);
        ui.add_space(spacing);
        ui.with_layout(Layout::top_down(Align::LEFT), |ui| {
            ui.add(egui::DragValue::new(value)
                .clamp_range(range)
                .fixed_decimals(decimals)
                .suffix(suffix));
        });
    });
}
