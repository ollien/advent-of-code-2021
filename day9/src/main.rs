#![warn(clippy::all, clippy::pedantic)]
use itertools::Itertools;
use std::collections::{HashSet, VecDeque};
use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader};
use thiserror::Error;

#[derive(Error, Debug)]
enum Error {
    #[error("Tried to get out of bounds row, {0}")]
    RowOutOfRange(usize),
}

/// Get the indices (row, col) of all adjacent items that are in the input
fn get_adjacent_indices(
    input: &[Vec<u32>],
    (depth, col): (usize, usize),
) -> Result<Vec<(usize, usize)>, Error> {
    let maybe_row = input.get(depth);
    if maybe_row.is_none() {
        return Err(Error::RowOutOfRange(depth));
    }

    let row = maybe_row.unwrap();
    let mut res = vec![];

    if col > 0 {
        res.push((depth, col - 1));
    }

    if row.get(col + 1).is_some() {
        res.push((depth, col + 1));
    }

    let below = input.get(depth + 1).map(|below_row| {
        below_row
            .get(col)
            .expect("below row does not match row length")
    });
    if below.is_some() {
        res.push((depth + 1, col));
    }

    let have_above_item = || {
        input
            .get(depth - 1)
            .map(|above_row| {
                above_row
                    .get(col)
                    .expect("above row does not match row length")
            })
            .is_some()
    };
    if depth > 0 && have_above_item() {
        res.push((depth - 1, col));
    }

    Ok(res)
}

/// Get the indices of all low points in the input
fn find_low_point_indices(input: &[Vec<u32>], depth: usize) -> Result<Vec<usize>, Error> {
    let maybe_row = input.get(depth);
    if maybe_row.is_none() {
        return Err(Error::RowOutOfRange(depth));
    }

    let row = maybe_row.unwrap();
    let mut res = vec![];
    for (i, &item) in row.iter().enumerate() {
        let adjacent_res = get_adjacent_indices(input, (depth, i));
        if let Err(err) = adjacent_res {
            return Err(err);
        }

        if adjacent_res
            .unwrap()
            .into_iter()
            .all(|(adjacent_depth, adjacent_col)| item < input[adjacent_depth][adjacent_col])
        {
            res.push(i);
        }
    }

    Ok(res)
}

fn part1(input: &[Vec<u32>]) -> u32 {
    input
        .iter()
        .enumerate()
        .map(|(depth, row)| {
            find_low_point_indices(input, depth)
                .expect("failed to get row for depth analysis")
                .into_iter()
                .map(|idx| row[idx] + 1)
                .sum::<u32>()
        })
        .sum()
}

fn part2(input: &[Vec<u32>]) -> u32 {
    let basin_sizes = input.iter().enumerate().flat_map(|(depth, _)| {
        let low_points =
            find_low_point_indices(input, depth).expect("failed to get row for depth analysis");

        low_points.into_iter().map(move |low_point_idx| {
            let mut to_visit = [(depth, low_point_idx)]
                .into_iter()
                .collect::<VecDeque<_>>();
            let mut visited = HashSet::<(usize, usize)>::new();
            // 1 includes the low point
            let mut num_in_basin = 1;

            // Flood the board, terminating our search once we hit a nine
            while !to_visit.is_empty() {
                let (visiting_row, visiting_col) = to_visit.pop_front().unwrap();
                let visiting = input[visiting_row][visiting_col];
                let adjacent_iter = get_adjacent_indices(input, (visiting_row, visiting_col))
                    .expect("failed to get adjacent items for bfs");
                for (adjacent_row, adjacent_col) in adjacent_iter {
                    if visited.contains(&(adjacent_row, adjacent_col)) {
                        continue;
                    }

                    let adjacent = input[adjacent_row][adjacent_col];
                    // flows from high to low, nine can't be part of the basin
                    if adjacent > visiting && adjacent != 9 {
                        num_in_basin += 1;
                        to_visit.push_back((adjacent_row, adjacent_col));
                        visited.insert((adjacent_row, adjacent_col));
                    }
                }
            }

            num_in_basin
        })
    });

    basin_sizes.sorted().rev().take(3).product()
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

    let first_row_length = input_lines.get(0).expect("input must be non-empty").len();
    assert!(
        input_lines.iter().all(|row| row.len() == first_row_length),
        "All input lines must be the same length"
    );

    println!("Part 1: {}", part1(&input_lines));
    println!("Part 2: {}", part2(&input_lines));
}
