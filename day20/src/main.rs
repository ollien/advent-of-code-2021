#![warn(clippy::all, clippy::pedantic)]

use itertools::Itertools;
use std::collections::BTreeMap;
use std::env;
use std::fmt::Debug;
use std::fs::File;
use std::io::{BufRead, BufReader};
use thiserror::Error;

const BRIGHT_CHAR: char = '#';
const DARK_CHAR: char = '.';

#[derive(Error, Debug)]
enum ParseError {
    #[error("invalid character '{0}' encountered")]
    InvalidChar(char),
    #[error("board cannot fit into an index with an isize")]
    BoardTooBig,
}

#[derive(Error, Debug)]
enum SimulationError {
    #[error("Attempted to look up address {0} in enhancement algorithm, which doesn't exist")]
    InvalidAddress(u16),
}

#[derive(Debug, Copy, Clone)]
enum BoardTile {
    Dark,
    Bright,
}

#[derive(Clone)]
struct Board(BTreeMap<(isize, isize), BoardTile>);

impl From<BoardTile> for u8 {
    fn from(tile: BoardTile) -> Self {
        match tile {
            BoardTile::Dark => 0,
            BoardTile::Bright => 1,
        }
    }
}

impl From<BoardTile> for char {
    fn from(tile: BoardTile) -> Self {
        match tile {
            BoardTile::Dark => DARK_CHAR,
            BoardTile::Bright => BRIGHT_CHAR,
        }
    }
}

impl TryFrom<char> for BoardTile {
    type Error = ParseError;

    fn try_from(c: char) -> Result<Self, Self::Error> {
        match c {
            BRIGHT_CHAR => Ok(BoardTile::Bright),
            DARK_CHAR => Ok(BoardTile::Dark),
            _ => Err(ParseError::InvalidChar(c)),
        }
    }
}

impl Board {
    fn from_input<S: AsRef<str>>(input: &[S]) -> Result<Self, ParseError> {
        let mut board_map = BTreeMap::new();

        for (i, row) in input.iter().enumerate() {
            for (j, c) in row.as_ref().chars().enumerate() {
                let board_tile = BoardTile::try_from(c)?;
                let row = isize::try_from(i).map_err(|_| ParseError::BoardTooBig)?;
                let col = isize::try_from(j).map_err(|_| ParseError::BoardTooBig)?;

                board_map.insert((row, col), board_tile);
            }
        }

        let board = Self(board_map);

        Ok(board)
    }

    fn get_enhancement_address(&self, row: isize, col: isize, default: BoardTile) -> u16 {
        let mut address = 0;
        for d_row in -1..=1 {
            for d_col in -1..=1 {
                let tile = self.0.get(&(d_row + row, d_col + col)).unwrap_or(&default);

                let bit = u8::from(*tile);
                address = address * 2 + u16::from(bit);
            }
        }

        address
    }

    /// Get the bounds of the defined part of the board in two inclusive ranges, row, column
    /// If the board is empty, an empty Option is returned.
    fn get_bounds(&self) -> Option<((isize, isize), (isize, isize))> {
        let row_minmax = self.0.keys().map(|(row, _)| row).minmax();
        let row_range = match row_minmax {
            itertools::MinMaxResult::MinMax(&min, &max) => (min, max),
            itertools::MinMaxResult::OneElement(&n) => (n, n),
            itertools::MinMaxResult::NoElements => return None,
        };

        let col_minmax = self.0.keys().map(|(row, _)| row).minmax();
        let col_range = match col_minmax {
            itertools::MinMaxResult::MinMax(&min, &max) => (min, max),
            itertools::MinMaxResult::OneElement(&n) => (n, n),
            itertools::MinMaxResult::NoElements => return None,
        };

        Some((row_range, col_range))
    }

    fn enhance(
        self,
        enhancement_algorithm: &[BoardTile],
        default: BoardTile,
    ) -> Result<Self, SimulationError> {
        let maybe_bounds = self.get_bounds();
        if maybe_bounds.is_none() {
            // This is an empty board, so we just return ourselves and move on
            // Realistically, this won't happen
            return Ok(self);
        }

        let (row_bounds, col_bounds) = maybe_bounds.unwrap();

        // The board is still valid in the position where the very corners of the infinite board
        // are the corners of the bound being swept, given that the board it is infinite.
        let adjusted_row_bounds = (row_bounds.0 - 1, row_bounds.1 + 1);
        let adjusted_col_bounds = (col_bounds.0 - 1, col_bounds.1 + 1);

        let mut enhanced_board_map = BTreeMap::new();
        for row in adjusted_row_bounds.0..=adjusted_row_bounds.1 {
            for col in adjusted_col_bounds.0..=adjusted_col_bounds.1 {
                let enhancement_address = self.get_enhancement_address(row, col, default);
                let enhanced_tile_candidate =
                    enhancement_algorithm.get(usize::from(enhancement_address));

                if enhanced_tile_candidate.is_none() {
                    return Err(SimulationError::InvalidAddress(enhancement_address));
                }

                enhanced_board_map.insert((row, col), *enhanced_tile_candidate.unwrap());
            }
        }

        Ok(Self(enhanced_board_map))
    }
}

impl Debug for Board {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let maybe_bounds = self.get_bounds();
        if maybe_bounds.is_none() {
            return Ok(());
        }

        let (row_bounds, col_bounds) = maybe_bounds.unwrap();
        for row in row_bounds.0..=row_bounds.1 {
            for col in col_bounds.0..=col_bounds.1 {
                let tile = self.0.get(&(row, col)).unwrap_or(&BoardTile::Dark);
                write!(formatter, "{}", char::from(*tile))?;
            }

            if row != row_bounds.1 {
                writeln!(formatter)?;
            }
        }

        Ok(())
    }
}

fn parse_enhancement_algorithm(input_algorithm: &str) -> Result<Vec<BoardTile>, ParseError> {
    input_algorithm.chars().map(char::try_into).collect()
}

fn run(mut board: Board, enhancement_algorithtm: &[BoardTile], num_iterations: u32) -> usize {
    let all_zeroes_transformation = enhancement_algorithtm[0];
    let all_ones_transformation = enhancement_algorithtm[enhancement_algorithtm.len() - 1];
    let (evens_default, odds_default) = {
        match (all_zeroes_transformation, all_ones_transformation) {
            // If, after encountering either an all dark or all bright region, we turn bright,
            // after the first step the default should always be bright.
            (BoardTile::Bright, BoardTile::Bright) => (BoardTile::Bright, BoardTile::Bright),
            // If, after encountering either an all dark region we turn bright, and vice versa,
            // we must alternate on every step
            (BoardTile::Bright, BoardTile::Dark) => (BoardTile::Dark, BoardTile::Bright),
            // But if dark always maps to dark, there's no need to change anything.
            (BoardTile::Dark, _) => (BoardTile::Dark, BoardTile::Dark),
        }
    };

    for i in 0..num_iterations {
        let default = if i == 0 {
            BoardTile::Dark
        } else if i % 2 == 0 {
            evens_default
        } else {
            odds_default
        };

        board = board
            .enhance(enhancement_algorithtm, default)
            .expect("failed to run simulation");
    }

    board
        .0
        .values()
        .filter(|tile| matches!(tile, &BoardTile::Bright))
        .count()
}

fn main() {
    let input_file_name = env::args().nth(1).expect("No input filename specified");
    let input_file = File::open(input_file_name).expect("Could not open input file");
    let mut lines_iter = BufReader::new(input_file).lines();

    let raw_enhancement_algorithm = lines_iter
        .next()
        .expect("Failed to read input for algorithm line")
        .expect("No enhancement algorithm present");

    let raw_board = lines_iter
        .skip(1)
        .map(|res| res.expect("Failed to read input"))
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>();

    let board = Board::from_input(&raw_board).expect("Failed to parse board");
    let enhancement_algorithm = parse_enhancement_algorithm(&raw_enhancement_algorithm)
        .expect("Failed to parse input algorithm");

    println!("Part 1: {}", run(board.clone(), &enhancement_algorithm, 2));
    println!("Part 2: {}", run(board, &enhancement_algorithm, 50));
}
