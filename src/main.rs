use std::collections::BTreeMap;
use std::fs::File;
use std::io::{self, Read, Write};

#[derive(Debug)]
struct Measurement {
    min: f32,
    max: f32,
    sum: f32,
    count: u32,
}

fn main() {
    let mut accumulator: BTreeMap<&str, Measurement> = BTreeMap::new();
    let mut buf = vec![];

    let mut file = File::open("./measurements.txt").unwrap();
    file.read_to_end(&mut buf).unwrap();
    assert!(buf.pop() == Some(b'\n'));

    for line in buf.split(|&c| c == b'\n') {
        process_line(line, &mut accumulator);
    }

    print_results(&accumulator);
}

fn process_line<'a>(line: &'a [u8], accumulator: &mut BTreeMap<&'a str, Measurement>) {
    let parts = line.split(|&c| c == b';').collect::<Vec<_>>();
    let key = unsafe { std::str::from_utf8_unchecked(parts[0]) };
    let value = unsafe {
        std::str::from_utf8_unchecked(parts[1])
            .parse::<f32>()
            .unwrap()
    };

    match accumulator.get_mut(&key) {
        Some(measurement) => {
            update_measurement(measurement, value);
        }
        None => {
            accumulator.insert(
                key,
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

fn print_results(accumulator: &BTreeMap<&str, Measurement>) {
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
