#![warn(clippy::all, clippy::pedantic)]

use nom::{
    bytes::complete::{tag, take_while1},
    character::complete::char,
    combinator::{eof, fail, map_res, opt},
    multi::{many0, separated_list1},
    sequence::{preceded, separated_pair, tuple},
    IResult,
};
use std::collections::VecDeque;
use std::env;
use std::fmt;
use std::fmt::{Debug, Display, Formatter};
use std::fs;

const BOARD_SIZE: usize = 5;

#[derive(Debug, Clone, Copy)]
enum BingoTile {
    Unmarked(u8),
    Marked(u8),
}

#[derive(Clone)]
struct BingoBoard([[BingoTile; BOARD_SIZE]; BOARD_SIZE]);

#[derive(Debug, Clone)]
struct Input {
    calls: Vec<u8>,
    boards: Vec<BingoBoard>,
}

/// `BoardState` indicates whether or not a board has won
struct BoardState {
    won: bool,
    board: BingoBoard,
}

struct BingoGame {
    calls: VecDeque<u8>,
    boards: Vec<BoardState>,
}

/// `BingoPlayer` is an iterator that will iterate over the successive winners of a bingo game.
struct BingoPlayer<'a> {
    game: &'a mut BingoGame,
}

impl Display for BingoTile {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let width = f.width();
        match self {
            BingoTile::Marked(_) => write!(f, "{:>width$}", "x", width = width.unwrap_or(1)),
            BingoTile::Unmarked(n) => {
                write!(
                    f,
                    "{:>width$}",
                    n,
                    width = width.unwrap_or_else(|| n.to_string().len())
                )
            }
        }
    }
}

impl Debug for BingoBoard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for row in self.0 {
            for tile in row {
                write!(f, "{:2} ", tile)?;
            }

            writeln!(f)?;
        }

        Ok(())
    }
}

impl From<Input> for BingoGame {
    /// Start a new game from the given puzzle input
    fn from(input: Input) -> Self {
        let calls = VecDeque::from(input.calls);
        let boards = input
            .boards
            .into_iter()
            .map(|board| BoardState { won: false, board })
            .collect();

        Self { calls, boards }
    }
}

impl BingoGame {
    /// Return an iterator to play this bingo game
    fn play(&mut self) -> BingoPlayer {
        BingoPlayer { game: self }
    }
}

impl BingoBoard {
    /// Check if this board has won the game
    fn is_winner(&self) -> bool {
        assert_eq!(
            self.0.len(),
            BOARD_SIZE,
            "Board number of rows doesn't match expected size"
        );
        assert_eq!(
            self.0[0].len(),
            BOARD_SIZE,
            "Board number of cols doesn't match expected size"
        );

        for col in 0..BOARD_SIZE {
            let mut won_by_col = true;
            for row in 0..BOARD_SIZE {
                let won_by_row = self.0[row]
                    .iter()
                    .all(|item| matches!(item, BingoTile::Marked(_)));
                if won_by_row {
                    return true;
                } else if !matches!(self.0[row][col], BingoTile::Marked(_)) {
                    won_by_col = false;
                    break;
                }
            }
            if won_by_col {
                return true;
            }
        }

        false
    }

    /// Mark the given number on the board, if it exists
    fn mark_n(&mut self, n: u8) {
        for row in &mut self.0 {
            for tile in row {
                if let BingoTile::Unmarked(tile_n) = tile {
                    if *tile_n == n {
                        *tile = BingoTile::Marked(n);
                    }
                }
            }
        }
    }
}

impl<'a> Iterator for BingoPlayer<'a> {
    // Yields the winning call and all of the boards that won with that call
    type Item = (u8, Vec<BingoBoard>);

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(call) = self.game.calls.pop_front() {
            let boards = &mut self.game.boards;
            let mut winning_boards: Vec<BingoBoard> = vec![];
            for BoardState {
                won: board_has_won,
                board,
            } in boards.iter_mut()
            {
                // If a board has one, do not consider it as an item to check.
                // This prevents winning boards from attempting to win more than once.
                if *board_has_won {
                    continue;
                }

                board.mark_n(call);
                if board.is_winner() {
                    // We should mark all boards, and not return immediately, so that an early board winning
                    // does not ruin the winners for everyone else
                    //
                    // Also, more than one board can win in the same turn (I've seen it happen!)
                    winning_boards.push(board.clone());

                    // We could probably remove the board from the boards vec, but for debugging, this changes the
                    // indexes, which makes it difficult to follow the continuity of boards
                    *board_has_won = true;
                }
            }

            if !winning_boards.is_empty() {
                return Some((call, winning_boards));
            }
        }

        None
    }
}

fn parse_bingo_number(input: &str) -> IResult<&str, u8> {
    map_res(take_while1(|c: char| c.is_ascii_digit()), str::parse)(input)
}

fn parse_bingo_calls(calls_line: &str) -> IResult<&str, Vec<u8>> {
    separated_list1(char(','), parse_bingo_number)(calls_line)
}

fn parse_bingo_board(input_chunk: &str) -> IResult<&str, BingoBoard> {
    let (remaining, raw_board) = separated_list1(
        char('\n'),
        separated_list1(char(' '), preceded(opt(char(' ')), parse_bingo_number)),
    )(input_chunk)?;

    // If we didn't get the correct board back from reading, this board is not parsable.
    if raw_board.len() != BOARD_SIZE || raw_board[0].len() != BOARD_SIZE {
        return fail(input_chunk);
    }

    let mut board = [[BingoTile::Unmarked(0_u8); BOARD_SIZE]; BOARD_SIZE];
    for i in 0..board.len() {
        let board_row = &mut board[i];
        let raw_board_row = &raw_board[i];
        assert_eq!(
            board_row.len(),
            raw_board_row.len(),
            "board size was allocated incorrectly for the board, or an invalid board was passed"
        );
        for j in 0..board_row.len() {
            board_row[j] = BingoTile::Unmarked(raw_board_row[j]);
        }
    }

    Ok((remaining, BingoBoard(board)))
}

// Calculate the score of a winning board, which is the same for both parts
fn calculate_score(winning_board: &BingoBoard, winning_call: u32) -> u32 {
    let unmarked_tiles_iter = winning_board
        .0
        .iter()
        .flatten()
        .filter(|tile| matches!(tile, BingoTile::Unmarked(_)))
        .map(|&tile| match tile {
            BingoTile::Unmarked(n) | BingoTile::Marked(n) => n,
        });

    unmarked_tiles_iter.map(u32::from).sum::<u32>() * winning_call
}

fn part1(input: &Input) -> u32 {
    let mut game = BingoGame::from(input.clone());
    let (winning_call, winning_board) = game
        .play()
        .next()
        .map(|(winning_call, winning_boards)| {
            (
                winning_call,
                winning_boards
                    .into_iter()
                    .next()
                    .expect("Got back empty vec of winners, which shouldn't ever happen"),
            )
        })
        .expect("Puzzle produced no winner for any bingo boards");

    calculate_score(&winning_board, winning_call.into())
}

fn part2(input: &Input) -> u32 {
    let mut game = BingoGame::from(input.clone());
    let (winning_call, winning_board) = game
        .play()
        .last()
        .map(|(winning_call, winning_boards)| {
            (
                winning_call,
                winning_boards
                    .into_iter()
                    .last()
                    .expect("Got back empty vec of winners, which shouldn't ever happen"),
            )
        })
        .expect("Puzzle produced no winner for any bingo boards");

    calculate_score(&winning_board, winning_call.into())
}

fn parse_input(input: &str) -> IResult<&str, Input> {
    let (_, ((calls, boards), _, _)) = tuple((
        separated_pair(
            parse_bingo_calls,
            tag("\n\n"),
            separated_list1(tag("\n\n"), parse_bingo_board),
        ),
        many0(tag("\n")),
        eof,
    ))(input)?;

    let input = Input { calls, boards };

    Ok(("", input))
}

fn main() {
    let input_file_name = env::args().nth(1).expect("No input filename specified");
    let input = fs::read_to_string(input_file_name).expect("Could not open input file");
    let (_, parsed_input) = parse_input(&input).expect("Failed to parse input");

    println!("Part 1: {}", part1(&parsed_input));
    println!("Part 2: {}", part2(&parsed_input));
}
