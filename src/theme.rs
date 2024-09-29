use egui::{style, Color32, Visuals};

pub const THEME: Theme = {
    Theme {
        fg: Color32::from_rgb(214, 216, 220),
        bg: Color32::from_rgb(41, 45, 59),
        bg_dark: Color32::from_rgb(30, 33, 43),
        bg_light: Color32::from_rgb(56, 60, 74),
        bg_selected: Color32::from_rgb(68, 72, 85),
        fg_selected: Color32::from_rgb(128, 132, 145),
    }
};

pub struct Theme {
    pub fg: Color32,
    pub bg: Color32,
    pub bg_dark: Color32,
    pub bg_light: Color32,
    pub bg_selected: Color32,
    pub fg_selected: Color32,
}

impl Theme {
    pub fn visuals(&self, old: Visuals) -> egui::Visuals {
        Visuals {
            dark_mode: true,
            override_text_color: Some(self.fg),
            hyperlink_color: self.fg,
            faint_bg_color: self.bg_light,
            extreme_bg_color: self.bg_dark,
            window_fill: self.bg,
            panel_fill: self.bg,
            widgets: style::Widgets {
                noninteractive: self.make_widget_visual(old.widgets.noninteractive, self.bg),
                inactive: self.make_widget_visual(old.widgets.inactive, self.bg_dark),
                hovered: self.make_widget_visual(old.widgets.hovered, self.bg_dark),
                active: self.make_widget_visual(old.widgets.active, self.bg_light),
                open: self.make_widget_visual(old.widgets.open, self.bg),
            },
            selection: style::Selection {
                bg_fill: self.bg_selected,
                stroke: egui::Stroke {
                    color: self.fg,
                    ..old.selection.stroke
                },
            },
            ..old
        }
    }

    fn make_widget_visual(
        &self,
        old: style::WidgetVisuals,
        bg_fill: egui::Color32,
    ) -> style::WidgetVisuals {
        style::WidgetVisuals {
            bg_fill,
            weak_bg_fill: bg_fill,
            bg_stroke: egui::Stroke {
                color: self.fg_selected,
                ..old.bg_stroke
            },
            fg_stroke: egui::Stroke {
                color: self.fg,
                ..old.fg_stroke
            },
            ..old
        }
    }
}
