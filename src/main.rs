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
    let file = File::open("./measurements.txt").unwrap();
    let data = unsafe { Mmap::map(&file).unwrap() };
    let num_threads = std::thread::available_parallelism().unwrap().get();

    let chunks = get_chunks(&data, num_threads);

    std::thread::scope(|s| {
        let mut threads_handle = Vec::with_capacity(num_threads);
        for (start, end) in chunks {
            let chunk = &data[start..end];
            let t = s.spawn(|| {
                let mut chunk_accumulator: FxHashMap<&str, Measurement> = FxHashMap::default();
                let mut start = 0;
                for index in memchr_iter(b'\n', chunk) {
                    process_line(&chunk[start..index], &mut chunk_accumulator);
                    start = index + 1;
                }
                chunk_accumulator
            });
            threads_handle.push(t);
        }

        let mut ordered_accumulator: BTreeMap<&str, Measurement> = BTreeMap::new();
        for thread in threads_handle {
            let chunk_accumulator = thread.join().unwrap();
            for (key, measurement) in chunk_accumulator {
                ordered_accumulator
                    .entry(key)
                    .and_modify(|e| {
                        e.min = e.min.min(measurement.min);
                        e.max = e.max.max(measurement.max);
                        e.sum += measurement.sum;
                        e.count += measurement.count;
                    })
                    .or_insert(measurement);
            }
        }

        print_results(&ordered_accumulator);
    });
}

fn get_chunks(data: &[u8], num_threads: usize) -> Vec<(usize, usize)> {
    let chunk_size = data.len() / num_threads;
    let mut chunks = Vec::with_capacity(num_threads);
    let mut start = 0;
    for _ in 0..num_threads - 1 {
        let end = start + chunk_size;
        match memchr::memchr(b'\n', &data[end..]) {
            Some(pos) => {
                chunks.push((start, end + pos));
                start = end + pos + 1;
            }
            None => {
                chunks.push((start, data.len()));
                break;
            }
        }
    }
    chunks
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
