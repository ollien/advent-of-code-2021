#![warn(clippy::all, clippy::pedantic)]
use std::collections::HashMap;
use std::env;
use std::fs;

use nom::{
    bytes::complete::{tag, take_while1},
    combinator::eof,
    multi::{many0, separated_list1};,
    sequence::{pair, separated_pair, terminated},
    character::complete::char,
    IResult
};

fn parse_polymer(chunk: &str) -> IResult<&str, &str> {
    take_while1(|c: char| c.is_ascii_uppercase())(chunk)
}

fn parse_insertion_rule(chunk: &str) -> IResult<&str, (&str, &str)> {
    separated_pair(parse_polymer, tag(" -> "), parse_polymer)(chunk)
}

fn parse_input(input: &str) -> IResult<&str, (&str, Vec<(&str, &str)>)> {
    terminated(
        separated_pair(
            parse_polymer,
            tag("\n\n"),
            separated_list1(char('\n'), parse_insertion_rule),
        ),
        pair(many0(char('\n')), eof),
    )(input)
}

/// Get all of the pairs of chars in the template string, overlapping.
/// This is equivalent (though it does allocate, but this is only used
/// on a small string) to using slice::window(2), but this is not
/// available for strings :(
fn get_all_pairs(template: &str) -> Vec<&str> {
    let mut pairs = Vec::<&str>::new();
    for i in 0..template.len() - 1 {
        let window = &template[i..=i + 1];
        pairs.push(window);
    }

    pairs
}

fn run(template: &str, mappings: &HashMap<&str, char>, num_iterations: usize) -> u64 {
    // Populate the counts of pairs with 0 for any pairs in a rule, and 1 for every pair in our template
    let mut pair_counts = mappings
        .keys()
        .copied()
        .map(|pair| (pair, 0))
        .chain(get_all_pairs(template).into_iter().map(|pair| (pair, 1)))
        .collect::<HashMap<&str, u64>>();

    let mut element_counts = template
        .chars()
        .map(|c| (c, 0))
        .collect::<HashMap<char, u64>>();

    for _ in 0..num_iterations {
        let non_zero_count_pairs = pair_counts.iter().filter(|(_, &count)| count > 0);
        let mut next_pair_counts = pair_counts.clone();
        for (pair, &count) in non_zero_count_pairs {
            let new_char = mappings
                .get(pair)
                .unwrap_or_else(|| panic!("could not find mapping for rule {}", pair));

            *element_counts.entry(*new_char).or_insert(0) += count;
            // One of each of these pairs will no longer exist
            *next_pair_counts.entry(pair).or_insert(0) -= count;

            // ...but there will be newly formed pairs to add in, two per pair we removed
            for (i, c) in pair.chars().enumerate() {
                // i will always be 0 or 1 here (since it's a pair of numbers, which is true by the parsing logic)
                // passed into here
                let rule_output = if i == 0 {
                    format!("{}{}", c, new_char)
                } else {
                    format!("{}{}", new_char, c)
                };

                let current_pair_count = next_pair_counts
                    .get_mut(rule_output.as_str())
                    .unwrap_or_else(|| {
                        panic!(
                            "somehow produced pair {} which was being tracked (and thus not in the rules map)",
                            pair
                        )
                    });

                *current_pair_count += count;
            }
        }

        pair_counts = next_pair_counts;
    }

    element_counts.values().max().unwrap() - element_counts.values().min().unwrap()
}

fn main() {
    let input_file_name = env::args().nth(1).expect("No input filename specified");
    let input = fs::read_to_string(input_file_name).expect("Failed to read input file");
    let (_, (template, raw_mappings)) = parse_input(&input).expect("Failed to parse input");
    let mappings = raw_mappings
        .into_iter()
        .map(|(rule, mapping)| {
            assert_eq!(rule.len(), 2, "Rule inputs should have length of 2");
            assert_eq!(mapping.len(), 1, "Rule outputs should have length of 1");

            (rule, mapping.chars().next().unwrap())
        })
        .collect::<HashMap<_, _>>();

    println!("Part 1: {}", run(template, &mappings, 10));
    println!("Part 2: {}", run(template, &mappings, 40));
}
