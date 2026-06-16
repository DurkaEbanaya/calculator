mod calculator;

use calculator::{BinOp, UnaryOp};
use eframe::egui;
use egui::{Color32, RichText, Vec2};

const WINDOW_WIDTH: f32 = 360.0;
const WINDOW_HEIGHT: f32 = 600.0;
const BG: Color32 = Color32::from_rgb(31, 31, 31);
const BUTTON_BG: Color32 = Color32::from_rgb(45, 45, 45);
const BUTTON_HOVER: Color32 = Color32::from_rgb(60, 60, 60);
const BUTTON_ACTIVE: Color32 = Color32::from_rgb(80, 80, 80);
const TEXT_WHITE: Color32 = Color32::WHITE;
const TEXT_DIM: Color32 = Color32::from_rgb(160, 160, 160);

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
        cc.egui_ctx.set_visuals(visuals);
        Self {
            calc: calculator::Calculator::new(),
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default()
            .frame(egui::Frame::none().fill(BG))
            .show(ctx, |ui| {
                ui.spacing_mut().item_spacing = Vec2::new(2.0, 2.0);
                ui.visuals_mut().widgets.inactive.rounding = 4.0.into();
                ui.visuals_mut().widgets.hovered.rounding = 4.0.into();
                ui.visuals_mut().widgets.active.rounding = 4.0.into();

                // Title / mode bar
                ui.horizontal(|ui| {
                    ui.visuals_mut().override_text_color = Some(TEXT_WHITE);
                    menu_button(ui, "≡");
                    ui.add_space(8.0);
                    ui.label(RichText::new("Standard").size(18.0).strong());
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        menu_button(ui, "↻");
                    });
                });
                ui.add_space(8.0);

                // Display area
                let available = ui.available_width();
                ui.allocate_ui_with_layout(
                    Vec2::new(available, 90.0),
                    egui::Layout::top_down(egui::Align::RIGHT),
                    |ui| {
                        ui.label(
                            RichText::new(&self.calc.expression)
                                .size(14.0)
                                .color(TEXT_DIM),
                        );
                        ui.label(
                            RichText::new(&self.calc.display)
                                .size(48.0)
                                .strong()
                                .color(TEXT_WHITE),
                        );
                    },
                );
                ui.add_space(8.0);

                // Memory buttons row
                let mem = ["MC", "MR", "M+", "M-", "MS"];
                let mem_width = (ui.available_width() - 4.0 * 2.0) / 5.0;
                let mem_height = 34.0;
                ui.horizontal(|ui| {
                    for label in mem {
                        if calc_button(ui, label, Vec2::new(mem_width, mem_height), BUTTON_BG, TEXT_DIM)
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
                });
                ui.add_space(4.0);

                // Main keypad grid
                let rows: [[&str; 4]; 6] = [
                    ["%", "√", "x²", "1/x"],
                    ["CE", "C", "⌫", "÷"],
                    ["7", "8", "9", "×"],
                    ["4", "5", "6", "-"],
                    ["1", "2", "3", "+"],
                    ["±", "0", ".", "="],
                ];
                let gap = 2.0;
                let btn_width = (ui.available_width() - 3.0 * gap) / 4.0;
                let btn_height = 58.0;
                for row in rows {
                    ui.horizontal(|ui| {
                        for label in row {
                            let (text, color) = label_color(label);
                            if calc_button(ui, text, Vec2::new(btn_width, btn_height), color, TEXT_WHITE)
                                .clicked()
                            {
                                handle_input(&mut self.calc, label);
                            }
                        }
                    });
                }
            });
    }
}

fn menu_button(ui: &mut egui::Ui, label: &str) -> egui::Response {
    ui.add(
        egui::Button::new(RichText::new(label).size(20.0).color(TEXT_WHITE))
            .fill(BG)
            .frame(false),
    )
}

fn calc_button(
    ui: &mut egui::Ui,
    label: &str,
    size: Vec2,
    bg: Color32,
    text: Color32,
) -> egui::Response {
    ui.add_sized(
        size,
        egui::Button::new(RichText::new(label).size(18.0).color(text))
            .fill(bg)
            .rounding(4.0),
    )
}

fn label_color(label: &str) -> (&str, Color32) {
    match label {
        "=" => ("=", Color32::from_rgb(0, 90, 158)),
        _ => (label, BUTTON_BG),
    }
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
            .with_min_inner_size([320.0, 480.0])
            .with_resizable(true),
        ..Default::default()
    };
    eframe::run_native(
        "Calculator",
        options,
        Box::new(|cc| Ok(Box::new(App::new(cc)))),
    )
}
