fn rgb2hsv(rgb: &(u8, u8, u8)) -> (f64, f64, f64) {
    let r: f64 = rgb.0 as f64 / 255.0;
    let g: f64 = rgb.1 as f64 / 255.0;
    let b: f64 = rgb.2 as f64 / 255.0;

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
        return (0.0, 0.0, v);
    }

    let s = range / rgb_max;
    if s == 0.0 { // grey
        return (0.0, s, v);
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

    (h, s, v)
}

fn hsv2rgb(hsv: &(f64, f64, f64)) -> (u8, u8, u8) {
    let h6 = hsv.0 / 60.0;
    let s = hsv.1;
    let v = hsv.2;

    if s == 0.0 { // grey
        return (v as u8, v as u8, v as u8);
    }

    let f = h6 % 1.0;
    let p = v * (1.0 - s);
    let q = v * (1.0 - s * f);
    let t = v * (1.0 - s * (1.0 - f));

    let r;
    let g;
    let b;
    if h6 < 1.0 {
        r = v;
        g = t;
        b = p;
    } else if h6 < 2.0 {
        r = q;
        g = v;
        b = p;
    } else if h6 < 3.0 {
        r = p;
        g = v;
        b = t;
    } else if h6 < 4.0 {
        r = p;
        g = q;
        b = v;
    } else if h6 < 5.0 {
        r = t;
        g = p;
        b = v;
    } else {
        r = v;
        g = p;
        b = q;
    }

    ((r * 255.0) as u8, (g * 255.0) as u8, (b * 255.0) as u8)
}
