#![warn(clippy::all, clippy::pedantic)]

use nom::{
    branch::alt,
    bytes::complete::{tag, take_while1},
    character::is_digit,
    combinator::{eof, map_res},
    sequence::{separated_pair, terminated},
    IResult,
};
use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader};

enum Part {
    Part1,
    Part2,
}

/// The direction of a sub command
enum Direction {
    Forward(i32),
    Down(i32),
    Up(i32),
}

/// The location of the sub
#[derive(Default)]
struct Location {
    position: i32,
    depth: i32,
    aim: i32,
}

impl Location {
    /// Produce a new location that is moved based  based on simple directionality:
    /// forward/up/down map to positions and depths
    fn perform_directional_move(&self, direction: &Direction) -> Location {
        match direction {
            Direction::Up(n) => Location {
                position: self.position,
                depth: self.depth - n,
                aim: self.aim,
            },
            Direction::Down(n) => Location {
                position: self.position,
                depth: self.depth + n,
                aim: self.aim,
            },
            Direction::Forward(n) => Location {
                position: self.position + n,
                depth: self.depth,
                aim: self.aim,
            },
        }
    }

    /// Produce a new location that is moved based on the current aim; depth is controlled by a multiple of aim
    fn perform_aim_based_move(&self, direction: &Direction) -> Location {
        match direction {
            Direction::Up(n) => Location {
                position: self.position,
                depth: self.depth,
                aim: self.aim - n,
            },
            Direction::Down(n) => Location {
                position: self.position,
                depth: self.depth,
                aim: self.aim + n,
            },
            Direction::Forward(n) => Location {
                position: self.position + n,
                depth: self.depth + (self.aim * n),
                aim: self.aim,
            },
        }
    }
}

fn simulate(directions: &[Direction], part: &Part) -> i32 {
    let final_location =
        directions
            .iter()
            .fold(Location::default(), |memo, direction| match part {
                Part::Part1 => memo.perform_directional_move(direction),
                Part::Part2 => memo.perform_aim_based_move(direction),
            });

    final_location.position * final_location.depth
}

fn parse_line(line: &str) -> IResult<&str, Direction> {
    // using a parser combinator to split a string
    // https://i.imgur.com/B7bfMdE.jpg
    // (I really just want to get practice with nom because it's fun)
    let parse_direction = alt((tag("forward"), tag("down"), tag("up")));
    let (_, (raw_direction, magnitude)) = terminated(
        separated_pair(
            parse_direction,
            tag(" "),
            map_res(take_while1(|c| is_digit(c as u8)), str::parse),
        ),
        eof,
    )(line)?;

    let direction = match raw_direction {
        "forward" => Direction::Forward,
        "down" => Direction::Down,
        "up" => Direction::Up,
        _ => panic!("invalid direction returned by parser; this can't happen"),
    }(magnitude);

    Ok(("", direction))
}

fn main() {
    let input_file_name = env::args().nth(1).expect("No input filename specified");
    let input_file = File::open(input_file_name).expect("Could not open input file");
    let directions = BufReader::new(input_file)
        .lines()
        .map(|res| res.expect("Failed to read line"))
        .map(|line| {
            let (remaining, direction) = parse_line(&line)
                .unwrap_or_else(|err| panic!("Failed to parse line '{}': {}", line, err));

            // Should never happen if the parse function succeeded
            assert!(
                remaining.is_empty(),
                "Input remained after parsing: {}",
                remaining
            );

            direction
        })
        .collect::<Vec<_>>();

    println!("Part 1: {}", simulate(&directions, &Part::Part1));
    println!("Part 2: {}", simulate(&directions, &Part::Part2));
}
