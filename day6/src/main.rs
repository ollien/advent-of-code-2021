#![warn(clippy::all, clippy::pedantic)]
use std::env;
use std::fs;

fn simulate(input: &[u8], num_days: u32) -> u64 {
    // A "map" of each day value of the laternfishes, 0-8 (hence size 9).
    let mut day_map = [0_u64; 9];

    #[allow(clippy::cast_possible_truncation, clippy::naive_bytecount)]
    day_map.iter_mut().enumerate().for_each(|(i, n)| {
        // i can be safely cast because we know that i will not exceed 8
        //
        // The count can be safely cast because we know that it's going to be pretty small given the input size
        // (at most 300 if every element is the same, at least on my input)
        //
        // We _could_ use the bytecount crate to satisfy clippy's naive_bytecount warning, but the actual input
        // is like, 300 elements long so any performance loss due to not using it is going to be super neglibible
        *n = input.iter().filter(|&&n| n == i as u8).count() as u64;
    });

    for _ in 0..num_days {
        let to_add = day_map[0];
        // Shift all elements down one
        for i in 1..day_map.len() {
            day_map[i - 1] = day_map[i];
        }

        // Add the newly spawned elements (n=8 starts at 0)
        day_map[6] += to_add;
        day_map[8] = to_add;
    }

    day_map.iter().sum()
}

fn main() {
    let input_file_name = env::args().nth(1).expect("No input filename specified");
    let raw_input = fs::read_to_string(input_file_name).expect("Failed to read input file");
    let input = raw_input
        .trim_end()
        .split(',')
        .map(str::parse::<u8>)
        .collect::<Result<Vec<_>, _>>()
        .expect("Invalid number in input");

    println!("Part 1: {}", simulate(&input, 80));
    println!("Part 2: {}", simulate(&input, 256));
}
