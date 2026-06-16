mod calculator;

use calculator::{BinOp, UnaryOp};
use eframe::egui;
use egui::{Align, Color32, Layout, RichText, Stroke, Vec2};

const WINDOW_WIDTH: f32 = 402.0;
const WINDOW_HEIGHT: f32 = 666.0;
const TOP_BAR_HEIGHT: f32 = 40.0;
const MEMORY_HEIGHT: f32 = 32.0;
const KEYPAD_GAP: f32 = 1.0;

// Windows Calculator 2020 dark Acrylic palette: number buttons are dense,
// surrounding chrome lets the system blur/material show through.
const BTN_NUMBER: Color32 = Color32::from_rgb(5, 9, 10);
const BTN_NUMBER_HOVER: Color32 = Color32::from_rgb(24, 28, 30);
const BTN_NUMBER_ACTIVE: Color32 = Color32::from_rgb(44, 48, 50);
const TEXT_WHITE: Color32 = Color32::WHITE;
const TEXT_DIM: Color32 = Color32::from_rgb(128, 144, 144); // disabled memory

struct ButtonStyle {
    fill: Color32,
    hover: Color32,
    active: Color32,
    text: Color32,
}

struct AcrylicSettings {
    transparency: f32,
    intensity: f32,
    brightness: f32,
    blur: f32,
}

struct App {
    calc: calculator::Calculator,
    acrylic_view: usize,
    settings_open: bool,
    acrylic: AcrylicSettings,
}

impl App {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        install_system_font(&cc.egui_ctx);

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
            acrylic_view: 0,
            settings_open: false,
            acrylic: AcrylicSettings {
                transparency: 0.45,
                intensity: 0.70,
                brightness: 0.75,
                blur: 0.45,
            },
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        self.acrylic_view = setup_acrylic(frame, &self.acrylic, self.acrylic_view);

        let full_size = ctx.screen_rect().size();

        egui::CentralPanel::default()
            .frame(egui::Frame::none().fill(Color32::TRANSPARENT))
            .show(ctx, |ui| {
                ui.spacing_mut().item_spacing = Vec2::new(0.0, 0.0);
                let acrylic_rect = ui.max_rect();
                ui.painter()
                    .rect_filled(acrylic_rect, 0.0, acrylic_tint(&self.acrylic));

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
                            if top_bar_button(ui, "⚙", TOP_BAR_HEIGHT).clicked() {
                                self.settings_open = !self.settings_open;
                            }
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
                let mem_width = full_size.x / mem_labels.len() as f32;
                ui.allocate_ui_with_layout(
                    Vec2::new(full_size.x, MEMORY_HEIGHT),
                    Layout::left_to_right(Align::Center),
                    |ui| {
                        ui.spacing_mut().item_spacing = Vec2::ZERO;
                        for (label, enabled) in mem_labels {
                            let text_color = if enabled { TEXT_WHITE } else { TEXT_DIM };
                            let style = ButtonStyle {
                                fill: chrome_bg(&self.acrylic),
                                hover: chrome_hover_bg(&self.acrylic),
                                active: chrome_active_bg(&self.acrylic),
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

                                let style = button_style(label, &self.calc, &self.acrylic);
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

        if self.settings_open {
            show_acrylic_settings(ctx, &mut self.settings_open, &mut self.acrylic);
        }
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

fn acrylic_tint(settings: &AcrylicSettings) -> Color32 {
    let opacity = 1.0 - settings.transparency;
    let value =
        (24.0 * settings.brightness * (0.75 + settings.intensity * 0.10)).clamp(0.0, 100.0) as u8;
    let alpha = (10.0 + 145.0 * opacity + 28.0 * settings.intensity).clamp(0.0, 215.0) as u8;
    Color32::from_rgba_unmultiplied(
        value,
        value.saturating_add(5),
        value.saturating_add(8),
        alpha,
    )
}

fn chrome_alpha(settings: &AcrylicSettings, base: f32) -> u8 {
    let opacity = 1.0 - settings.transparency;
    (base * (0.18 + opacity * 0.46 + settings.intensity * 0.16)).clamp(0.0, 255.0) as u8
}

fn chrome_bg(settings: &AcrylicSettings) -> Color32 {
    let b =
        (30.0 * settings.brightness * (0.80 + settings.intensity * 0.10)).clamp(3.0, 120.0) as u8;
    Color32::from_rgba_unmultiplied(b / 2, b, b + 4, chrome_alpha(settings, 150.0))
}

fn chrome_hover_bg(settings: &AcrylicSettings) -> Color32 {
    let b =
        (70.0 * settings.brightness * (0.85 + settings.intensity * 0.10)).clamp(16.0, 190.0) as u8;
    Color32::from_rgba_unmultiplied(b - 8, b, b + 2, chrome_alpha(settings, 190.0))
}

fn chrome_active_bg(settings: &AcrylicSettings) -> Color32 {
    let b =
        (96.0 * settings.brightness * (0.85 + settings.intensity * 0.10)).clamp(26.0, 220.0) as u8;
    Color32::from_rgba_unmultiplied(b - 10, b, b + 2, chrome_alpha(settings, 220.0))
}

fn show_acrylic_settings(
    ctx: &egui::Context,
    settings_open: &mut bool,
    settings: &mut AcrylicSettings,
) {
    egui::Window::new("Acrylic")
        .open(settings_open)
        .title_bar(false)
        .resizable(false)
        .collapsible(false)
        .default_pos(egui::pos2(170.0, 42.0))
        .frame(
            egui::Frame::none()
                .fill(Color32::from_rgba_unmultiplied(20, 26, 28, 235))
                .stroke(Stroke::new(
                    1.0,
                    Color32::from_rgba_unmultiplied(255, 255, 255, 40),
                ))
                .inner_margin(egui::Margin::same(12.0))
                .rounding(0.0),
        )
        .show(ctx, |ui| {
            ui.set_width(250.0);
            ui.label(RichText::new("Acrylic settings").strong().color(TEXT_WHITE));
            ui.add_space(6.0);
            ui.label(RichText::new("Transparency").color(TEXT_DIM));
            ui.add(egui::Slider::new(&mut settings.transparency, 0.0..=1.0).show_value(true));
            ui.label(RichText::new("Intensity").color(TEXT_DIM));
            ui.add(egui::Slider::new(&mut settings.intensity, 0.0..=3.0).show_value(true));
            ui.label(RichText::new("Brightness").color(TEXT_DIM));
            ui.add(egui::Slider::new(&mut settings.brightness, 0.20..=2.40).show_value(true));
            ui.label(RichText::new("Blur").color(TEXT_DIM));
            ui.add(egui::Slider::new(&mut settings.blur, 0.0..=1.0).show_value(true));
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                if ui.button("Subtle").clicked() {
                    *settings = AcrylicSettings {
                        transparency: 0.70,
                        intensity: 0.30,
                        brightness: 0.70,
                        blur: 0.25,
                    };
                }
                if ui.button("Glass").clicked() {
                    *settings = AcrylicSettings {
                        transparency: 0.74,
                        intensity: 3.0,
                        brightness: 1.10,
                        blur: 1.0,
                    };
                }
            });
        });
}

#[cfg(target_os = "macos")]
fn setup_acrylic(frame: &eframe::Frame, settings: &AcrylicSettings, acrylic_view: usize) -> usize {
    use objc2::runtime::AnyObject;
    use objc2::{class, msg_send};
    use objc2_foundation::CGRect;
    use raw_window_handle::{HasWindowHandle, RawWindowHandle};

    const NS_VIEW_WIDTH_SIZABLE: usize = 2;
    const NS_VIEW_HEIGHT_SIZABLE: usize = 16;
    const NS_WINDOW_BELOW: isize = -1;
    const NS_VISUAL_EFFECT_BLENDING_MODE_BEHIND_WINDOW: isize = 0;
    const NS_VISUAL_EFFECT_STATE_ACTIVE: isize = 1;

    let material = macos_material_for_blur(settings.blur);

    if acrylic_view != 0 {
        unsafe {
            let effect_view = acrylic_view as *mut AnyObject;
            let _: () = msg_send![effect_view, setMaterial: material];
            let _: () = msg_send![effect_view, setState: NS_VISUAL_EFFECT_STATE_ACTIVE];
        }
        return acrylic_view;
    }

    let Ok(handle) = frame.window_handle() else {
        return 0;
    };

    let RawWindowHandle::AppKit(handle) = handle.as_raw() else {
        return 0;
    };

    unsafe {
        let egui_view = handle.ns_view.as_ptr() as *mut AnyObject;
        if egui_view.is_null() {
            return 0;
        }

        let superview: *mut AnyObject = msg_send![egui_view, superview];
        let ns_window: *mut AnyObject = msg_send![egui_view, window];
        if superview.is_null() || ns_window.is_null() {
            return 0;
        }

        let frame_rect: CGRect = msg_send![egui_view, frame];
        let effect_view: *mut AnyObject = msg_send![class!(NSVisualEffectView), alloc];
        let effect_view: *mut AnyObject = msg_send![effect_view, initWithFrame: frame_rect];
        if effect_view.is_null() {
            return 0;
        }

        let clear_color: *mut AnyObject = msg_send![class!(NSColor), clearColor];
        let _: () = msg_send![ns_window, setOpaque: false];
        let _: () = msg_send![ns_window, setBackgroundColor: clear_color];
        let _: () = msg_send![effect_view, setAutoresizingMask: NS_VIEW_WIDTH_SIZABLE | NS_VIEW_HEIGHT_SIZABLE];
        let _: () = msg_send![effect_view, setMaterial: material];
        let _: () =
            msg_send![effect_view, setBlendingMode: NS_VISUAL_EFFECT_BLENDING_MODE_BEHIND_WINDOW];
        let _: () = msg_send![effect_view, setState: NS_VISUAL_EFFECT_STATE_ACTIVE];

        let _: () = msg_send![superview, addSubview: effect_view positioned: NS_WINDOW_BELOW relativeTo: egui_view];
        let _: () = msg_send![effect_view, release];

        effect_view as usize
    }
}

#[cfg(not(target_os = "macos"))]
fn setup_acrylic(
    _frame: &eframe::Frame,
    _settings: &AcrylicSettings,
    acrylic_view: usize,
) -> usize {
    acrylic_view
}

#[cfg(target_os = "macos")]
fn macos_material_for_blur(blur: f32) -> isize {
    match blur {
        b if b < 0.20 => 12, // NSVisualEffectMaterialWindowBackground
        b if b < 0.45 => 21, // NSVisualEffectMaterialUnderWindowBackground
        b if b < 0.70 => 18, // NSVisualEffectMaterialContentBackground
        b if b < 0.90 => 7,  // NSVisualEffectMaterialSidebar
        _ => 13,             // NSVisualEffectMaterialHudWindow
    }
}

fn top_bar_button(ui: &mut egui::Ui, label: &str, height: f32) -> egui::Response {
    ui.add_sized(
        Vec2::new(height, height),
        egui::Button::new(RichText::new(label).size(20.0).color(TEXT_WHITE))
            .fill(Color32::TRANSPARENT)
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

fn button_style<'a>(
    label: &'a str,
    _calc: &calculator::Calculator,
    acrylic: &AcrylicSettings,
) -> ButtonStyle {
    if label == "=" {
        ButtonStyle {
            fill: chrome_bg(acrylic),
            hover: chrome_hover_bg(acrylic),
            active: chrome_active_bg(acrylic),
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
            fill: chrome_bg(acrylic),
            hover: chrome_hover_bg(acrylic),
            active: chrome_active_bg(acrylic),
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
