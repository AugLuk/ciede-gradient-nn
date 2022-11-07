mod color;

use std::fs;
use configparser::ini::Ini;
use nn::NN;
use rand::{Rng, SeedableRng};
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use crate::color::*;

fn cost(ic1: &[f64; 3], ic2: &[f64; 3], ii: f64, oc: &[f64; 3]) -> f64 {
    let de1 = cielab_dist_ciede2000(&ic1, &oc);
    let de2 = cielab_dist_ciede2000(&ic2, &oc);
    let oi = de1 / (de1 + de2);

    (de1 + de2) * ((oi - ii).abs() + 0.5)
}

fn main() -> Result<(), String> {
    // ---------------------------------------------------------------------------------------------
    // Setup
    // ---------------------------------------------------------------------------------------------

    let config_str = fs::read_to_string("config.ini").expect("Error while reading the configuration file.");
    let mut config = Ini::new();
    let _ = config.read(config_str);

    let use_sample_nn = config.getbool("general", "use_sample_nn").unwrap().unwrap();
    let image_width = config.getint("images", "width").unwrap().unwrap() as i32;
    let image_height = config.getint("images", "height").unwrap().unwrap() as i32;
    let images_x = config.getint("images", "images_x").unwrap().unwrap() as i32;
    let images_y = config.getint("images", "images_y").unwrap().unwrap() as i32;
    let padding = config.getint("images", "padding").unwrap().unwrap() as i32;
    let iteration_count = config.getint("quality", "iteration_count").unwrap().unwrap() as i32;
    let min_width = config.getint("quality", "min_width").unwrap().unwrap() as i32;

    let nn = if use_sample_nn {
        NN::from_json(
            &fs::read_to_string("sample_nn.json")
                .expect("Couldn't read the file \"sample_nn.json\"")
        )
    } else {
        NN::from_json(
            &fs::read_to_string("nn.json")
                .expect("Couldn't read the file \"nn.json\"")
        )
    };

    let seed = rand::thread_rng().gen();
    println!("Seed: {}", seed);
    let mut rng = rand_xoshiro::Xoshiro256PlusPlus::seed_from_u64(seed);

    // ---------------------------------------------------------------------------------------------
    // SDL2 setup
    // ---------------------------------------------------------------------------------------------

    let sdl_context = sdl2::init()?;
    let video_subsys = sdl_context.video()?;

    let window = video_subsys
        .window(
            "CIEDE Gradients",
            ((image_width + padding) * images_x + padding) as u32,
            ((image_height + padding) * images_y + padding) as u32,
        )
        .position_centered()
        .opengl()
        .build()
        .map_err(|e| e.to_string())?;

    let mut canvas = window
        .into_canvas()
        .present_vsync()
        .build()
        .map_err(|e| e.to_string())?;

    let mut event_pump = sdl_context.event_pump()?;

    canvas.set_draw_color(sdl2::pixels::Color::RGB(0, 0, 0));
    canvas.clear();

    // ---------------------------------------------------------------------------------------------
    // Main loop
    // ---------------------------------------------------------------------------------------------

    let mut draw_once = true;
    let mut running = true;
    while running {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => {
                    running = false;
                }
                _ => {}
            }
        }

        if draw_once {
            draw_once = false;

            canvas.set_draw_color(sdl2::pixels::Color::RGB(0, 0, 0));
            canvas.clear();

            for iy in 0..images_y {
                let oy = padding + (image_height + padding) * iy;

                for ix in 0..images_x {
                    println!("Generating image {} of {}", iy * images_x + ix + 1, images_x * images_y);

                    let ox = padding + (image_width + padding) * ix;

                    let color1 = sdl2::pixels::Color::RGB(rng.gen(), rng.gen(), rng.gen());
                    let color2 = sdl2::pixels::Color::RGB(rng.gen(), rng.gen(), rng.gen());

                    println!("color1 = {:?}", color1);
                    println!("color2 = {:?}", color2);

                    let lab1 = sdl2_color_to_cielab(color1);
                    let lab2 = sdl2_color_to_cielab(color2);

                    println!("Generating...");

                    for px in 0..image_width {
                        if px == 0 {
                            canvas.set_draw_color(color1);
                            canvas.fill_rect(sdl2::rect::Rect::new(ox + px as i32, oy as i32, 1, image_height as u32)).unwrap();
                        } else if px == image_width - 1 {
                            canvas.set_draw_color(color2);
                            canvas.fill_rect(sdl2::rect::Rect::new(ox + px as i32, oy as i32, 1, image_height as u32)).unwrap();
                        } else {
                            let mut input = [color1.r, color1.g, color1.b, color2.r, color2.g, color2.b]
                                .iter()
                                .map(|c| *c as f64 / 255.0)
                                .collect::<Vec<_>>();

                            let interpolant = px as f64 / (image_width - 1) as f64;

                            input.push(px as f64 / (image_width - 1) as f64);

                            let output = nn.run(&input);

                            let c = output
                                .iter()
                                .map(|c| (c.clamp(0.0, 1.0) * 255.0).round() as u8)
                                .collect::<Vec<_>>();

                            let c = sdl2::pixels::Color::RGB(c[0], c[1], c[2]);

                            canvas.set_draw_color(c);
                            canvas.fill_rect(sdl2::rect::Rect::new(ox + px as i32, (oy + image_height / 2) as i32, 1, (image_height / 2) as u32)).unwrap();



                            let input_c1 = lab1;
                            let input_c2 = lab2;
                            let input_i = interpolant;

                            let width = min_width + rng.gen_range(0..=5);
                            let mut span = 210.0 + 5.0 * rng.gen::<f64>();
                            let mut increment = span / width as f64;
                            let mut min_l = -54.5 + rng.gen::<f64>();
                            let mut min_a = -99.5 + rng.gen::<f64>();
                            let mut min_b = -109.5 + rng.gen::<f64>();
                            let mut min_cost = f64::INFINITY;
                            let mut min_cost_oc = [0.0; 3];
                            for iteration in 0..iteration_count {
                                if iteration == 0 {
                                    for pl in 0..(width * 4) {
                                        let l = min_l + (pl as f64 + 0.5) * (increment / 4.0);

                                        for pa in 0..(width * 4) {
                                            let a = min_a + (pa as f64 + 0.5) * (increment / 4.0);

                                            for pb in 0..(width * 4) {
                                                let b = min_b + (pb as f64 + 0.5) * (increment / 4.0);

                                                let candidate_c = [l, a, b];

                                                if is_valid_cielab(&candidate_c) {
                                                    let cost = cost(&input_c1, &input_c2, input_i, &candidate_c);

                                                    if cost < min_cost {
                                                        min_cost = cost;
                                                        min_cost_oc = candidate_c;
                                                    }
                                                }
                                            }
                                        }
                                    }
                                } else {
                                    for pl in 0..width {
                                        let l = min_l + pl as f64 * increment;

                                        for pa in 0..width {
                                            let a = min_a + pa as f64 * increment;

                                            for pb in 0..width {
                                                let b = min_b + pb as f64 * increment;

                                                let candidate_c = [l, a, b];

                                                let cost = cost(&input_c1, &input_c2, input_i, &candidate_c);

                                                if cost < min_cost {
                                                    min_cost = cost;
                                                    min_cost_oc = candidate_c;
                                                }
                                            }
                                        }
                                    }
                                }

                                span *= 0.5;
                                increment = span / width as f64;
                                min_l = min_cost_oc[0] - span / 2.0;
                                min_a = min_cost_oc[1] - span / 2.0;
                                min_b = min_cost_oc[2] - span / 2.0;
                            }

                            let srgb = rgb_to_srgb(&cielab_to_rgb(&min_cost_oc));

                            let c = sdl2::pixels::Color::RGB(
                                (srgb[0].clamp(0.0, 1.0) * 255.0).round() as u8,
                                (srgb[1].clamp(0.0, 1.0) * 255.0).round() as u8,
                                (srgb[2].clamp(0.0, 1.0) * 255.0).round() as u8,
                            );

                            canvas.set_draw_color(c);
                            canvas.fill_rect(sdl2::rect::Rect::new(ox + px as i32, oy as i32, 1, (image_height / 2) as u32)).unwrap();
                        }

                        if (px + 1) % 50 == 0 {
                            println!("\tProgress: {} of {}", px + 1, image_width);
                        }
                    }
                }
            }

            println!("Done.");
        }

        canvas.present();
    }

    // ---------------------------------------------------------------------------------------------
    // Ending tasks
    // ---------------------------------------------------------------------------------------------

    Ok(())
}
