use std::{
    collections::HashMap,
    process::Command,
    sync::{Arc, Mutex},
};

use eframe::{egui_glow, glow};
use egui::{
    Align, Color32, Frame, Grid, Image, ImageButton, Label, Layout, Pos2, Response,
    Rounding, Sense, Stroke, TextEdit, Vec2,
};

use crate::{
    color::{rgb_to_cymk, rgb_to_hsl, Color},
    gradient::{Gradient, GradientType},
    theme,
};

pub struct App {
    tab: String,
    color: Color,
    hex: String,
    spacing: f32,
    gradient_width: f32,
    gradient_height: f32,
    main_handle_radius: f32,
    main_handle_stroke: f32,
    slider_handle_stroke: f32,
    slider_height: f32,
    slider_margin: f32,
    gradient_click: bool,
    gradient: Arc<Mutex<Gradient>>,
    slider_clicks: HashMap<String, bool>,
    slider_gradients: HashMap<String, Arc<Mutex<Gradient>>>,
    slider_texts: HashMap<String, String>,
}

impl App {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let gl = cc
            .gl
            .as_ref()
            .expect("You need to run eframe with the glow backend");

        let color = Color::from_rgb(22, 22, 33);
        let slider_labels = ["r", "g", "b", "h", "s", "v"];
        Self {
            tab: String::from("HSV"),
            hex: color.hex.clone(),
            color: color.clone(),
            spacing: 5.0,
            gradient_width: 380.0,
            gradient_height: 270.0,
            main_handle_radius: 13.0,
            main_handle_stroke: 2.5,
            slider_handle_stroke: 2.0,
            slider_height: 20.0,
            slider_margin: 12.0,
            gradient: Arc::new(Mutex::new(Gradient::new(gl, GradientType::Gradient))),
            gradient_click: false,
            slider_clicks: slider_labels
                .iter()
                .map(|n| (n.to_string(), false))
                .collect(),
            slider_gradients: slider_labels
                .iter()
                .map(|n| {
                    (
                        n.to_string(),
                        Arc::new(Mutex::new(Gradient::new(
                            gl,
                            GradientType::Slider(n.to_string()),
                        ))),
                    )
                })
                .collect(),
            slider_texts: slider_labels
                .iter()
                .map(|n| (n.to_string(), format!("{:.0}", color.value_by_name(n))))
                .collect(),
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let old = ctx.style().visuals.clone();
        ctx.set_visuals(theme::THEME.visuals(old));
        egui::CentralPanel::default().show(ctx, |ui| {
            let hue = Color::from_hsv(self.color.h, 100, 100);
            ui.set_max_width(self.gradient_width);
            ui.spacing_mut().item_spacing = Vec2::new(self.spacing, self.spacing);
            self.draw_gradient(
                ui,
                GradientType::Gradient,
                Vec2::new(self.gradient_width, self.gradient_height),
                &hue,
            );

            ui.spacing_mut().item_spacing = Vec2::new(self.spacing, self.spacing) * 2.0;
            ui.horizontal(|ui| {
                ["RGB", "HSV", "Values"]
                    .iter()
                    .for_each(|label| self.draw_tab_toggle(ui, label.to_string()));
            });

            ui.spacing_mut().item_spacing = Vec2::ZERO;
            if self.tab != "Values" {
                self.draw_sliders(ui);
            } else {
                self.draw_values(ui);
            }

            ui.with_layout(Layout::left_to_right(Align::BOTTOM), |ui| {
                self.draw_footer(ui);
            });
        });
    }

    fn on_exit(&mut self, gl: Option<&glow::Context>) {
        if let Some(gl) = gl {
            self.gradient.lock().unwrap().destroy(gl);
            for (_, s) in self.slider_gradients.iter_mut() {
                s.lock().unwrap().destroy(gl);
            }
        }
    }
}

impl App {
    fn draw_tab_toggle(&mut self, ui: &mut egui::Ui, label: String) {
        let mut is_open = self.tab.contains(&label);
        ui.toggle_value(&mut is_open, &format!("  {label}  "));
        self.set_open(label, is_open);
    }

    fn draw_sliders(&mut self, ui: &mut egui::Ui) {
        Frame::default()
            .inner_margin(self.slider_margin)
            .show(ui, |ui| {
                Grid::new("Sliders")
                    .num_columns(3)
                    .min_col_width(0.0)
                    .show(ui, |ui| {
                        match self.tab.as_str() {
                            "RGB" => vec!["r", "g", "b"],
                            "HSV" => vec!["h", "s", "v"],
                            _ => vec![],
                        }
                        .iter()
                        .for_each(|label| {
                            self.draw_slider(ui, label.to_string());
                            ui.end_row();
                        });
                    });
            });
    }

    fn draw_slider(&mut self, ui: &mut egui::Ui, label: String) {
        let label_width = 5.0;
        let text_edit_width = 50.0;
        ui.add_sized(
            Vec2::new(label_width, 20.0),
            Label::new(label.chars().next().unwrap().to_uppercase().to_string()),
        );
        let size = Vec2::new(
            self.gradient_width
                - self.slider_margin * 2.0
                - self.spacing * 2.0
                - text_edit_width
                - label_width,
            self.slider_height,
        );
        self.draw_gradient(
            ui,
            GradientType::Slider(label.clone()),
            size,
            &self.color.clone(),
        );
        if ui
            .add_sized(
                Vec2::new(text_edit_width, 20.0),
                TextEdit::singleline(self.slider_texts.get_mut(&label).unwrap())
                    .horizontal_align(Align::RIGHT)
                    .vertical_align(Align::Center),
            )
            .changed()
        {
            self.on_slider_text_changed(label);
        };
    }

    fn draw_gradient(&mut self, ui: &mut egui::Ui, gtype: GradientType, size: Vec2, hue: &Color) {
        let response = match &gtype {
            GradientType::Gradient => self.draw_main_gradient(ui, size, hue),
            GradientType::Slider(stype) => self.draw_slider_gradient(ui, stype.clone(), size, hue),
        };
        self.handle_gradient_scroll(ui, &response, &gtype);
        self.handle_gradient_click(&response, &gtype);
    }

    fn draw_main_gradient(&mut self, ui: &mut egui::Ui, size: Vec2, hue: &Color) -> Response {
        let response = self.draw_gradient_frame(ui, size, hue, self.gradient.clone());
        let rect = response.rect;
        let position = Pos2 {
            x: rect.min.x + rect.width() * self.color.float_by_name("s"),
            y: rect.min.y + rect.height() - rect.height() * self.color.float_by_name("v"),
        };
        self.draw_gradient_handle(
            ui,
            position,
            &self.color.clone(),
            &self.color.inv(),
            self.main_handle_radius,
            self.main_handle_stroke,
        );
        response
    }

    fn draw_slider_gradient(
        &mut self,
        ui: &mut egui::Ui,
        stype: String,
        size: Vec2,
        hue: &Color,
    ) -> Response {
        let gradient = self.slider_gradients.get(&stype).unwrap().clone();
        let response = self.draw_gradient_frame(ui, size, hue, gradient);
        let radius = (self.slider_height - self.slider_handle_stroke) * 0.5;
        let color = if stype == "h" {
            Color::from_hsv(self.color.h, 100, 100)
        } else {
            self.color.clone()
        };
        let rect = response.rect;
        let position = Pos2 {
            x: rect.min.x + rect.width() * color.float_by_name(&stype),
            y: rect.min.y + rect.height() * 0.5,
        };
        self.draw_gradient_handle(
            ui,
            position,
            &color,
            &color.inv(),
            radius,
            self.slider_handle_stroke,
        );
        response
    }

    fn draw_gradient_frame(
        &mut self,
        ui: &mut egui::Ui,
        size: Vec2,
        hue: &Color,
        gradient: Arc<Mutex<Gradient>>,
    ) -> Response {
        egui::Frame::default()
            .stroke(Stroke::new(1.0, Color32::from_black_alpha(255)))
            .fill(Color32::from_black_alpha(255))
            .inner_margin(0.0)
            .outer_margin(10.0)
            .rounding(Rounding::same(0.0))
            .show(ui, |ui| {
                self.draw_gradient_canvas(ui, size, hue.clone(), gradient)
            })
            .inner
    }

    fn draw_gradient_canvas(
        &mut self,
        ui: &mut egui::Ui,
        size: Vec2,
        hue: Color,
        gradient: Arc<Mutex<Gradient>>,
    ) -> Response {
        let (rect, response) = ui.allocate_exact_size(size, egui::Sense::click_and_drag());
        let callback = egui::PaintCallback {
            rect,
            callback: std::sync::Arc::new(egui_glow::CallbackFn::new(move |_info, painter| {
                gradient.lock().unwrap().paint(painter.gl(), hue.clone());
            })),
        };
        ui.painter().add(callback);
        response
    }

    fn handle_gradient_click(&mut self, response: &Response, gtype: &GradientType) {
        let click = match gtype {
            GradientType::Gradient => &mut self.gradient_click,
            GradientType::Slider(stype) => self.slider_clicks.get_mut(stype).unwrap(),
        };
        if response.contains_pointer() && response.is_pointer_button_down_on() && !*click {
            *click = true;
        } else if !response.is_pointer_button_down_on() && *click {
            *click = false;
        }
        if !*click {
            return;
        }

        let rect = response.interact_rect;
        if let Some(pos) = response.hover_pos() {
            let pos = Vec2::new(
                pos.x.clamp(rect.min.x, rect.max.x),
                pos.y.clamp(rect.min.y, rect.max.y),
            );
            self.set_color(match gtype {
                GradientType::Gradient => {
                    let s = (pos.x - rect.min.x) / rect.width() * 100.0;
                    let v = (1.0 - (pos.y - rect.min.y) / rect.height()) * 100.0;
                    Color::from_hsv(self.color.h, s as u16, v as u16)
                }
                GradientType::Slider(stype) => {
                    let t = (pos.x - rect.min.x) / (rect.max.x - rect.min.x);
                    self.change_color_value(stype.clone(), t, true)
                }
            });
        }
    }

    fn handle_gradient_scroll(
        &mut self,
        ui: &mut egui::Ui,
        response: &Response,
        gtype: &GradientType,
    ) {
        let scroll_detla = ui.input(|i| i.raw_scroll_delta);
        if scroll_detla.y == 0.0 || !response.contains_pointer() {
            return;
        }
        match gtype {
            GradientType::Gradient => (),
            GradientType::Slider(stype) => {
                let value = self.color.value_by_name(stype) as i32
                    + if scroll_detla.y > 0.0 { 1 } else { -1 };
                self.set_color(self.change_color_value(stype.clone(), value as f32, false));
            }
        };
    }

    fn change_color_value(&self, label: String, t: f32, scaled: bool) -> Color {
        match label.as_str() {
            "r" => Color::from_rgb(
                self.get_fixed_color_value(t, 255, scaled),
                self.color.g,
                self.color.b,
            ),
            "g" => Color::from_rgb(
                self.color.r,
                self.get_fixed_color_value(t, 255, scaled),
                self.color.b,
            ),
            "b" => Color::from_rgb(
                self.color.r,
                self.color.g,
                self.get_fixed_color_value(t, 255, scaled),
            ),
            "h" => Color::from_hsv(
                self.get_fixed_color_value(t, 360, scaled),
                self.color.s,
                self.color.v,
            ),
            "s" => Color::from_hsv(
                self.color.h,
                self.get_fixed_color_value(t, 100, scaled),
                self.color.v,
            ),
            "v" => Color::from_hsv(
                self.color.h,
                self.color.s,
                self.get_fixed_color_value(t, 100, scaled),
            ),
            _ => Color::from_rgb(255, 0, 0),
        }
    }

    fn get_fixed_color_value(&self, t: f32, max: u16, scaled: bool) -> u16 {
        let mut value = t as u16;
        if scaled {
            value = (t * max as f32) as u16
        }
        value.clamp(0, max)
    }

    fn draw_gradient_handle(
        &mut self,
        ui: &mut egui::Ui,
        position: Pos2,
        color: &Color,
        stroke: &Color,
        radius: f32,
        width: f32,
    ) {
        ui.painter().circle(
            position,
            radius,
            color.to_color32(),
            Stroke {
                width,
                color: stroke.to_color32(),
            },
        );
    }

    fn draw_values(&self, ui: &mut egui::Ui) {
        ui.add_space(5.0);
        Frame::default()
            .inner_margin(self.slider_margin)
            .show(ui, |ui| {
                Grid::new("Values")
                    .num_columns(4)
                    .spacing([20.0, 10.0])
                    .max_col_width(140.0)
                    .show(ui, |ui| {
                        ui.label("RGB:");
                        ui.text_edit_singleline(&mut format!(
                            "{:.0}, {:.0}, {:.0}",
                            self.color.r, self.color.g, self.color.b
                        ));
                        ui.text_edit_singleline(&mut format!(
                            "{:.2}, {:.2}, {:.2}",
                            self.color.float_by_name("r"),
                            self.color.float_by_name("g"),
                            self.color.float_by_name("b"),
                        ));
                        ui.end_row();

                        ui.label("HSV:");
                        ui.text_edit_singleline(&mut format!(
                            "{:.0}, {:.0}, {:.0}",
                            self.color.h, self.color.s, self.color.v,
                        ));
                        ui.text_edit_singleline(&mut format!(
                            "{:.2}, {:.2}, {:.2}",
                            self.color.float_by_name("h"),
                            self.color.float_by_name("s"),
                            self.color.float_by_name("v"),
                        ));
                        ui.end_row();

                        ui.label("HSL:");
                        let (h, s, l) = rgb_to_hsl(self.color.r, self.color.g, self.color.b);
                        ui.text_edit_singleline(&mut format!("{:.0}, {:.0}, {:.0}", h, s, l));
                        ui.text_edit_singleline(&mut format!(
                            "{:.2}, {:.2}, {:.2}",
                            self.color.float_by_name("h"),
                            s as f32 * 0.01,
                            l as f32 * 0.01,
                        ));
                        ui.end_row();

                        ui.label("CYMK:");
                        let (c, y, m, k) = rgb_to_cymk(self.color.r, self.color.g, self.color.b);
                        ui.text_edit_singleline(&mut format!(
                            "{:.0}, {:.0}, {:.0}, {:.0}",
                            c, y, m, k,
                        ));
                        ui.text_edit_singleline(&mut format!(
                            "{:.2}, {:.2}, {:.2}, {:.2}",
                            c as f32 * 0.01,
                            y as f32 * 0.01,
                            m as f32 * 0.01,
                            k as f32 * 0.01
                        ));
                        ui.end_row();
                    });
            });
    }

    fn draw_footer(&mut self, ui: &mut egui::Ui) {
        ui.spacing_mut().item_spacing = Vec2::new(7.0, 0.0);
        ui.spacing_mut().button_padding = Vec2::new(8.0, 8.0);
        let picker_button =
            ImageButton::new(Image::new(egui::include_image!("../picker_icon.png")))
                .tint(theme::THEME.fg)
                .rounding(4.0);
        if ui.add_sized([30.0, 30.0], picker_button).clicked() {
            self.run_hyprpicker();
        }
        let (rect, _) =
            ui.allocate_exact_size(Vec2::new(100.0, 32.0), Sense::focusable_noninteractive());
        ui.painter().rect_filled(rect, 2.0, self.color.to_color32());
        ui.painter().rect_stroke(
            rect,
            1.0,
            Stroke {
                width: 1.0,
                color: theme::THEME.bg_selected,
            },
        );
        if ui
            .add_sized(
                [80.0, 20.0],
                TextEdit::singleline(&mut self.hex).vertical_align(Align::Center),
            )
            .changed()
        {
            if let Some(hex) = Color::from_hex(self.hex.clone()) {
                self.set_color(hex);
            }
        }
    }

    fn set_open(&mut self, key: String, is_open: bool) {
        if is_open && self.tab != key {
            self.tab = key;
        }
    }

    fn set_color(&mut self, color: Color) {
        self.color = color;
        self.hex.clone_from(&self.color.hex);
        let slider_labels = ["r", "g", "b", "h", "s", "v"];
        for label in slider_labels.iter() {
            if let Some(text) = self.slider_texts.get_mut(label.to_owned()) {
                *text = self.color.value_by_name(label).to_string();
            }
        }
    }

    fn on_slider_text_changed(&mut self, label: String) {
        if let Some(text) = self.slider_texts.get_mut(&label) {
            match text.parse::<f32>() {
                Ok(t) => {
                    self.set_color(self.change_color_value(label, t, false));
                }
                Err(_) => {
                    self.set_color(self.color.clone());
                }
            }
        }
    }

    fn run_hyprpicker(&mut self) {
        let output = Command::new("/bin/hyprpicker")
            .output()
            .expect("Failed to get 'hyprpicker' output.");
        let hex = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if let Some(hex) = Color::from_hex(hex) {
            self.set_color(hex);
        }
    }
}
