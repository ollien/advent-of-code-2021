use std::collections::HashMap;
use std::{env, fs};

use nom::bytes::complete::{tag, take_while1};
use nom::combinator::eof;
use nom::multi::{many0, separated_list1};
use nom::sequence::{pair, separated_pair, terminated};
use nom::{character::complete::char, IResult};

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

fn get_all_pairs(template: &str) -> Vec<&str> {
    let mut pairs = Vec::<&str>::new();
    for i in 0..template.len() - 1 {
        let window = &template[i..=i + 1];
        pairs.push(window);
    }

    pairs
}

fn run(template: &str, mappings: &HashMap<&str, char>, num_iterations: usize) -> u64 {
    let mut pair_counts = mappings
        .keys()
        .copied()
        .map(|pair| (pair, 0))
        .chain(get_all_pairs(template).into_iter().map(|pair| (pair, 1)))
        .collect::<HashMap<&str, u64>>();

    let mut element_counts = HashMap::<char, u64>::new();
    for c in template.chars() {
        *element_counts.entry(c).or_insert(0) += 1
    }

    for _ in 0..num_iterations {
        let pair_iter = pair_counts.iter().filter(|(_, &count)| count > 0);
        let mut rule_application_counts = HashMap::<String, i64>::new();
        for (pair, &count) in pair_iter {
            let new_char = mappings
                .get(pair)
                .unwrap_or_else(|| panic!("could not find mapping for rule {}", pair));

            *element_counts.entry(*new_char).or_insert(0) += count;
            *rule_application_counts.entry(pair.to_string()).or_insert(0) -=
                i64::try_from(count).unwrap();

            for (i, c) in pair.chars().enumerate() {
                let rule_output = if i == 0 {
                    format!("{}{}", c, new_char)
                } else {
                    format!("{}{}", new_char, c)
                };

                *rule_application_counts.entry(rule_output).or_insert(0) +=
                    i64::try_from(count).unwrap();
            }
        }

        for (pair, count) in rule_application_counts {
            let current_pair_count = pair_counts.get_mut(pair.as_str()).unwrap_or_else(|| {
                panic!(
                    "somehow produced pair {} which is not in the rules map",
                    pair
                )
            });

            if count > 0 {
                *current_pair_count = current_pair_count.saturating_add(count as u64)
            } else {
                *current_pair_count = current_pair_count.saturating_sub(-count as u64)
            }
        }
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
            if mapping.len() != 1 {
                // This could be done in the parser but nom was giving me difficulty and this was easier
                panic!("Rules should only have a length of one");
            }

            (rule, mapping.chars().next().unwrap())
        })
        .collect::<HashMap<_, _>>();

    println!("Part 1: {}", run(template, &mappings, 10));
    println!("Part 2: {}", run(template, &mappings, 40));
}
