use egui::{Color32, TextBuffer};

// r,g,b: 0..255
// h: 0..360
// s,v,l,c,y,m,k: 0..100
#[derive(Debug, Default, Clone)]
pub struct Color {
    pub r: u16,
    pub g: u16,
    pub b: u16,
    pub h: u16,
    pub s: u16,
    pub v: u16,
    pub hex: String,
}

impl Color {
    pub fn from_rgb(r: u16, g: u16, b: u16) -> Self {
        let r = r.clamp(0, 255);
        let g = g.clamp(0, 255);
        let b = b.clamp(0, 255);
        let (h, s, v) = rgb_to_hsv(r, g, b);
        let hex = get_hex(r, g, b);
        Color {
            r,
            g,
            b,
            h,
            s,
            v,
            hex,
        }
    }

    pub fn from_hsv(h: u16, s: u16, v: u16) -> Self {
        let h = h.clamp(0, 360);
        let s = s.clamp(0, 100);
        let v = v.clamp(0, 100);
        let (r, g, b) = hsv_to_rbg(h, s, v);
        let hex = get_hex(r, g, b);
        Color {
            r,
            g,
            b,
            h,
            s,
            v,
            hex,
        }
    }

    pub fn from_hex(hex: String) -> Option<Self> {
        if hex.len() != 7 {
            return None;
        }
        if let Some(stripped) = hex.strip_prefix('#') {
            let [_, r, g, b] = match u32::from_str_radix(stripped.as_str(), 16) {
                Ok(r) => r.to_be_bytes(),
                Err(_) => return None,
            };
            let (h, s, v) = rgb_to_hsv(r as u16, g as u16, b as u16);
            return Some(Color {
                r: r as u16,
                g: g as u16,
                b: b as u16,
                h,
                s,
                v,
                hex,
            });
        }
        None
    }

    pub fn dim(&self) -> Self {
        let h = (self.h + 180) % 360;
        let s = 30;
        let v = 100 - self.v;
        Color::from_hsv(h, s, v)
    }

    pub fn inv(&self) -> Self {
        let h = (self.h + 180) % 360;
        let s = 85;
        let v = 75;
        Color::from_hsv(h, s, v)
    }

    pub fn to_color32(&self) -> Color32 {
        Color32::from_rgb(self.r as u8, self.g as u8, self.b as u8)
    }

    pub fn value_by_name(&self, name: &str) -> u16 {
        match name {
            "r" => self.r,
            "g" => self.g,
            "b" => self.b,
            "h" => self.h,
            "s" => self.s,
            "v" => self.v,
            _ => 0,
        }
    }

    pub fn float_by_name(&self, name: &str) -> f32 {
        match name {
            "r" => self.r as f32 / 255.0,
            "g" => self.g as f32 / 255.0,
            "b" => self.b as f32 / 255.0,
            "h" => self.h as f32 / 360.0,
            "s" => self.s as f32 / 100.0,
            "v" => self.v as f32 / 100.0,
            _ => 0.0,
        }
    }
}

pub fn rgb_to_hsv(r: u16, g: u16, b: u16) -> (u16, u16, u16) {
    let r01 = r as f32 / 255.0;
    let g01 = g as f32 / 255.0;
    let b01 = b as f32 / 255.0;
    let cmax = r01.max(g01.max(b01));
    let cmin = r01.min(g01.min(b01));
    let delta = cmax - cmin;
    let mut h = 0.0;
    if delta <= 0.0001 {
        h = 0.0;
    } else if cmax == r01 {
        h = (60.0 * ((g01 - b01) / delta) + 360.0) % 360.0;
    } else if cmax == g01 {
        h = (60.0 * ((b01 - r01) / delta) + 120.0) % 360.0;
    } else if cmax == b01 {
        h = (60.0 * ((r01 - g01) / delta) + 240.0) % 360.0;
    }
    let s = if cmax != 0.0 {
        (delta / cmax) * 100.0
    } else {
        0.0
    };
    let v = cmax * 100.0;
    (h as u16, s as u16, v as u16)
}

pub fn rgb_to_hsl(r: u16, g: u16, b: u16) -> (u16, u16, u16) {
    let r01 = r as f32 / 255.0;
    let g01 = g as f32 / 255.0;
    let b01 = b as f32 / 255.0;
    let cmax = r01.max(g01.max(b01));
    let cmin = r01.min(g01.min(b01));
    let delta = cmax - cmin;
    let mut h = 0.0;
    if delta <= 0.0001 {
        h = 0.0;
    } else if cmax == r01 {
        h = (60.0 * ((g01 - b01) / delta) + 360.0) % 360.0;
    } else if cmax == g01 {
        h = (60.0 * ((b01 - r01) / delta) + 120.0) % 360.0;
    } else if cmax == b01 {
        h = (60.0 * ((r01 - g01) / delta) + 240.0) % 360.0;
    }
    let l = (cmax + cmin) * 0.5;
    let s = if delta > 0.0001 {
        delta / (1.0 - (2.0 * l - 1.0).abs()) * 100.0
    } else {
        0.0
    };
    (h as u16, s as u16, (l * 100.0) as u16)
}

pub fn rgb_to_cymk(r: u16, g: u16, b: u16) -> (u16, u16, u16, u16) {
    let r01 = r as f32 / 255.0;
    let g01 = g as f32 / 255.0;
    let b01 = b as f32 / 255.0;
    let k = 1.0 - r01.max(g01.max(b01));
    let c = (1.0 - r01 - k) / (1.0 - k) * 100.0;
    let m = (1.0 - g01 - k) / (1.0 - k) * 100.0;
    let y = (1.0 - b01 - k) / (1.0 - k) * 100.0;
    (c as u16, y as u16, m as u16, (k * 100.0) as u16)
}

fn hsv_to_rbg(h: u16, s: u16, v: u16) -> (u16, u16, u16) {
    let s01 = s as f32 / 100.0;
    let v01 = v as f32 / 100.0;
    let c = s01 * v01;
    let x = c * (1.0 - (((h as f32 / 60.0) % 2.0) - 1.0).abs());
    let m = v01 - c;
    let (r01, g01, b01) = if h < 60 {
        (c, x, 0.0)
    } else if h < 120 {
        (x, c, 0.0)
    } else if h < 180 {
        (0.0, c, x)
    } else if h < 240 {
        (0.0, x, c)
    } else if h < 300 {
        (x, 0.0, c)
    } else {
        (c, 0.0, x)
    };
    let r = (r01 + m) * 255.0;
    let g = (g01 + m) * 255.0;
    let b = (b01 + m) * 255.0;
    (r as u16, g as u16, b as u16)
}

fn get_hex(r: u16, g: u16, b: u16) -> String {
    format!("#{:X?}{:X?}{:X?}", r, g, b)
}
