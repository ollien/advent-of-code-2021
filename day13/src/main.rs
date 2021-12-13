#![warn(clippy::all, clippy::pedantic)]
use std::env;
use std::fs;

use itertools::Itertools;
use itertools::MinMaxResult;
use nom::{
    branch::alt,
    bytes::complete::{tag, take_while1},
    character::complete::char,
    combinator::{eof, map_res},
    multi::{many0, separated_list1},
    sequence::{preceded, separated_pair, terminated},
    IResult,
};
use std::collections::HashSet;

#[derive(Debug, Copy, Clone)]
enum Fold {
    AlongX(u32),
    AlongY(u32),
}

impl Fold {
    /// Fold a point along the axis given by this Fold, and return it
    fn fold_point(self, (x, y): (u32, u32)) -> (u32, u32) {
        match self {
            Fold::AlongX(fold_x) => {
                // points to the _LEFT_ of the line should stay in place
                if x < fold_x {
                    (x, y)
                } else {
                    let delta = x - fold_x;
                    (fold_x - delta, y)
                }
            }
            Fold::AlongY(fold_y) => {
                // points _ABOVE_ the line should stay in place
                if y < fold_y {
                    (x, y)
                } else {
                    let delta = y - fold_y;
                    (x, fold_y - delta)
                }
            }
        }
    }
}

fn parse_number(chunk: &str) -> IResult<&str, u32> {
    map_res(take_while1(|c: char| c.is_ascii_digit()), str::parse)(chunk)
}

fn parse_point(chunk: &str) -> IResult<&str, (u32, u32)> {
    separated_pair(parse_number, char(','), parse_number)(chunk)
}

fn parse_fold_instruction(chunk: &str) -> IResult<&str, Fold> {
    let (remaining, (axis, value)) = preceded(
        tag("fold along "),
        separated_pair(alt((char('x'), char('y'))), char('='), parse_number),
    )(chunk)?;

    let res = match axis {
        'x' => Fold::AlongX(value),
        'y' => Fold::AlongY(value),
        // This should never happen by the parsing rules
        _ => panic!("invalid char extracted from parser"),
    };

    Ok((remaining, res))
}

fn parse_input(input: &str) -> IResult<&str, (Vec<(u32, u32)>, Vec<Fold>)> {
    terminated(
        terminated(
            separated_pair(
                separated_list1(char('\n'), parse_point),
                tag("\n\n"),
                separated_list1(char('\n'), parse_fold_instruction),
            ),
            many0(char('\n')),
        ),
        eof,
    )(input)
}

fn display_points(points: &HashSet<(u32, u32)>) {
    let minmax_x = points.iter().clone().map(|point| point.0).minmax();
    let minmax_y = points.iter().map(|point| point.1).minmax();
    match (minmax_x, minmax_y) {
        (MinMaxResult::NoElements, _) | (_, MinMaxResult::NoElements) => println!("No points to display"),
        (MinMaxResult::OneElement(_), MinMaxResult::OneElement(_)) => println!("#"),
        (MinMaxResult::MinMax(min_x, max_x), MinMaxResult::MinMax(min_y, max_y)) => {
            for y in min_y..=max_y {
                for x in min_x..=max_x {
                    if points.contains(&(x, y)) {
                        print!("#");
                    } else {
                        print!(" ");
                    }
                }

                println!();
            }
        }
        _ => panic!("this can't ever happen; a differing number of elements between the number of x's and y's")
    }
}

fn part1(points: &[(u32, u32)], folds: &[Fold]) -> usize {
    let mut point_set = points.iter().copied().collect::<HashSet<_>>();

    let first_fold = folds[0];
    point_set = point_set
        .into_iter()
        .map(|point| first_fold.fold_point(point))
        .collect();

    point_set.len()
}

fn part2(points: &[(u32, u32)], folds: &[Fold]) {
    let mut point_set = points.iter().copied().collect::<HashSet<_>>();

    for fold in folds {
        point_set = point_set
            .into_iter()
            .map(|point| fold.fold_point(point))
            .collect();
    }

    display_points(&point_set);
}

fn main() {
    let input_file_name = env::args().nth(1).expect("No input filename specified");
    let input = fs::read_to_string(input_file_name).expect("Could not open input file");
    let (_, (points, folds)) = parse_input(&input).expect("Failed to parse input");

    println!("Part 1: {}", part1(&points, &folds));
    println!("Part 2: (use your eyes)");
    part2(&points, &folds);
}
