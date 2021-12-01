use std::collections::VecDeque;
use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader};

fn part1(items: &[i32]) -> i32 {
    let mut prev: Option<i32> = None;
    let mut num_increasing = 0;
    for &item in items {
        if let Some(prev_item) = prev {
            if item > prev_item {
                num_increasing += 1;
            }
        }

        prev = Some(item);
    }

    num_increasing
}

fn part2(items: &[i32]) -> i32 {
    const WINDOW_SIZE: usize = 3;

    let mut num_increasing = 0;
    let mut window = VecDeque::<i32>::new();
    // Initialize the window with the first few elements
    window.extend(&items[..WINDOW_SIZE]);
    let mut last_window_sum = window.iter().sum::<i32>();
    for &item in &items[WINDOW_SIZE - 1..] {
        window.pop_back();
        window.push_front(item);

        let current_sum = window.iter().sum();
        if current_sum > last_window_sum {
            num_increasing += 1;
        }

        last_window_sum = current_sum;
    }

    num_increasing
}

fn main() {
    let input_file_name = env::args().nth(1).expect("No input filename specified");
    let input_file = File::open(input_file_name).expect("Could not open input file");
    let items = BufReader::new(input_file)
        .lines()
        .map(|line_res| {
            let line = line_res.expect("failed to read a line from the input file");
            line.parse::<i32>()
                .unwrap_or_else(|_| panic!("Failed to convert input line '{}' to integer", line))
        })
        .collect::<Vec<i32>>();

    println!("Part 1: {}", part1(&items));
    println!("Part 2: {}", part2(&items));
}
