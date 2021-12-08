use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader};

use nom::{
    bytes::complete::{tag, take_while1},
    character::complete::char,
    combinator::eof,
    multi::separated_list1,
    sequence::{separated_pair, terminated},
    IResult,
};

use itertools::Itertools;

#[derive(Debug)]
struct SignalInfo {
    signal_patterns: Vec<String>,
    output_values: Vec<String>,
}

fn is_segment_char(c: char) -> bool {
    ('a'..='g').contains(&c)
}

fn parse_signal_block(chunk: &str) -> IResult<&str, &str> {
    take_while1(is_segment_char)(chunk)
}

fn parse_signal_list(chunk: &str) -> IResult<&str, Vec<&str>> {
    separated_list1(char(' '), parse_signal_block)(chunk)
}

fn parse_line(line: &str) -> IResult<&str, SignalInfo> {
    let (_, (raw_signal_patterns, raw_output_values)) = terminated(
        separated_pair(parse_signal_list, tag(" | "), parse_signal_list),
        eof,
    )(line)?;

    let parsed_info = SignalInfo {
        signal_patterns: raw_signal_patterns
            .into_iter()
            .map(|s| s.to_owned())
            .collect(),
        output_values: raw_output_values
            .into_iter()
            .map(|s| s.to_owned())
            .collect(),
    };

    Ok(("", parsed_info))
}

fn determine_signal_mapping(signal_patterns: &[String]) -> HashMap<String, u8> {
    signal_patterns
        .iter()
        .filter_map(|signal_pattern| {
            let sorted_pattern = signal_pattern.chars().sorted().collect();
            match signal_pattern.len() {
                2 => Some((sorted_pattern, 1)),
                3 => Some((sorted_pattern, 7)),
                4 => Some((sorted_pattern, 4)),
                7 => Some((sorted_pattern, 8)),
                _ => None,
            }
        })
        .collect()
}

fn part1(signal_infos: &[SignalInfo]) -> usize {
    signal_infos
        .iter()
        .map(|signal_info| {
            let signal_mapping = determine_signal_mapping(&signal_info.signal_patterns);
            signal_info
                .output_values
                .iter()
                .filter(|output_value| {
                    let sorted_output_value = output_value.chars().sorted().collect::<String>();
                    signal_mapping.contains_key(&sorted_output_value)
                })
                .count()
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
            let (_, coords) = parse_line(&s).expect("Failed to read line");
            coords
        })
        .collect::<Vec<_>>();

    // println!("{:?}", input_lines);
    println!("Part 1: {}", part1(&input_lines));
}
