#![warn(clippy::all, clippy::pedantic)]
use std::{collections::HashMap, env, fs, iter};

use nom::{
    bytes::complete::tag,
    character::complete::{char, digit1},
    combinator::{eof, map_res},
    multi::many0,
    sequence::{pair, preceded, separated_pair, terminated, tuple},
    IResult,
};

#[derive(Debug, Clone)]
struct DeterministicDie {
    next_value: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Player {
    Player1,
    Player2,
}

#[derive(Debug, Clone)]
struct PlayerState {
    score: u32,
    position: u32,
}

impl Player {
    fn get_other(self) -> Player {
        match self {
            Self::Player1 => Self::Player2,
            Self::Player2 => Self::Player1,
        }
    }
}

impl PlayerState {
    fn new(starting_pos: u32) -> Self {
        Self {
            score: 0,
            position: starting_pos,
        }
    }

    fn move_forward(&mut self, n: u32) {
        self.position = (self.position + n - 1) % 10 + 1;
        self.score += self.position;
    }
}

impl Default for DeterministicDie {
    fn default() -> Self {
        Self { next_value: 1 }
    }
}

impl DeterministicDie {
    fn new() -> Self {
        Self::default()
    }

    fn roll(&mut self) -> u32 {
        let val = self.next_value;
        self.next_value = self.next_value % 100 + 1;

        val
    }
}

fn parse_player_starting_position(chunk: &str) -> IResult<&str, u32> {
    preceded(
        tuple((tag("Player "), digit1, tag(" starting position: "))),
        map_res(digit1, str::parse),
    )(chunk)
}

fn parse_input(input: &str) -> IResult<&str, (u32, u32)> {
    terminated(
        separated_pair(
            parse_player_starting_position,
            char('\n'),
            parse_player_starting_position,
        ),
        pair(many0(char('\n')), eof),
    )(input)
}

fn part1(player1_start_pos: u32, player2_start_pos: u32) -> u32 {
    let mut die = DeterministicDie::new();
    let mut num_rolls = 0;
    let mut current_player = Player::Player1;

    let mut player_states = [
        (Player::Player1, PlayerState::new(player1_start_pos)),
        (Player::Player2, PlayerState::new(player2_start_pos)),
    ]
    .into_iter()
    .collect::<HashMap<_, _>>();

    while (player_states
        .values()
        .map(|state| state.score)
        .max()
        .unwrap_or(0))
        < 1000
    {
        let roll_value = iter::once(0).cycle().take(3).map(|_| die.roll()).sum();
        num_rolls += 3;

        let current_player_state = player_states
            .get_mut(&current_player)
            .expect("current player does not have a state associated with it");
        current_player_state.move_forward(roll_value);

        current_player = current_player.get_other();
    }

    let losing_score = player_states
        .values()
        .map(|state| state.score)
        .min()
        .unwrap_or(0);

    losing_score * num_rolls
}

fn main() {
    let input_file_name = env::args().nth(1).expect("No input filename specified");
    let input = fs::read_to_string(input_file_name).expect("Failed to read input file");
    let (_, (player1_start_pos, player2_start_pos)) =
        parse_input(&input).expect("Failed to parse input");

    println!("Part 1: {}", part1(player1_start_pos, player2_start_pos));
}
