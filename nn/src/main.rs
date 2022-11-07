use std::fs;
use std::fs::File;
use std::io::{Write, BufRead, BufReader};
use std::time::Instant;
use configparser::ini::Ini;
use nn::{NN, HaltCondition};

fn main() {
    let config_str = fs::read_to_string("config.ini").expect("Error while reading the configuration file.");
    let mut config = Ini::new();
    let _ = config.read(config_str);

    let use_sample_data = config.getbool("general", "use_sample_data").unwrap().unwrap();
    let epochs = config.getuint("general", "epochs").unwrap().unwrap() as u32;
    let test_sample_count = config.getuint("general", "test_sample_count").unwrap().unwrap() as usize;
    let hidden_layer_count = config.getint("hidden_layers", "layer_count").unwrap().unwrap() as i32;
    let hidden_layer_width = config.getuint("hidden_layers", "layer_width").unwrap().unwrap() as u32;

    let buffered = if use_sample_data {
        BufReader::new(File::open("sample_data.csv").unwrap())
    } else {
        BufReader::new(File::open("data.csv").unwrap())
    };

    let mut data = Vec::new();
    for line in buffered.lines() {
        match line {
            Ok(line) => {
                if line.is_empty() {
                    continue;
                }

                let nums = line.split(", ").map(|s| s.parse::<f64>().expect("Found an invalid sample.")).collect::<Vec<_>>();
                if nums.len() != 10 {
                    panic!("Found an invalid sample.");
                }

                let (input, output) = nums.split_at(7);

                data.push((input.to_vec(), output.to_vec()));
            },
            Err(_) => {},
        }
    }

    println!("Valid sample count: {}", data.len());

    let (training_data, test_data) = data.split_at(data.len() - test_sample_count);

    let mut layers = vec![7];
    for _ in 0..hidden_layer_count {
        layers.push(hidden_layer_width);
    }
    layers.push(3);

    let mut net = NN::new(&layers);

    let instant = Instant::now();

    net.train(&training_data)
        .halt_condition( HaltCondition::Epochs(epochs) )
        .log_interval( Some(1) )
        //.momentum( 0.1 )
        .rate( 0.3 )
        .go();

    println!("Elapsed: {:?}", instant.elapsed());

    write!(File::create("nn.json").unwrap(), "{}", net.to_json()).unwrap();

    let mut sum = 0.0;
    for &(ref inputs, ref outputs) in test_data.iter() {
        let results = net.run(inputs);

        let mut current_sum = 0.0;
        for (&result, &target) in results.iter().zip(outputs.iter()) {
            current_sum += (target - result).powi(2);
        }
        sum += current_sum / (results.len() as f64)
    }
    println!("Test sample error: {}", sum / test_data.len() as f64);
}
