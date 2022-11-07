use std::fs;
use std::fs::File;
use std::io::Write;
use std::time::Instant;
use configparser::ini::Ini;
use rand::{Rng, SeedableRng};
use crate::color::*;

mod color;

fn cost(ic1: &[f64; 3], ic2: &[f64; 3], ii: f64, oc: &[f64; 3]) -> f64 {
    let de1 = cielab_dist_ciede2000(&ic1, &oc);
    let de2 = cielab_dist_ciede2000(&ic2, &oc);
    let oi = de1 / (de1 + de2);

    (de1 + de2) * ((oi - ii).abs() + 0.5)
}

fn main() {
    let config_str = fs::read_to_string("config.ini").expect("Error while reading the configuration file.");
    let mut config = Ini::new();
    let _ = config.read(config_str);

    let sample_count = config.getint("general", "sample_count").unwrap().unwrap() as i32;
    let iteration_count = config.getint("quality", "iteration_count").unwrap().unwrap() as i32;
    let min_width = config.getint("quality", "min_width").unwrap().unwrap() as i32;

    let seed = rand::thread_rng().gen();
    println!("Seed: {}", seed);
    let mut rng = rand_xoshiro::Xoshiro256PlusPlus::seed_from_u64(seed);

    let mut output = File::create(format!("data_{}.csv", seed)).unwrap();

    println!("Generating samples...");

    let instant = Instant::now();

    let mut str = String::new();
    let mut sample_idx = 0;
    'outer_loop:
        while sample_idx < sample_count {
            let input_c1 = random_cielab(&mut rng);
            let input_c2 = random_cielab(&mut rng);
            let input_i: f64 = rng.gen();

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

            let output_c = min_cost_oc;

            if !is_valid_cielab(&output_c) {
                continue 'outer_loop;
            }

            {
                let de1 = cielab_dist_ciede2000(&input_c1, &output_c);
                let de2 = cielab_dist_ciede2000(&input_c2, &output_c);
                let output_i = de1 / (de1 + de2);
                let (min_i, max_i) = if input_i >= output_i {
                    (output_i, input_i)
                } else {
                    (input_i, output_i)
                };
                if max_i / min_i > 1.001 {
                    continue 'outer_loop;
                }
            }

            let input_c1 = rgb_to_srgb(&cielab_to_rgb(&input_c1));
            let input_c2 = rgb_to_srgb(&cielab_to_rgb(&input_c2));
            let output_c = rgb_to_srgb(&cielab_to_rgb(&output_c));

            str += &format!("{}, {}, {}, {}, {}, {}, {}, {}, {}, {}\n", input_c1[0], input_c1[1], input_c1[2], input_c2[0], input_c2[1], input_c2[2], input_i, output_c[0], output_c[1], output_c[2]);

            sample_idx += 1;

            if sample_idx % 1000 == 0 {
                write!(output, "{}", str).unwrap();
                str = String::new();
                println!("\tSamples so far: {} of {}", sample_idx, sample_count);
            }
        }

    if !str.is_empty() {
        println!("\tSamples so far: {} of {}", sample_count, sample_count);
        write!(output, "{}", str).unwrap();
    }

    let elapsed = instant.elapsed();
    println!("Duration: {:.2?}", elapsed);
}
