#![warn(clippy::all, clippy::pedantic)]
// Needed for auto_ops to work properly
#[allow(clippy::wildcard_imports)]
use auto_ops::*;
use fmt::Debug;
use std::collections::HashSet;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::{env, fmt};

// A wrapper for println that only prints in debug mode
macro_rules! dprintln {
    () => {
        #[cfg(feature = "debug_print")]
        println!() };
    ($($arg : tt) *) => {
        #[cfg(feature = "debug_print")]
        println!($($arg) *)
    };
}

#[derive(Clone)]
struct Board(Vec<Vec<u8>>);

/// Represesents -1/0/1 for the purposes of calculating adjacencies
// exists strictly to work around the limitation that I can't have a negative usize, nor do the additions
// without annoying conversions
#[derive(Clone, Copy, Debug)]
enum AdjacencyDelta {
    NegativeOne,
    Zero,
    One,
}

impl_op_ex_commutative!(+ |size: usize, delta: AdjacencyDelta| -> usize {
    match delta {
        AdjacencyDelta::NegativeOne => size - 1,
        AdjacencyDelta::Zero => size,
        AdjacencyDelta::One => size + 1
    }
});

impl Debug for Board {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (i, row) in self.0.iter().enumerate() {
            for tile in row {
                write!(f, "{:2}", tile)?;
            }

            if i != self.0.len() - 1 {
                writeln!(f)?;
            }
        }

        Ok(())
    }
}

impl Board {
    fn is_on_board(&self, (row, col): (usize, usize)) -> bool {
        // Flippy false positives here; you can't call flat_map on Option
        #[allow(clippy::map_flatten)]
        self.0
            .get(row)
            .map(|stored_row| stored_row.get(col))
            .flatten()
            .is_some()
    }

    /// Get the indices (row, col) of all adjacent items that are in the input
    /// Returns None if the given index is out of bounds
    fn get_adjacent_indices(&self, (row, col): (usize, usize)) -> Option<Vec<(usize, usize)>> {
        // This can't easily be a lazy iterator because we mutate the board as we go over this :(
        if !self.is_on_board((row, col)) {
            return None;
        }

        let mut res = vec![];
        for d_row in [
            AdjacencyDelta::NegativeOne,
            AdjacencyDelta::Zero,
            AdjacencyDelta::One,
        ] {
            // This is just more readable here, IMO
            #[allow(clippy::needless_continue)]
            #[allow(clippy::if_same_then_else)]
            for d_col in [
                AdjacencyDelta::NegativeOne,
                AdjacencyDelta::Zero,
                AdjacencyDelta::One,
            ] {
                if matches!(d_row, AdjacencyDelta::Zero) && matches!(d_col, AdjacencyDelta::Zero) {
                    continue;
                } else if matches!(d_row, AdjacencyDelta::NegativeOne) && row == 0 {
                    continue;
                } else if matches!(d_col, AdjacencyDelta::NegativeOne) && col == 0 {
                    continue;
                }

                let pos_candidate = (row + d_row, col + d_col);
                if self.is_on_board(pos_candidate) {
                    res.push(pos_candidate);
                }
            }
        }

        Some(res)
    }
}

/// Simulate a step of the simulation, and return the new board and the number of flashers
fn simulate_step(board: &Board) -> (Board, u32) {
    let mut next = board.clone();
    let mut num_flashes = 0;
    let mut active_flashers = HashSet::<(usize, usize)>::new();
    // Setup flashers
    for (i, row) in board.0.iter().enumerate() {
        #[allow(clippy::needless_continue)]
        for (j, &tile) in row.iter().enumerate() {
            if active_flashers.contains(&(i, j)) && tile < 9 {
                continue;
            } else if tile < 9 {
                next.0[i][j] = tile + 1;
                continue;
            }

            next.0[i][j] = 0;
            active_flashers.insert((i, j));
        }
    }

    dprintln!("pre-flash");
    dprintln!("{:?}", next);

    let mut flashed = HashSet::<(usize, usize)>::new();
    let mut to_flash = HashSet::<(usize, usize)>::new();
    // Flash!
    while !active_flashers.is_empty() {
        for &(row, col) in &active_flashers {
            num_flashes += 1;
            flashed.insert((row, col));
            let adj_indices = next
                .get_adjacent_indices((row, col))
                .expect("attempted to get adjacencies out of bounds");
            for (adj_row, adj_col) in adj_indices {
                let adj_tile = next.0[adj_row][adj_col];
                // Don't attempt to operate on something we need to flash
                if active_flashers.contains(&(adj_row, adj_col)) {
                    continue;
                }

                let next_adj_tile_val = adj_tile + 1;
                next.0[adj_row][adj_col] = next_adj_tile_val;
                if next_adj_tile_val > 9 && !flashed.contains(&(adj_row, adj_col)) {
                    to_flash.insert((adj_row, adj_col));
                }
            }
        }

        active_flashers = to_flash.clone();
        to_flash.clear();
    }

    // Reset all flashed items
    for (row, col) in flashed {
        next.0[row][col] = 0;
    }

    active_flashers.clear();
    dprintln!("post-flash");
    dprintln!("{:?}", next);
    dprintln!();

    (next, num_flashes)
}

fn part1(board: Board) -> u32 {
    let mut current_board = board;
    let mut total_flashes = 0;
    for _ in 0..100 {
        let (next_board, num_flashes) = simulate_step(&current_board);
        total_flashes += num_flashes;
        current_board = next_board;
    }

    total_flashes
}

fn part2(board: Board) -> u32 {
    let mut current_board = board;
    let mut num_steps = 0;
    loop {
        let (next_board, num_flashes) = simulate_step(&current_board);
        num_steps += 1;
        if usize::try_from(num_flashes).unwrap() == next_board.0.len() * next_board.0[0].len() {
            return num_steps;
        }

        current_board = next_board;
    }
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
                        .unwrap_or_else(|| panic!("Got non-numeric char {} in input", c))
                        .try_into()
                        // This literally cannot fail wht a digit
                        .unwrap()
                })
                .collect::<Vec<u8>>()
        })
        .collect::<Vec<_>>();

    let board = Board(input_lines);
    println!("Part 1: {}", part1(board.clone()));
    println!("Part 2: {}", part2(board));
}
