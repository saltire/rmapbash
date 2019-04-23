#[derive(Clone, Copy, Debug, Default)]
pub struct RGBA {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

struct RGB {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

struct HSV {
    pub h: f64,
    pub s: f64,
    pub v: f64,
}

const BLOCKLIGHT_MIN: f64 = 0.17;
const BLOCKLIGHT_MULT: f64 = 1.0 - BLOCKLIGHT_MIN;

fn rgb2hsv(rgb: &RGB) -> HSV {
    let r: f64 = rgb.r as f64 / 255.0;
    let g: f64 = rgb.g as f64 / 255.0;
    let b: f64 = rgb.b as f64 / 255.0;

    let rgb_min = if g < b {
        if r < g { r } else { g }
    } else {
        if r < b { r } else { b }
    };

    let rgb_max = if g > b {
        if r > g { r } else { g }
    } else {
        if r > b { r } else { b }
    };

    let range = rgb_max - rgb_min;

    let v = rgb_max;
    if v == 0.0 { // black
        return HSV { h: 0.0, s: 0.0, v };
    }

    let s = range / rgb_max;
    if s == 0.0 { // grey
        return HSV { h: 0.0, s, v };
    }

    let mut h = if r == rgb_max {
        (g - b) / range
    } else if g == rgb_max {
        (b - r) / range + 2.0
    } else {
        (r - g) / range + 4.0
    };
    h *= 60.0;
    if h < 0.0 {
        h += 360.0;
    }

    HSV { h, s, v }
}

fn hsv2rgb (hsv: &HSV) -> RGB {
    let h6 = hsv.h / 60.0;

    if hsv.s == 0.0 { // grey
        return RGB { r: hsv.v as u8, g: hsv.v as u8, b: hsv.v as u8 };
    }

    let f = h6 % 1.0;
    let p = hsv.v * (1.0 - hsv.s);
    let q = hsv.v * (1.0 - hsv.s * f);
    let t = hsv.v * (1.0 - hsv.s * (1.0 - f));

    let r;
    let g;
    let b;
    if h6 < 1.0 {
        r = hsv.v;
        g = t;
        b = p;
    } else if h6 < 2.0 {
        r = q;
        g = hsv.v;
        b = p;
    } else if h6 < 3.0 {
        r = p;
        g = hsv.v;
        b = t;
    } else if h6 < 4.0 {
        r = p;
        g = q;
        b = hsv.v;
    } else if h6 < 5.0 {
        r = t;
        g = p;
        b = hsv.v;
    } else {
        r = hsv.v;
        g = p;
        b = q;
    }

    RGB { r: (r * 255.0) as u8, g: (g * 255.0) as u8, b: (b * 255.0) as u8 }
}

pub fn shade_biome_color(blockcolor: &RGBA, biomecolor: &RGBA) -> RGBA {
    let biome_hsv = rgb2hsv(&RGB { r: biomecolor.r, g: biomecolor.g, b: biomecolor.b });
    let block_hsv = rgb2hsv(&RGB { r: blockcolor.r, g: blockcolor.g, b: blockcolor.b });

    // use hue/sat from biome color, and val from block color
    let rgb = hsv2rgb(&HSV { h: biome_hsv.h, s: biome_hsv.s, v: block_hsv.v });

    // use alpha from block color
    RGBA { r: rgb.r, g: rgb.g, b: rgb.b, a: blockcolor.a }
}

pub fn multiply_color(color1: &RGBA, color2: &RGBA) -> RGBA {
    RGBA {
        r: (color1.r as u16 * color2.r as u16 / 255) as u8,
        g: (color1.g as u16 * color2.g as u16 / 255) as u8,
        b: (color1.b as u16 * color2.b as u16 / 255) as u8,
        a: color1.a,
    }
}

pub fn blend_alpha_color(top: &RGBA, bottom: &RGBA) -> RGBA {
    if top.a == 255 || bottom.a == 0 {
        return top.clone();
    }
    if top.a == 0 {
        return bottom.clone();
    }

    let talpha = top.a as f64;
    let balpha = bottom.a as f64 * (255.0 - top.a as f64) / 255.0;
    let alpha = talpha + balpha;
    RGBA {
        r: ((top.r as f64 * talpha + bottom.r as f64 * balpha) / alpha) as u8,
        g: ((top.g as f64 * talpha + bottom.g as f64 * balpha) / alpha) as u8,
        b: ((top.b as f64 * talpha + bottom.b as f64 * balpha) / alpha) as u8,
        a: alpha as u8,
    }
}

pub fn set_light_level(color: &RGBA, level: &u8) -> RGBA {
    let llevel = *level as f64 * BLOCKLIGHT_MULT / 15.0 + BLOCKLIGHT_MIN;
    RGBA {
        r: (color.r as f64 * llevel) as u8,
        g: (color.g as f64 * llevel) as u8,
        b: (color.b as f64 * llevel) as u8,
        a: color.a,
    }
}
