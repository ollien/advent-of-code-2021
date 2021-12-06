#![warn(clippy::all, clippy::pedantic)]
use std::env;
use std::fs;

fn part1(input: &[u32]) -> usize {
    const NUM_DAYS: u32 = 80;
    let mut laternfish = input.to_vec();
    for _ in 0..NUM_DAYS {
        let next = laternfish
            .iter()
            .flat_map(|&n| if n > 0 { vec![n - 1] } else { vec![6, 8] })
            .collect::<Vec<_>>();
        laternfish = next;
    }

    laternfish.len()
}

fn main() {
    let input_file_name = env::args().nth(1).expect("No input filename specified");
    let raw_input = fs::read_to_string(input_file_name).expect("Failed to read input file");
    let input: Vec<u32> = raw_input
        .trim_end()
        .split(',')
        .map(str::parse)
        .collect::<Result<_, _>>()
        .expect("Invalid number in input");

    println!("Part 1: {}", part1(&input));
}
