#![warn(clippy::all, clippy::pedantic)]
use std::cmp;
use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader};

use nom::{
    bytes::complete::{tag, take_while1},
    character::complete::char,
    combinator::eof,
    combinator::map_res,
    sequence::{separated_pair, terminated},
    IResult,
};
use thiserror::Error;

#[derive(Debug, Error)]
enum Error {
    #[error("The coordinates {0:?} and {0:?} are not in line with the given strategy {:0?}")]
    InvalidDirection(Coordinate, Coordinate, Strategy),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Coordinate(u32, u32);

#[derive(Debug, Clone, Copy)]
enum Strategy {
    OrthogonalOnly,
    OrthogonalAnd45Degrees,
}

impl Strategy {
    fn points_follow_strategy(self, a: Coordinate, b: Coordinate) -> bool {
        let x_range = order_pair(a.0, b.0);
        let y_range = order_pair(a.1, b.1);

        let rise = y_range.1 - y_range.0;
        let run = x_range.1 - x_range.0;

        match self {
            Self::OrthogonalOnly => run == 0 || rise == 0,
            Self::OrthogonalAnd45Degrees => run == 0 || rise == 0 || run == rise,
        }
    }
}

impl Coordinate {
    /// Create an iterator that will move between this point and a given ending point.
    ///
    /// # Errors
    /// If the direction between this point and the other are not in a direction
    /// that matches the given strategy, [`Error::InvalidDirection`] is returned
    fn iter_between(
        self,
        other: Coordinate,
        strategy: Strategy,
    ) -> Result<impl Iterator<Item = Coordinate>, Error> {
        if !strategy.points_follow_strategy(self, other) {
            return Err(Error::InvalidDirection(self, other, strategy));
        }

        let x_range = order_pair(self.0, other.0);
        let y_range = order_pair(self.1, other.1);
        // Calculate the distance between points using the "max norm"
        // this is like the manhattan distance, but diagonals are 1
        // (I'm effectively using it as an integral euclidian distance given the context)
        let travel_distance = cmp::max(x_range.1 - x_range.0, y_range.1 - y_range.0);

        let iter = (0..=travel_distance).map(move |n| {
            let end_x = add_to_component_directionally(self.0, other.0, n);
            let end_y = add_to_component_directionally(self.1, other.1, n);

            Coordinate(end_x, end_y)
        });
        Ok(iter)
    }
}

fn order_pair<T: PartialOrd>(a: T, b: T) -> (T, T) {
    if a > b {
        (b, a)
    } else {
        (a, b)
    }
}

/// Add some number, n, to the start component of a vector, in the direction of its ending point.
fn add_to_component_directionally(start: u32, end: u32, n: u32) -> u32 {
    match start.cmp(&end) {
        cmp::Ordering::Equal => start,
        cmp::Ordering::Greater => start - n,
        cmp::Ordering::Less => start + n,
    }
}

/// Build a map of the number of intersections between lines bounded (inclusively) by each element
/// the `coordinate_pairs` slice. The retruend map will indicate the number of (non-zero) interactions
/// at each point
fn build_intersection_count_map(
    coordinate_pairs: &[(Coordinate, Coordinate)],
    strategy: Strategy,
) -> Result<HashMap<Coordinate, u32>, Error> {
    let mut counts = HashMap::new();
    for &pair in coordinate_pairs {
        let (start, end) = pair;

        let iter_res = start.iter_between(end, strategy);
        if let Err(Error::InvalidDirection(_, _, _)) = iter_res {
            continue;
        }

        for coord in iter_res? {
            let count = counts.get(&coord).unwrap_or(&0);
            let updated_count = count + 1;
            counts.insert(coord, updated_count);
        }
    }

    Ok(counts)
}

fn part1(coordinate_pairs: &[(Coordinate, Coordinate)]) -> usize {
    let map = build_intersection_count_map(coordinate_pairs, Strategy::OrthogonalOnly)
        .expect("Failed to build coordinate map from input");

    map.values().filter(|&&n| n >= 2).count()
}

fn part2(coordinate_pairs: &[(Coordinate, Coordinate)]) -> usize {
    let map = build_intersection_count_map(coordinate_pairs, Strategy::OrthogonalAnd45Degrees)
        .expect("Failed to build coordinate map from input");

    map.values().filter(|&&n| n >= 2).count()
}

fn parse_number(s: &str) -> IResult<&str, u32> {
    map_res(take_while1(|c: char| c.is_ascii_digit()), str::parse)(s)
}

fn parse_coordinate(s: &str) -> IResult<&str, Coordinate> {
    let (remaining, parsed_numbers) = separated_pair(parse_number, char(','), parse_number)(s)?;

    Ok((remaining, Coordinate(parsed_numbers.0, parsed_numbers.1)))
}

fn parse_line(line: &str) -> IResult<&str, (Coordinate, Coordinate)> {
    terminated(
        separated_pair(parse_coordinate, tag(" -> "), parse_coordinate),
        eof,
    )(line)
}

fn main() {
    let input_file_name = env::args().nth(1).expect("No input filename specified");
    let input_file = File::open(input_file_name).expect("Could not open input file");

    let input_coordinates = BufReader::new(input_file)
        .lines()
        .map(|res| res.expect("Failed to read line"))
        .map(|s| {
            let (_, coords) = parse_line(&s).expect("Failed to read line");
            coords
        })
        .collect::<Vec<_>>();

    println!("Part 1: {}", part1(&input_coordinates));
    println!("Part 2: {}", part2(&input_coordinates));
}
