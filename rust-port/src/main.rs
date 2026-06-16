mod calculator;

use calculator::{BinOp, UnaryOp};
use eframe::egui;
use egui::{Align, Color32, Layout, RichText, Stroke, Vec2};

const WINDOW_WIDTH: f32 = 360.0;
const WINDOW_HEIGHT: f32 = 600.0;
const TOP_BAR_HEIGHT: f32 = 32.0;
const MEMORY_HEIGHT: f32 = 34.0;
const KEYPAD_GAP: f32 = 1.0;

const BG: Color32 = Color32::from_rgb(0, 0, 0);
const BUTTON_BG: Color32 = Color32::from_rgb(23, 23, 23);
const BUTTON_HOVER: Color32 = Color32::from_rgb(45, 45, 45);
const BUTTON_ACTIVE: Color32 = Color32::from_rgb(60, 60, 60);
const ACCENT: Color32 = Color32::from_rgb(0, 120, 212);
const ACCENT_HOVER: Color32 = Color32::from_rgb(20, 140, 230);
const ACCENT_ACTIVE: Color32 = Color32::from_rgb(0, 100, 180);
const TEXT_WHITE: Color32 = Color32::WHITE;
const TEXT_DIM: Color32 = Color32::from_rgb(130, 130, 130);

struct App {
    calc: calculator::Calculator,
}

impl App {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let mut visuals = egui::Visuals::dark();
        visuals.window_fill = BG;
        visuals.panel_fill = BG;
        visuals.widgets.inactive.bg_fill = BUTTON_BG;
        visuals.widgets.hovered.bg_fill = BUTTON_HOVER;
        visuals.widgets.active.bg_fill = BUTTON_ACTIVE;
        visuals.widgets.inactive.rounding = 0.0.into();
        visuals.widgets.hovered.rounding = 0.0.into();
        visuals.widgets.active.rounding = 0.0.into();
        visuals.widgets.inactive.bg_stroke = Stroke::NONE;
        visuals.widgets.hovered.bg_stroke = Stroke::NONE;
        visuals.widgets.active.bg_stroke = Stroke::NONE;
        visuals.window_rounding = 0.0.into();
        cc.egui_ctx.set_visuals(visuals);
        Self {
            calc: calculator::Calculator::new(),
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let full_size = ctx.screen_rect().size();

        egui::CentralPanel::default()
            .frame(egui::Frame::none().fill(BG))
            .show(ctx, |ui| {
                ui.spacing_mut().item_spacing = Vec2::new(0.0, 0.0);

                // Top bar: hamburger, Standard, history
                ui.allocate_ui_with_layout(
                    Vec2::new(full_size.x, TOP_BAR_HEIGHT),
                    Layout::left_to_right(Align::Center),
                    |ui| {
                        ui.spacing_mut().item_spacing = Vec2::ZERO;
                        top_bar_button(ui, "≡", TOP_BAR_HEIGHT);
                        ui.add_space(8.0);
                        ui.label(
                            RichText::new("Standard")
                                .size(18.0)
                                .strong()
                                .color(TEXT_WHITE),
                        );
                        ui.with_layout(
                            Layout::right_to_left(Align::Center),
                            |ui| {
                                ui.spacing_mut().item_spacing = Vec2::ZERO;
                                top_bar_button(ui, "↻", TOP_BAR_HEIGHT);
                            },
                        );
                    },
                );

                // Display area
                let display_height = (full_size.y * 0.22).max(70.0).min(140.0);
                ui.allocate_ui_with_layout(
                    Vec2::new(full_size.x, display_height),
                    Layout::top_down(Align::RIGHT),
                    |ui| {
                        ui.spacing_mut().item_spacing = Vec2::ZERO;
                        ui.add_space(display_height * 0.15);
                        let expr_size = display_height * 0.22;
                        let result_size = (display_height * 0.55)
                            .min(full_size.x * 0.22)
                            .max(28.0);
                        ui.label(
                            RichText::new(&self.calc.expression)
                                .size(expr_size)
                                .color(TEXT_DIM),
                        );
                        ui.label(
                            RichText::new(&self.calc.display)
                                .size(result_size)
                                .strong()
                                .color(TEXT_WHITE),
                        );
                    },
                );

                // Memory row
                let mem_labels = [("MC", self.calc.memory != 0.0), ("MR", self.calc.memory != 0.0), ("M+", true), ("M-", true), ("MS", true)];
                let mem_width = (full_size.x - (mem_labels.len() as f32 - 1.0) * KEYPAD_GAP) / mem_labels.len() as f32;
                ui.allocate_ui_with_layout(
                    Vec2::new(full_size.x, MEMORY_HEIGHT),
                    Layout::left_to_right(Align::Center),
                    |ui| {
                        ui.spacing_mut().item_spacing = Vec2::new(KEYPAD_GAP, 0.0);
                        for (label, enabled) in mem_labels {
                            let text_color = if enabled { TEXT_WHITE } else { TEXT_DIM };
                            if calc_button(ui, label, Vec2::new(mem_width, MEMORY_HEIGHT), BUTTON_BG, text_color, MEMORY_HEIGHT * 0.45)
                                .clicked()
                            {
                                match label {
                                    "MC" => self.calc.memory_clear(),
                                    "MR" => self.calc.memory_recall(),
                                    "M+" => self.calc.memory_add(),
                                    "M-" => self.calc.memory_sub(),
                                    "MS" => self.calc.memory_store(),
                                    _ => {}
                                }
                            }
                        }
                    },
                );

                // Keypad
                let keypad_height = full_size.y - TOP_BAR_HEIGHT - display_height - MEMORY_HEIGHT - KEYPAD_GAP;
                let show_functions = keypad_height >= 200.0;
                let rows: Vec<[&str; 4]> = if show_functions {
                    vec![
                        ["%", "√", "x²", "1/x"],
                        ["CE", "C", "⌫", "÷"],
                        ["7", "8", "9", "×"],
                        ["4", "5", "6", "-"],
                        ["1", "2", "3", "+"],
                        ["±", "0", ".", "="],
                    ]
                } else {
                    vec![
                        ["CE", "C", "⌫", "÷"],
                        ["7", "8", "9", "×"],
                        ["4", "5", "6", "-"],
                        ["1", "2", "3", "+"],
                        ["±", "0", ".", "="],
                    ]
                };
                let num_rows = rows.len() as f32;
                let row_height = (keypad_height - (num_rows - 1.0) * KEYPAD_GAP) / num_rows;
                let btn_width = (full_size.x - 3.0 * KEYPAD_GAP) / 4.0;

                for (row_idx, row) in rows.iter().enumerate() {
                    ui.allocate_ui_with_layout(
                        Vec2::new(full_size.x, row_height),
                        Layout::left_to_right(Align::Center),
                        |ui| {
                            ui.spacing_mut().item_spacing = Vec2::new(KEYPAD_GAP, 0.0);
                            for (col_idx, &label) in row.iter().enumerate() {
                                let label = if row_idx == 0 && !show_functions && col_idx == 0 {
                                    clear_label(&self.calc)
                                } else if row_idx == 1 && show_functions && col_idx == 0 {
                                    clear_label(&self.calc)
                                } else {
                                    label
                                };

                                let (text, bg, text_color) = button_style(label, &self.calc);
                                let font_size = (btn_width * 0.35)
                                    .min(row_height * 0.45)
                                    .clamp(12.0, 28.0);

                                let mut hover_restore = None;
                                let mut active_restore = None;
                                if bg == ACCENT {
                                    let v = ui.visuals_mut();
                                    hover_restore = Some(v.widgets.hovered.bg_fill);
                                    active_restore = Some(v.widgets.active.bg_fill);
                                    v.widgets.hovered.bg_fill = ACCENT_HOVER;
                                    v.widgets.active.bg_fill = ACCENT_ACTIVE;
                                }

                                let resp = calc_button(ui, text, Vec2::new(btn_width, row_height), bg, text_color, font_size);

                                if let (Some(old_h), Some(old_a)) = (hover_restore, active_restore) {
                                    let v = ui.visuals_mut();
                                    v.widgets.hovered.bg_fill = old_h;
                                    v.widgets.active.bg_fill = old_a;
                                }

                                if resp.clicked() {
                                    handle_input(&mut self.calc, label);
                                }
                            }
                        },
                    );
                }
            });
    }
}

fn top_bar_button(ui: &mut egui::Ui, label: &str, height: f32) -> egui::Response {
    ui.add_sized(
        Vec2::new(height, height),
        egui::Button::new(RichText::new(label).size(18.0).color(TEXT_WHITE))
            .fill(BG)
            .frame(false),
    )
}

fn calc_button(
    ui: &mut egui::Ui,
    label: &str,
    size: Vec2,
    bg: Color32,
    text_color: Color32,
    font_size: f32,
) -> egui::Response {
    ui.add_sized(
        size,
        egui::Button::new(RichText::new(label).size(font_size).color(text_color))
            .fill(bg)
            .rounding(0.0)
            .stroke(Stroke::NONE),
    )
}

fn clear_label(calc: &calculator::Calculator) -> &'static str {
    if calc.error || calc.display == "0" || calc.fresh {
        "C"
    } else {
        "CE"
    }
}

fn button_style<'a>(label: &'a str, _calc: &calculator::Calculator) -> (&'a str, Color32, Color32) {
    if label == "=" {
        return ("=", ACCENT, TEXT_WHITE);
    }
    (label, BUTTON_BG, TEXT_WHITE)
}

fn handle_input(calc: &mut calculator::Calculator, label: &str) {
    if let Ok(d) = label.parse::<u8>() {
        if d <= 9 {
            calc.input_digit(d);
            return;
        }
    }
    match label {
        "." => calc.input_decimal(),
        "±" => calc.negate(),
        "+" => calc.set_op(BinOp::Add),
        "-" => calc.set_op(BinOp::Sub),
        "×" => calc.set_op(BinOp::Mul),
        "÷" => calc.set_op(BinOp::Div),
        "=" => calc.evaluate(),
        "%" => calc.unary_op(UnaryOp::Percent),
        "√" => calc.unary_op(UnaryOp::Sqrt),
        "x²" => calc.unary_op(UnaryOp::Square),
        "1/x" => calc.unary_op(UnaryOp::Reciprocal),
        "CE" => calc.clear_entry(),
        "C" => calc.clear_all(),
        "⌫" => calc.backspace(),
        _ => {}
    }
}

fn main() -> eframe::Result {
    env_logger::init();
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([WINDOW_WIDTH, WINDOW_HEIGHT])
            .with_min_inner_size([240.0, 320.0])
            .with_resizable(true),
        ..Default::default()
    };
    eframe::run_native(
        "Calculator",
        options,
        Box::new(|cc| Ok(Box::new(App::new(cc)))),
    )
}
