use std::collections::BTreeMap;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Write};

#[derive(Debug)]
struct Measurement {
    min: f32,
    max: f32,
    sum: f32,
    count: u32,
}

fn main() {
    let mut accumulator: BTreeMap<String, Measurement> = BTreeMap::new();
    let file = File::open("./measurements.txt").unwrap();
    let reader = BufReader::new(file);
    for line in reader.lines() {
        process_line(line.unwrap(), &mut accumulator);
    }

    print_results(&accumulator);
}

fn process_line(line: String, accumulator: &mut BTreeMap<String, Measurement>) {
    let parts: Vec<&str> = line.split(";").collect();
    let key = parts[0];
    let value = parts[1].parse::<f32>().unwrap();

    match accumulator.get_mut(key) {
        Some(measurement) => {
            update_measurement(measurement, value);
        }
        None => {
            accumulator.insert(
                key.to_string(),
                Measurement {
                    min: value,
                    max: value,
                    sum: value,
                    count: 1,
                },
            );
        }
    }
}

fn update_measurement(measurement: &mut Measurement, value: f32) {
    measurement.min = measurement.min.min(value);
    measurement.max = measurement.max.max(value);
    measurement.sum += value;
    measurement.count += 1;
}

fn print_results(accumulator: &BTreeMap<String, Measurement>) {
    let stdout = io::stdout();
    let mut handle = stdout.lock();
    for (key, measurement) in accumulator.iter() {
        let average = measurement.sum / measurement.count as f32;
        writeln!(
            handle,
            "{}={:.1}/{:.1}/{:.1}",
            key, measurement.min, measurement.max, average
        )
        .unwrap();
    }
}
