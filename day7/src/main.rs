#![warn(clippy::all, clippy::pedantic)]
use std::env;
use std::fs;

enum Part {
    Part1,
    Part2,
}

fn run(input: &[i32], part: &Part) -> i32 {
    let smallest = *input
        .iter()
        .min()
        .expect("Input should have more than zero elements");
    let largest = *input
        .iter()
        .max()
        .expect("Input should have more than zero elements");

    // This assumes that the answer is in the range of the min/max of the crab positions,
    // but that seems exceedingly likely. If this puzzle were more complex I might add/sub
    // "largest" on each of the bounds
    //
    // Honestly I didn't expect this to work on both parts
    let costs = (smallest..largest)
        .map(|possible_destination| {
            input
                .iter()
                .map(|crab_location| {
                    let steps = (crab_location - possible_destination).abs();
                    match part {
                        Part::Part1 => steps,
                        // 1 + 2 + 3 + ... + n => n(n+1)/2
                        Part::Part2 => steps * (steps + 1) / 2,
                    }
                })
                .sum()
        })
        .collect::<Vec<i32>>();

    *costs
        .iter()
        .min()
        .expect("fuels list has size equal to zero")
}

fn main() {
    let input_file_name = env::args().nth(1).expect("No input filename specified");
    let raw_input = fs::read_to_string(input_file_name).expect("Failed to read input file");

    let input = raw_input
        .trim_end()
        .split(',')
        .map(str::parse::<i32>)
        .collect::<Result<Vec<_>, _>>()
        .expect("Invalid number in input");

    println!("Part 1: {}", run(&input, &Part::Part1));
    println!("Part 2: {}", run(&input, &Part::Part2));
}
