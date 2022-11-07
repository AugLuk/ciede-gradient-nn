use std::f64::consts::PI;
// use rand::Rng;

// pub fn random_cielab(rng: &mut impl Rng) -> [f64; 3] {
//     loop {
//         let c = [
//             rng.gen_range(0.0..=100.0),
//             rng.gen_range(-86.18704166062818..=98.25137200280926),
//             rng.gen_range(-107.86309588218504..=94.48279336975611),
//         ];
//
//         if is_valid_rgb(&cielab_to_rgb(&c)) {
//             break c;
//         }
//     }
// }

pub fn is_valid_cielab(c: &[f64; 3]) -> bool {
    if c[0] < 0.0 || c[0] > 100.0 || c[1] < -86.18704166062818 || c[1] > 98.25137200280926 || c[2] < -107.86309588218504 || c[2] > 94.48279336975611 {
        false
    } else {
        is_valid_rgb(&cielab_to_rgb(&c))
    }
}

fn is_valid_rgb(c: &[f64; 3]) -> bool {
    for c in c {
        if *c < 0.0 || *c > 1.0 {
            return false;
        }
    }
    true
}

pub fn rgb_to_srgb(c: &[f64; 3]) -> [f64; 3] {
    let mut output = [0.0; 3];
    for (c_s_rgb, c_lin) in output.iter_mut().zip(c.iter()) {
        let c_lin = c_lin.clamp(0.0, 1.0);

        *c_s_rgb = if c_lin <= 0.0031308 {
            12.92 * c_lin
        } else {
            1.055 * c_lin.powf(1.0 / 2.4) - 0.055
        }
    }
    output
}

pub fn sdl2_color_to_rgb(c: sdl2::pixels::Color) -> [f64; 3] {
    let input = [c.r, c.g, c.b];
    let mut output = [0.0; 3];
    for (c_lin, c_s_rgb_8) in output.iter_mut().zip(input.iter()) {
        let c_s_rgb = *c_s_rgb_8 as f64 / 255.0;

        *c_lin = if c_s_rgb <= 0.04045 {
            c_s_rgb / 12.92
        } else {
            ((c_s_rgb + 0.055) / 1.055).powf(2.4)
        }
    }

    output
}

pub fn sdl2_color_to_cielab(c: sdl2::pixels::Color) -> [f64; 3] {
    let c = rgb_to_ciexyz(&sdl2_color_to_rgb(c));
    ciexyz_to_cielab(&[c[0] * 100.0, c[1] * 100.0, c[2] * 100.0])
}

fn ciexyz_to_rgb(c: &[f64; 3]) -> [f64; 3] {
    [
        3.2406 * c[0] - 1.5372 * c[1] - 0.4986 * c[2],
        -0.9689 * c[0] + 1.8758 * c[1] + 0.0415 * c[2],
        0.0557 * c[0] - 0.2040 * c[1] + 1.0570 * c[2]
    ]
}

pub fn rgb_to_ciexyz(c: &[f64; 3]) -> [f64; 3] {
    [
        0.4124 * c[0] + 0.3576 * c[1] + 0.1805 * c[2],
        0.2126 * c[0] + 0.7152 * c[1] + 0.0722 * c[2],
        0.0193 * c[0] + 0.1192 * c[1] + 0.9505 * c[2]
    ]
}

pub fn cielab_to_rgb(c: &[f64; 3]) -> [f64; 3] {
    let c = cielab_to_ciexyz(c);
    ciexyz_to_rgb(&[c[0] / 100.0, c[1] / 100.0, c[2] / 100.0])
}

fn cie_f(t: f64) -> f64 {
    if t > 216.0 / 24389.0 {
        t.powf(1.0 / 3.0)
    } else {
        841.0 * t / 108.0 + 4.0 / 29.0
    }
}

fn cie_f_inverse(t: f64) -> f64 {
    let delta = 6.0 / 29.0;

    if t > delta {
        t.powi(3)
    } else {
        3.0 * delta.powi(2) * (t - 4.0 / 29.0)
    }
}

fn cielab_to_ciexyz(c: &[f64; 3]) -> [f64; 3] {
    let xn = 95.0489;
    let yn = 100.0;
    let zn = 108.8840;

    let foo = (c[0] + 16.0) / 116.0;

    [
        xn * cie_f_inverse(foo + c[1] / 500.0),
        yn * cie_f_inverse(foo),
        zn * cie_f_inverse(foo - c[2] / 200.0),
    ]
}

fn ciexyz_to_cielab(c: &[f64; 3]) -> [f64; 3] {
    [
        116.0 * cie_f(c[1] / 100.0) - 16.0,
        500.0 * (cie_f(c[0] / 95.0489) - cie_f(c[1] / 100.0)),
        200.0 * (cie_f(c[1] / 100.0) - cie_f(c[2] / 108.8840))
    ]
}

fn cie_atan2(y: f64, x: f64) -> f64 {
    (y.atan2(x) / (2.0 * PI)).rem_euclid(1.0) * 360.0
}

fn cie_sin(x: f64) -> f64 {
    (x / 360.0 * 2.0 * PI).sin()
}

fn cie_cos(x: f64) -> f64 {
    (x / 360.0 * 2.0 * PI).cos()
}

pub fn cielab_dist_ciede2000(c1: &[f64; 3], c2: &[f64; 3]) -> f64 {
    let l1 = c1[0];
    let a1 = c1[1];
    let b1 = c1[2];

    let l2 = c2[0];
    let a2 = c2[1];
    let b2 = c2[2];

    let k_l = 1.0;
    let k_c = 1.0;
    let k_h = 1.0;

    let epsilon = 0.0;

    let dlp = l2 - l1;

    let c1 = a1.hypot(b1);
    let c2 = a2.hypot(b2);
    let lm = (l1 + l2) / 2.0;
    let cm = (c1 + c2) / 2.0;

    let ap1 = a1 + a1 / 2.0 * (1.0 - (cm.powi(7) / (cm.powi(7) + 6103515625.0)).sqrt());
    let ap2 = a2 + a2 / 2.0 * (1.0 - (cm.powi(7) / (cm.powi(7) + 6103515625.0)).sqrt());

    let cp1 = ap1.hypot(b1);
    let cp2 = ap2.hypot(b2);
    let cpm = (cp1 + cp2) / 2.0;
    let dcp = cp2 - cp1;

    // println!("cp1 = {}, cp2 = {}", cp1, cp2);

    let hp1 = if cp1 <= epsilon {
        0.0
    } else {
        cie_atan2(b1, ap1)
    };

    let hp2 = if cp2 <= epsilon {
        0.0
    } else {
        cie_atan2(b2, ap2)
    };

    // println!("hp1 = {}, hp2 = {}", hp1, hp2);

    let mut dlhp= 0.0;
    let hpm;

    if cp1 <= epsilon || cp2 <= epsilon {
        hpm = hp1 + hp2;
    } else {
        dlhp = if (hp1 - hp2).abs() <= 180.0 {
            hp2 - hp1
        } else {
            if hp2 <= hp1 {
                hp2 - hp1 + 360.0
            } else {
                hp2 - hp1 - 360.0
            }
        };

        hpm = if (hp1 - hp2).abs() <= 180.0 {
            (hp1 + hp2) / 2.0
        } else {
            if hp1 + hp2 < 360.0 {
                (hp1 + hp2 + 360.0) / 2.0
            } else {
                (hp1 + hp2 - 360.0) / 2.0
            }
        };
    }

    // println!("dlhp = {}", dlhp);

    let duhp = 2.0 * (cp1 * cp2).sqrt() * cie_sin(dlhp / 2.0);

    let t = 1.0 - 0.17 * cie_cos(hpm - 30.0) + 0.24 * cie_cos(2.0 * hpm) + 0.32 * cie_cos(3.0 * hpm + 6.0) - 0.20 * cie_cos(4.0 * hpm - 63.0);

    // println!("t = {}", t);

    let s_l = 1.0 + (0.015 * (lm - 50.0).powi(2)) / (20.0 + (lm - 50.0).powi(2)).sqrt();
    let s_c = 1.0 + 0.045 * cpm;
    let s_h = 1.0 + 0.015 * cpm * t;

    let r_t = -2.0 * (cpm.powi(7) / (cpm.powi(7) + 6103515625.0)).sqrt() * cie_sin(60.0 * (-((hpm - 275.0) / 25.0).powi(2)).exp());

    // println!("dlp = {}, s_l = {}", dlp, s_l);
    // println!("dcp = {}, s_c = {}", dcp, s_c);
    // println!("duhp = {}, s_h = {}", duhp, s_h);
    // println!("r_t = {}", r_t);

    ((dlp / (k_l * s_l)).powi(2) + (dcp / (k_c * s_c)).powi(2) + (duhp / (k_h * s_h)).powi(2) + r_t * dcp / (k_c * s_c) * duhp / (k_h * s_h)).sqrt()
}