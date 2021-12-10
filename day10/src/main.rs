use itertools::Itertools;
use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader};
use thiserror::Error;

#[derive(Error, Debug)]
enum Error {
    #[error("Received corrupted input to autocomplete function")]
    CorruptedInput(String),
}

// Find the char that is corrupted in this input, if any (a non corrupted line returns None)
fn find_corrupted_char(line: &str) -> Option<char> {
    let mut expected_stack = vec![];
    for c in line.chars() {
        match c {
            '(' => expected_stack.push(')'),
            '[' => expected_stack.push(']'),
            '<' => expected_stack.push('>'),
            '{' => expected_stack.push('}'),
            c => {
                let top_of_stack = expected_stack.pop();
                if top_of_stack.is_none() {
                    return None;
                } else if top_of_stack.unwrap() != c {
                    return Some(c);
                }
            }
        }
    }

    None
}

// Find the auto-completion on a non-corrupted line
fn find_completion(line: &str) -> Result<String, Error> {
    let mut expected_stack = vec![];
    for c in line.chars() {
        match c {
            '(' => expected_stack.push(')'),
            '[' => expected_stack.push(']'),
            '<' => expected_stack.push('>'),
            '{' => expected_stack.push('}'),
            c => {
                if let Some(top_of_stack) = expected_stack.pop() {
                    if top_of_stack != c {
                        return Err(Error::CorruptedInput(line.to_string()));
                    }
                }
            }
        }
    }

    Ok(expected_stack.into_iter().rev().join(""))
}

fn part1(input_lines: &[String]) -> u32 {
    input_lines
        .iter()
        .map(|s| find_corrupted_char(s))
        .flatten()
        .map(|failed_char| match failed_char {
            ')' => 3,
            ']' => 57,
            '}' => 1197,
            '>' => 25137,
            _ => panic!("unexpected char from find_corrupted_char, {}", failed_char),
        })
        .sum()
}

fn part2(input_lines: &[String]) -> u64 {
    let scores = input_lines
        .iter()
        .filter(|s| find_corrupted_char(s).is_none())
        .map(|s| find_completion(s))
        .map(|completion| {
            if let Err(err) = completion {
                panic!("Failed to find autocomplete: {}", err);
            }

            completion
                .unwrap()
                .chars()
                .map(|completed_char| match completed_char {
                    ')' => 1,
                    ']' => 2,
                    '}' => 3,
                    '>' => 4,
                    _ => panic!("unexpected char in completion, {}", completed_char),
                })
                .fold(0, |total, char_score| total * 5 + char_score)
        })
        .sorted()
        .collect::<Vec<_>>();

    scores[scores.len() / 2]
}

fn main() {
    let input_file_name = env::args().nth(1).expect("No input filename specified");
    let input_file = File::open(input_file_name).expect("Could not open input file");
    let input_lines = BufReader::new(input_file)
        .lines()
        .map(|res| res.expect("Failed to read line"))
        .collect::<Vec<_>>();

    println!("Part 1: {}", part1(&input_lines));
    println!("Part 2: {}", part2(&input_lines));
}
