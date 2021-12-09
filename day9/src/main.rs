#![warn(clippy::all, clippy::pedantic)]
use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader};
use thiserror::Error;

#[derive(Error, Debug)]
enum Error {
    #[error("Tried to get out of bounds row, {0}")]
    RowOutOfRange(usize),
}

fn find_low_point_indices(
    input: &[Vec<u32>],
    depth: usize,
) -> Result<impl Iterator<Item = usize> + '_, Error> {
    let maybe_row = input.get(depth);
    if maybe_row.is_none() {
        return Err(Error::RowOutOfRange(depth));
    }

    let row = maybe_row.unwrap();
    let iter = row.iter().enumerate().filter_map(move |(i, &item)| {
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

        if adjacent
            .into_iter()
            .filter(Option::is_some)
            .all(|adjacent_item| item < *adjacent_item.unwrap())
        {
            Some(i)
        } else {
            None
        }
    });

    Ok(iter)
}

fn part1(input: &[Vec<u32>]) -> u32 {
    input
        .iter()
        .enumerate()
        .map(|(depth, row)| {
            find_low_point_indices(input, depth)
                .expect("failed to get row for depth analysis")
                .map(|idx| row[idx] + 1)
                .sum::<u32>()
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
