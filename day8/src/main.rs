#![warn(clippy::all, clippy::pedantic)]
use std::collections::{HashMap, HashSet};
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
use thiserror::Error;

const SEGMENT_CHARS: &[char] = &['a', 'b', 'c', 'd', 'e', 'f', 'g'];

#[derive(Debug, Clone)]
struct SignalInfo {
    signal_patterns: Vec<String>,
    output_values: Vec<String>,
}

struct SevenSegmentSignals {
    top: char,
    top_right: char,
    bottom_right: char,
    bottom: char,
    bottom_left: char,
    top_left: char,
    middle: char,
}

#[derive(Error, Debug)]
enum DecodeError {
    #[error("Invalid configuraton string given: {0}")]
    InavlidConfiguration(String),
}

impl SevenSegmentSignals {
    fn decode_str(&self, s: &str) -> Result<u8, DecodeError> {
        // Could these be members? yes. Are they? no.
        let zero = Self::make_segment_str(&[
            self.top,
            self.top_left,
            self.bottom_left,
            self.bottom,
            self.bottom_right,
            self.top_right,
        ]);

        let one = Self::make_segment_str(&[self.top_right, self.bottom_right]);

        let two = Self::make_segment_str(&[
            self.top,
            self.top_right,
            self.middle,
            self.bottom_left,
            self.bottom,
        ]);

        let three = Self::make_segment_str(&[
            self.top,
            self.top_right,
            self.middle,
            self.bottom_right,
            self.bottom,
        ]);

        let four = Self::make_segment_str(&[
            self.top_left,
            self.middle,
            self.top_right,
            self.bottom_right,
        ]);

        let five = Self::make_segment_str(&[
            self.top,
            self.top_left,
            self.middle,
            self.bottom_right,
            self.bottom,
        ]);

        let six = Self::make_segment_str(&[
            self.top,
            self.top_left,
            self.middle,
            self.bottom_left,
            self.bottom,
            self.bottom_right,
        ]);

        let seven = Self::make_segment_str(&[self.top, self.top_right, self.bottom_right]);

        let eight = Self::make_segment_str(&[
            self.top,
            self.top_left,
            self.bottom_left,
            self.middle,
            self.bottom,
            self.bottom_right,
            self.top_right,
        ]);

        let nine = Self::make_segment_str(&[
            self.top,
            self.top_left,
            self.middle,
            self.bottom,
            self.bottom_right,
            self.top_right,
        ]);

        let sorted_str = s.chars().sorted().collect::<String>();
        if sorted_str == one {
            Ok(1)
        } else if sorted_str == two {
            Ok(2)
        } else if sorted_str == three {
            Ok(3)
        } else if sorted_str == four {
            Ok(4)
        } else if sorted_str == five {
            Ok(5)
        } else if sorted_str == six {
            Ok(6)
        } else if sorted_str == seven {
            Ok(7)
        } else if sorted_str == eight {
            Ok(8)
        } else if sorted_str == nine {
            Ok(9)
        } else if sorted_str == zero {
            Ok(0)
        } else {
            Err(DecodeError::InavlidConfiguration(s.to_string()))
        }
    }

    fn make_segment_str(segments: &[char]) -> String {
        segments.iter().sorted().collect::<String>()
    }
}

fn is_segment_char(c: char) -> bool {
    SEGMENT_CHARS.contains(&c)
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
        signal_patterns: raw_signal_patterns.into_iter().map(str::to_owned).collect(),
        output_values: raw_output_values.into_iter().map(str::to_owned).collect(),
    };

    Ok(("", parsed_info))
}

/// Find the signal mappings that can be easily known by their number of segments
fn determine_simple_signal_mappings(signal_patterns: &[String]) -> HashMap<String, u8> {
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

fn transpose_signal_map(original: &HashMap<String, u8>) -> HashMap<u8, String> {
    original
        .iter()
        .map(|(s, &count)| (count, s.clone()))
        .collect()
}

fn part1(signal_infos: &[SignalInfo]) -> usize {
    signal_infos
        .iter()
        .map(|signal_info| {
            let signal_mapping = determine_simple_signal_mappings(&signal_info.signal_patterns);
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

fn make_char_set(s: &str) -> HashSet<char> {
    s.chars().collect()
}

// Infer all of the segements from a signal info
// This is long and makes many assertions. I don't have the patience to clean it up at this moment.
// If I was feeling less lazy, I'd turn these assertions into Results on the error type. Maybe later.
//
// This is just advent of code after all :)
#[allow(clippy::too_many_lines)]
fn infer_segments(signal_info: &SignalInfo) -> SevenSegmentSignals {
    let signal_mapping = determine_simple_signal_mappings(&signal_info.signal_patterns);
    let num_to_signal_map = transpose_signal_map(&signal_mapping);

    let one_signals = make_char_set(
        num_to_signal_map
            .get(&1)
            .expect("no mapping for 1 was determined"),
    );
    let seven_signals = make_char_set(
        num_to_signal_map
            .get(&7)
            .expect("no mapping for 7 was determined"),
    );
    let seven_one_difference = seven_signals
        .difference(&one_signals)
        .collect::<HashSet<_>>();
    assert_eq!(seven_one_difference.len(), 1);

    let top_segment = **seven_one_difference.iter().next().unwrap();
    println!("top => {}", top_segment);

    let four_signals = make_char_set(
        num_to_signal_map
            .get(&4)
            .expect("no mapping for 4 was determined"),
    );

    // This will have the two segments that don't have the right "stick" of the four.
    let four_one_difference = four_signals
        .difference(&one_signals)
        .copied()
        .collect::<HashSet<_>>();
    assert_eq!(four_one_difference.len(), 2);

    // There are three items that use six segments: 0 and 6, 9. Only zero matches only the lefthand "prong" of the four,
    // so the one with one intersection will disambiguate that one
    let six_element_char_sets = signal_info
        .signal_patterns
        .iter()
        .filter(|s| s.len() == 6)
        .map(|s| make_char_set(s))
        .collect::<Vec<_>>();
    assert_eq!(six_element_char_sets.len(), 3);

    let one_element_left_from_six_set = six_element_char_sets
        .iter()
        .map(|set| {
            four_one_difference
                .difference(set)
                .copied()
                .collect::<HashSet<char>>()
        })
        .filter(|set| set.len() == 1)
        .collect::<Vec<_>>();
    assert_eq!(one_element_left_from_six_set.len(), 1);
    let middle_segment_set = &one_element_left_from_six_set[0];
    assert_eq!(middle_segment_set.len(), 1);
    let middle_segment = *middle_segment_set.iter().next().unwrap();
    println!("middle => {}", middle_segment);

    // Now that we know the middle, the only element left in the original "four one difference" set will be the top left
    let top_left_segment_set = four_one_difference
        .iter()
        .filter(|&&c| c != middle_segment)
        .collect::<Vec<_>>();
    assert_eq!(top_left_segment_set.len(), 1);
    let top_left_segment = *top_left_segment_set[0];
    println!("top left => {}", top_left_segment);

    // Finding the top right segment is pretty easy. If we consider the segments of the "one", there is only one
    // six-element segment which these sets is not a super-set of the "one": the top right.
    let five_signals_set = six_element_char_sets
        .iter()
        .filter(|set| !set.is_superset(&one_signals))
        .collect::<Vec<_>>();
    assert_eq!(five_signals_set.len(), 1);

    let five_signals = five_signals_set[0];
    let top_right_difference = one_signals.difference(five_signals).collect::<HashSet<_>>();
    assert_eq!(top_right_difference.len(), 1);
    let top_right_segment = **top_right_difference.iter().next().unwrap();
    println!("top right => {}", top_right_segment);

    // And of course, knowing the top right, we know the bottom right, given there's only one other element in the one.
    let bottom_right_set = one_signals
        .iter()
        .filter(|&&c| c != top_right_segment)
        .collect::<HashSet<_>>();
    let bottom_right_segment = **bottom_right_set.iter().next().unwrap();
    println!("bottom right => {}", bottom_right_segment);

    // Now that we know the bottom and top left, of the six signal elements, we can uniquely identify the nine.
    // Of our possible input signals, the only one it _doesn't_ have must be the bottom left.
    let nine_char_set_set = six_element_char_sets
        .iter()
        .filter(|set| set.contains(&top_right_segment) && set.contains(&middle_segment))
        .collect::<Vec<_>>();
    assert_eq!(nine_char_set_set.len(), 1);

    let nine_char_set = nine_char_set_set[0];
    let segment_chars_set = SEGMENT_CHARS.iter().copied().collect::<HashSet<_>>();

    let bottom_left_set = segment_chars_set
        .difference(nine_char_set)
        .collect::<HashSet<_>>();
    assert_eq!(bottom_left_set.len(), 1);
    let bottom_left_segment = **bottom_left_set.iter().next().unwrap();
    println!("bottom_left => {}", bottom_left_segment);

    // aaaand all that's left is the bottom
    let all_but_bottom = vec![
        top_segment,
        top_right_segment,
        bottom_right_segment,
        bottom_left_segment,
        top_left_segment,
        middle_segment,
    ]
    .into_iter()
    .collect::<HashSet<_>>();

    let bottom_set = segment_chars_set
        .difference(&all_but_bottom)
        .collect::<HashSet<_>>();
    assert_eq!(bottom_set.len(), 1);

    let bottom_segment = **bottom_set.iter().next().unwrap();

    println!("bottom => {}", bottom_segment);

    SevenSegmentSignals {
        top: top_segment,
        top_right: top_right_segment,
        bottom_right: bottom_right_segment,
        bottom: bottom_segment,
        bottom_left: bottom_left_segment,
        top_left: top_left_segment,
        middle: middle_segment,
    }
}

fn part2(signal_infos: &[SignalInfo]) -> u32 {
    signal_infos
        .iter()
        .enumerate()
        .map(|(i, signal_info)| {
            println!("Item {}", i + 1);
            let segments = infer_segments(signal_info);
            let res = signal_info
                .output_values
                .iter()
                .map(|output| {
                    segments
                        .decode_str(output)
                        .unwrap_or_else(|err| panic!("Failed to decode {}: {:?}", output, err))
                        .into()
                })
                .fold(0_u32, |total, n: u32| (total * 10) + n);

            println!("{}\n", res);
            res
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

    println!("Part 1: {}", part1(&input_lines));
    println!("Part 2: {}", part2(&input_lines));
}
