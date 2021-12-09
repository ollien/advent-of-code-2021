#![warn(clippy::all, clippy::pedantic)]
use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader};

fn part1(input: &[Vec<u32>]) -> u32 {
    input
        .iter()
        .enumerate()
        .map(|(depth, row)| {
            println!("Depth: {}", depth);
            row.iter()
                .enumerate()
                .map(|(i, &item)| {
                    let adjacent = {
                        let mut res = vec![
                            row.get(i + 1),
                            input.get(depth + 1).map(|below_row| {
                                below_row
                                    .get(i)
                                    .expect("below row does not match row length")
                            }),
                        ];

                        if i > 0 {
                            let left = row.get(i - 1);
                            res.push(left);
                        }
                        if depth > 0 {
                            let above = input.get(depth - 1).map(|above_row| {
                                above_row
                                    .get(i)
                                    .expect("above row does not match row length")
                            });
                            res.push(above);
                        }

                        res
                    };

                    // println!("{}, {}", depth, i);
                    if adjacent
                        .into_iter()
                        .filter(Option::is_some)
                        .all(|adjacent_item| item < *adjacent_item.unwrap())
                    {
                        // println!("{} => 1 + {}", item, height);
                        1 + item
                    } else {
                        0
                    }
                })
                .sum()
        })
        .sum()
}

fn main() {
    let input_file_name = env::args().nth(1).expect("No input filename specified");
    let input_file = File::open(input_file_name).expect("Could not open input file");
    let input_lines = BufReader::new(input_file)
        .lines()
        .map(|res| res.expect("Failed to read line"))
        .map(|s| {
            s.chars()
                .map(|c| {
                    c.to_digit(10)
                        .unwrap_or_else(|| panic!("Expected all chars to be digits, found {}", c))
                })
                .collect::<Vec<u32>>()
        })
        .collect::<Vec<_>>();

    println!("Part 1: {}", part1(&input_lines));
}
