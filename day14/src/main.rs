use std::collections::{HashMap, LinkedList};
use std::{env, fs};

use itertools::Itertools;
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

fn part1(template: &str, mappings: &HashMap<&str, &str>) -> usize {
    let mut polymer = template
        .chars()
        .enumerate()
        .map(|(i, _)| &template[i..i + 1])
        .collect::<LinkedList<&str>>();

    for step in 0..10 {
        println!("Step: {}", step);
        let rules_to_apply = polymer
            .iter()
            .enumerate()
            .tuple_windows()
            .filter_map(|((_, s1), (i, s2))| {
                mappings
                    .iter()
                    .find(|(key, _)| itertools::equal(s1.chars().chain(s2.chars()), key.chars()))
                    .map(|(_, insertion)| (i, insertion))
            })
            .collect::<Vec<_>>();

        for (insert_index, insertion) in rules_to_apply.into_iter().rev() {
            let mut end = polymer.split_off(insert_index);
            polymer.push_back(insertion);
            polymer.append(&mut end);
        }
    }

    let mut frequency_table = HashMap::<&str, usize>::new();
    for element in polymer {
        *frequency_table.entry(element).or_insert(0) += 1
    }

    let most_common = frequency_table
        .iter()
        .max_by(|(_, count1), (_, count2)| count1.cmp(count2))
        .unwrap();

    let least_common = frequency_table
        .iter()
        .min_by(|(_, count1), (_, count2)| count1.cmp(count2))
        .unwrap();

    most_common.1 - least_common.1
}

fn main() {
    let input_file_name = env::args().nth(1).expect("No input filename specified");
    let input = fs::read_to_string(input_file_name).expect("Failed to read input file");
    let (_, (template, raw_mappings)) = parse_input(&input).expect("Failed to parse inputt");
    let mappings = raw_mappings.into_iter().collect::<HashMap<_, _>>();

    println!("Part 1: {}", part1(template, &mappings));
}
