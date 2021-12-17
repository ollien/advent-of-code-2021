#![warn(clippy::all, clippy::pedantic)]
use std::cmp::Ordering;
use std::env;
use std::fs;
use std::str::FromStr;

use nom::bytes::complete::tag;
use nom::character::complete::{char, digit1};
use nom::combinator::opt;
use nom::combinator::{eof, map_res};
use nom::sequence::{pair, preceded, separated_pair, terminated};
use nom::IResult;

#[derive(Debug, Copy, Clone)]
enum SimulationMode {
    IgnoreX,
    Full,
}

fn parse_number(chunk: &str) -> IResult<&str, i64> {
    map_res(
        pair(opt(char('-')), digit1),
        |(negative, num): (Option<char>, &str)| -> Result<i64, <i64 as FromStr>::Err> {
            let parsed_num = num.parse::<i64>()?;
            Ok(match negative {
                Some(_) => -parsed_num,
                None => parsed_num,
            })
        },
    )(chunk)
}

fn parse_range(chunk: &str) -> IResult<&str, (i64, i64)> {
    separated_pair(parse_number, tag(".."), parse_number)(chunk)
}

fn parse_input(input: &str) -> IResult<&str, ((i64, i64), (i64, i64))> {
    terminated(
        preceded(
            tag("target area: "),
            separated_pair(
                preceded(tag("x="), parse_range),
                tag(", "),
                preceded(tag("y="), parse_range),
            ),
        ),
        eof,
    )(input)
}

/// Run the simulation, returning the max position of each component.
/// If None, this simulation did not run to completion because it overshot the bounding box.
fn simulate_to_max_position(
    x_range: (i64, i64),
    y_range: (i64, i64),
    initial_velocity: (i64, i64),
    mode: SimulationMode,
) -> Option<(i64, i64)> {
    let mut position = (0, 0);
    let mut max_position = (0, 0);
    let mut velocity = initial_velocity;
    while !(position.1 >= y_range.0
        && position.1 <= y_range.1
        && (matches!(mode, SimulationMode::IgnoreX)
            || (position.0 >= x_range.0 && position.0 <= x_range.1)))
    {
        // Check for an overshoot
        if position.1 < y_range.0 || position.0 > x_range.1 {
            return None;
        }

        position.0 += velocity.0;
        position.1 += velocity.1;

        velocity.0 = match velocity.0.cmp(&0) {
            Ordering::Greater => std::cmp::max(velocity.0 - 1, 0),
            Ordering::Less => std::cmp::max(velocity.0 + 1, 0),
            Ordering::Equal => 0,
        };
        velocity.1 -= 1;

        max_position.0 = std::cmp::max(position.0, max_position.0);
        max_position.1 = std::cmp::max(position.1, max_position.1);
    }

    Some(max_position)
}

fn part1(x_range: (i64, i64), y_range: (i64, i64)) -> i64 {
    // We know that at equal y positions in our arc, the y velocity will be opposite but equal.
    // Therefore, the bound on our velocity is nothing more than the distance between our starting point and
    // the lowest point, or the total range of our box, whichever is bigger.
    let y_vel_bound = std::cmp::max((y_range.0 - y_range.1).abs() + 1, y_range.0.abs());

    (0..=y_vel_bound)
        .filter_map(|y_vel| {
            // We can just igonre the x component to solve part 1. Our y velocity is totally independent.
            simulate_to_max_position(x_range, y_range, (0, y_vel), SimulationMode::IgnoreX)
        })
        .map(|(_, max_y)| max_y)
        .max()
        .expect("no solution found for part 1")
}

fn part2(x_range: (i64, i64), y_range: (i64, i64)) -> usize {
    // Same idea as part 1
    let y_vel_bound = std::cmp::max((y_range.0 - y_range.1).abs() + 1, y_range.0.abs());
    // The furthest we can throw x from our starting p oint is going to be the distance from our starting point,
    // or the size of the box, whichever is bigger. If it were larger than the starting point, we'd immediately
    // throw it past
    let x_vel_bound = std::cmp::max((x_range.0 - x_range.1).abs() + 1, x_range.1.abs());

    // Similar logic to part 1, but we search the entire negative space too, because throwing down is a possibility
    // (this didn't matter when searching for a maximum)
    (-y_vel_bound..=y_vel_bound)
        // For the given input technically the negative x check is redundant (since the bounding box
        // always has a positive x), but I did it anyway for funzies.
        .flat_map(|y| (-x_vel_bound..=x_vel_bound).map(move |x| (x, y)))
        .filter_map(|velocity| {
            simulate_to_max_position(x_range, y_range, velocity, SimulationMode::Full)
        })
        .count()
}

fn main() {
    let input_file_name = env::args().nth(1).expect("No input filename specified");
    let input = fs::read_to_string(input_file_name).expect("Failed to read input file");
    let (remaining, (x_range, y_range)) = parse_input(input.trim()).expect("Failed to parse input");
    assert!(remaining.is_empty(), "Expected EOF, found more input");

    println!("Part 1: {}", part1(x_range, y_range));
    println!("Part 2: {}", part2(x_range, y_range));
}
