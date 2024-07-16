use memchr::*;
use memmap2::Mmap;
use rustc_hash::FxHashMap;
use std::collections::BTreeMap;
use std::fs::File;
use std::io::{self, Write};

#[derive(Debug)]
struct Measurement {
    min: f32,
    max: f32,
    sum: f32,
    count: u32,
}

fn main() {
    let mut accumulator: FxHashMap<&str, Measurement> = FxHashMap::default();

    let file = File::open("./measurements.txt").unwrap();
    let buf = unsafe { Mmap::map(&file).unwrap() };
    assert!(buf.last() == Some(&b'\n'));

    let mut start: usize = 0;
    for index in memchr_iter(b'\n', &buf) {
        process_line(&buf[start..index], &mut accumulator);
        start = index + 1;
    }

    let mut ordered_accumulator: BTreeMap<&str, Measurement> = BTreeMap::new();
    ordered_accumulator.extend(accumulator);

    print_results(&ordered_accumulator);
}

fn process_line<'a>(line: &'a [u8], accumulator: &mut FxHashMap<&'a str, Measurement>) {
    let index = memchr(b';', line).unwrap();
    let city = &line[..index];
    let key = unsafe { std::str::from_utf8_unchecked(city) };
    let temperature = fast_float::parse::<f32, _>(&line[index + 1..]).unwrap();

    match accumulator.get_mut(&key) {
        Some(measurement) => {
            update_measurement(measurement, temperature);
        }
        None => {
            accumulator.insert(
                key,
                Measurement {
                    min: temperature,
                    max: temperature,
                    sum: temperature,
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
