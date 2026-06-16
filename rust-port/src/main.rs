mod calculator;

use calculator::{BinOp, UnaryOp};
use eframe::egui;
use egui::{Align, Color32, Layout, RichText, Stroke, Vec2};

const WINDOW_WIDTH: f32 = 402.0;
const WINDOW_HEIGHT: f32 = 666.0;
const TOP_BAR_HEIGHT: f32 = 40.0;
const MEMORY_HEIGHT: f32 = 32.0;
const KEYPAD_GAP: f32 = 1.0;

// Windows Calculator 2020 dark palette, sampled from the reference screenshots
const BTN_NUMBER: Color32 = Color32::from_rgb(7, 13, 15); // #070d0f
const BTN_FUNC: Color32 = Color32::from_rgb(20, 37, 46); // #14252e
const BTN_ACCENT: Color32 = Color32::from_rgb(20, 84, 131); // #145483
const BTN_NUMBER_HOVER: Color32 = Color32::from_rgb(25, 41, 50);
const BTN_FUNC_HOVER: Color32 = Color32::from_rgb(40, 60, 72);
const BTN_ACCENT_HOVER: Color32 = Color32::from_rgb(28, 100, 150);
const BTN_NUMBER_ACTIVE: Color32 = Color32::from_rgb(40, 60, 72);
const BTN_FUNC_ACTIVE: Color32 = Color32::from_rgb(55, 75, 90);
const BTN_ACCENT_ACTIVE: Color32 = Color32::from_rgb(35, 110, 160);
const TEXT_WHITE: Color32 = Color32::WHITE;
const TEXT_DIM: Color32 = Color32::from_rgb(128, 144, 144); // disabled memory

struct ButtonStyle {
    fill: Color32,
    hover: Color32,
    active: Color32,
    text: Color32,
}

struct App {
    calc: calculator::Calculator,
}

impl App {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        install_system_font(&cc.egui_ctx);
        setup_macos_acrylic(cc);

        let mut visuals = egui::Visuals::dark();
        visuals.window_fill = Color32::TRANSPARENT;
        visuals.panel_fill = Color32::TRANSPARENT;
        visuals.widgets.inactive.rounding = 0.0.into();
        visuals.widgets.hovered.rounding = 0.0.into();
        visuals.widgets.active.rounding = 0.0.into();
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
            .frame(egui::Frame::none().fill(acrylic_bg()))
            .show(ctx, |ui| {
                ui.spacing_mut().item_spacing = Vec2::new(0.0, 0.0);

                // Top bar: hamburger, Standard, history icon
                ui.allocate_ui_with_layout(
                    Vec2::new(full_size.x, TOP_BAR_HEIGHT),
                    Layout::left_to_right(Align::Center),
                    |ui| {
                        ui.spacing_mut().item_spacing = Vec2::ZERO;
                        top_bar_button(ui, "≡", TOP_BAR_HEIGHT);
                        ui.add_space(8.0);
                        ui.label(
                            RichText::new("Standard")
                                .size(20.0)
                                .strong()
                                .color(TEXT_WHITE),
                        );
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            ui.spacing_mut().item_spacing = Vec2::ZERO;
                            top_bar_button(ui, "↺", TOP_BAR_HEIGHT);
                        });
                    },
                );

                // Display area
                let display_height = (full_size.y * 0.22).max(70.0).min(140.0);
                ui.allocate_ui_with_layout(
                    Vec2::new(full_size.x, display_height),
                    Layout::top_down(Align::RIGHT),
                    |ui| {
                        ui.spacing_mut().item_spacing = Vec2::ZERO;
                        ui.add_space(display_height * 0.10);
                        let expr_size = display_height * 0.25;
                        let result_size = (display_height * 0.58).min(full_size.x * 0.22).max(28.0);
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
                let mem_labels = [
                    ("MC", self.calc.memory != 0.0),
                    ("MR", self.calc.memory != 0.0),
                    ("M+", true),
                    ("M-", true),
                    ("MS", self.calc.memory != 0.0),
                ];
                let mem_width = (full_size.x - (mem_labels.len() as f32 - 1.0) * KEYPAD_GAP)
                    / mem_labels.len() as f32;
                ui.allocate_ui_with_layout(
                    Vec2::new(full_size.x, MEMORY_HEIGHT),
                    Layout::left_to_right(Align::Center),
                    |ui| {
                        ui.spacing_mut().item_spacing = Vec2::new(KEYPAD_GAP, 0.0);
                        for (label, enabled) in mem_labels {
                            let text_color = if enabled { TEXT_WHITE } else { TEXT_DIM };
                            let style = ButtonStyle {
                                fill: acrylic_bg(),
                                hover: Color32::from_rgb(45, 70, 85),
                                active: Color32::from_rgb(60, 90, 110),
                                text: text_color,
                            };
                            let font_size = MEMORY_HEIGHT * 0.45;
                            if calc_button(
                                ui,
                                label,
                                Vec2::new(mem_width, MEMORY_HEIGHT),
                                &style,
                                font_size,
                            )
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
                let keypad_height = full_size.y - TOP_BAR_HEIGHT - display_height - MEMORY_HEIGHT;
                let show_functions = keypad_height >= 200.0;
                let rows: Vec<[&str; 4]> = if show_functions {
                    vec![
                        ["%", "CE", "C", "⌫"],
                        ["1/x", "x²", "√x", "÷"],
                        ["7", "8", "9", "×"],
                        ["4", "5", "6", "−"],
                        ["1", "2", "3", "+"],
                        ["+/-", "0", ".", "="],
                    ]
                } else {
                    vec![
                        ["CE", "C", "⌫", "÷"],
                        ["7", "8", "9", "×"],
                        ["4", "5", "6", "−"],
                        ["1", "2", "3", "+"],
                        ["+/-", "0", ".", "="],
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
                                } else if row_idx == 0 && show_functions && col_idx == 1 {
                                    clear_label(&self.calc)
                                } else {
                                    label
                                };

                                let style = button_style(label, &self.calc);
                                let font_size = font_size_for(label, btn_width, row_height);

                                let resp = calc_button(
                                    ui,
                                    label,
                                    Vec2::new(btn_width, row_height),
                                    &style,
                                    font_size,
                                );

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

fn install_system_font(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();
    let candidates = [
        "/System/Library/Fonts/SFNS.ttf",
        "/System/Library/Fonts/Helvetica.ttc",
        "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
    ];

    for path in candidates {
        let Ok(bytes) = std::fs::read(path) else {
            continue;
        };

        fonts
            .font_data
            .insert("system-ui".to_owned(), egui::FontData::from_owned(bytes));
        fonts
            .families
            .entry(egui::FontFamily::Proportional)
            .or_default()
            .insert(0, "system-ui".to_owned());
        fonts
            .families
            .entry(egui::FontFamily::Monospace)
            .or_default()
            .insert(0, "system-ui".to_owned());
        ctx.set_fonts(fonts);
        return;
    }
}

fn acrylic_bg() -> Color32 {
    Color32::from_rgba_unmultiplied(34, 60, 76, 218)
}

#[cfg(target_os = "macos")]
fn setup_macos_acrylic(window: &impl raw_window_handle::HasWindowHandle) {
    use objc2::runtime::AnyObject;
    use objc2::{class, msg_send};
    use objc2_foundation::CGRect;
    use raw_window_handle::RawWindowHandle;

    let Ok(handle) = window.window_handle() else {
        return;
    };

    let RawWindowHandle::AppKit(handle) = handle.as_raw() else {
        return;
    };

    unsafe {
        let ns_view = handle.ns_view.as_ptr() as *mut AnyObject;
        let ns_window: *mut AnyObject = msg_send![ns_view, window];
        if ns_window.is_null() {
            return;
        }
        let content_view: *mut AnyObject = msg_send![ns_window, contentView];
        if content_view.is_null() {
            return;
        }

        let bounds: CGRect = msg_send![content_view, bounds];
        let effect_view: *mut AnyObject = msg_send![class!(NSVisualEffectView), alloc];
        let effect_view: *mut AnyObject = msg_send![effect_view, initWithFrame: bounds];
        if effect_view.is_null() {
            return;
        }

        let _: () = msg_send![effect_view, setMaterial: 17isize]; // NSVisualEffectMaterialUnderWindowBackground
        let _: () = msg_send![effect_view, setBlendingMode: 0isize]; // NSVisualEffectBlendingModeBehindWindow
        let _: () = msg_send![effect_view, setState: 1isize]; // NSVisualEffectStateActive
        let _: () = msg_send![effect_view, setAutoresizingMask: 18usize]; // width + height sizable
        let _: () = msg_send![content_view, addSubview: effect_view positioned: -1isize relativeTo: std::ptr::null_mut::<AnyObject>()];
    }
}

#[cfg(not(target_os = "macos"))]
fn setup_macos_acrylic(_window: &impl raw_window_handle::HasWindowHandle) {}

fn top_bar_button(ui: &mut egui::Ui, label: &str, height: f32) -> egui::Response {
    ui.add_sized(
        Vec2::new(height, height),
        egui::Button::new(RichText::new(label).size(20.0).color(TEXT_WHITE))
            .fill(acrylic_bg())
            .frame(false),
    )
}

fn calc_button(
    ui: &mut egui::Ui,
    label: &str,
    size: Vec2,
    style: &ButtonStyle,
    font_size: f32,
) -> egui::Response {
    let v = ui.visuals_mut();
    let old_hover = v.widgets.hovered.bg_fill;
    let old_active = v.widgets.active.bg_fill;
    v.widgets.hovered.bg_fill = style.hover;
    v.widgets.active.bg_fill = style.active;

    let resp = ui.add_sized(
        size,
        egui::Button::new(RichText::new(label).size(font_size).color(style.text))
            .fill(style.fill)
            .rounding(0.0)
            .stroke(Stroke::NONE),
    );

    let v = ui.visuals_mut();
    v.widgets.hovered.bg_fill = old_hover;
    v.widgets.active.bg_fill = old_active;

    resp
}

fn clear_label(calc: &calculator::Calculator) -> &'static str {
    if calc.error {
        "C"
    } else {
        "CE"
    }
}

fn button_style<'a>(label: &'a str, _calc: &calculator::Calculator) -> ButtonStyle {
    if label == "=" {
        ButtonStyle {
            fill: BTN_ACCENT,
            hover: BTN_ACCENT_HOVER,
            active: BTN_ACCENT_ACTIVE,
            text: TEXT_WHITE,
        }
    } else if is_number_button(label) {
        ButtonStyle {
            fill: BTN_NUMBER,
            hover: BTN_NUMBER_HOVER,
            active: BTN_NUMBER_ACTIVE,
            text: TEXT_WHITE,
        }
    } else {
        ButtonStyle {
            fill: BTN_FUNC,
            hover: BTN_FUNC_HOVER,
            active: BTN_FUNC_ACTIVE,
            text: TEXT_WHITE,
        }
    }
}

fn is_number_button(label: &str) -> bool {
    matches!(
        label,
        "0" | "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9" | "." | "+/-"
    )
}

fn font_size_for(label: &str, btn_width: f32, row_height: f32) -> f32 {
    let base = (btn_width * 0.35).min(row_height * 0.45).clamp(12.0, 32.0);
    let factor = match label {
        "0" | "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9" | "." | "+/-" => 1.0,
        "+" | "−" | "×" | "÷" | "=" => 0.9,
        "%" | "CE" | "C" | "⌫" | "1/x" | "x²" | "√x" => 0.75,
        "MC" | "MR" | "M+" | "M-" | "MS" => 0.55,
        _ => 0.85,
    };
    (base * factor).clamp(10.0, 32.0)
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
        "+/-" => calc.negate(),
        "+" => calc.set_op(BinOp::Add),
        "−" => calc.set_op(BinOp::Sub),
        "×" => calc.set_op(BinOp::Mul),
        "÷" => calc.set_op(BinOp::Div),
        "=" => calc.evaluate(),
        "%" => calc.unary_op(UnaryOp::Percent),
        "√x" => calc.unary_op(UnaryOp::Sqrt),
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
            .with_resizable(true)
            .with_transparent(true),
        ..Default::default()
    };
    eframe::run_native(
        "Calculator",
        options,
        Box::new(|cc| Ok(Box::new(App::new(cc)))),
    )
}
