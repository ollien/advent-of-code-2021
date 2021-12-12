#![warn(clippy::all, clippy::pedantic)]
use nom::{
    branch::alt,
    bytes::complete::take_while1,
    character::complete::char,
    combinator::eof,
    sequence::{separated_pair, terminated},
    IResult,
};
use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader};

const START_CAVE_NAME: &str = "start";
const END_CAVE_NAME: &str = "end";

#[derive(Clone, Copy)]
enum Part {
    Part1,
    Part2,
}

#[derive(Debug, Hash, Clone, PartialEq, Eq)]
struct Cave {
    name: String,
}

impl Cave {
    /// Check if a have is a "big" cave, which may be revisited as many times as we like
    fn is_big(&self) -> bool {
        // We know from the parsing that it will be either all capital or all lowercase,
        // so any() will suffice
        self.name.chars().any(|c| c.is_ascii_uppercase())
    }
}

struct Puzzle<'a> {
    part: Part,
    adjacencies: &'a HashMap<&'a Cave, Vec<&'a Cave>>,
}

impl<'a> Puzzle<'a> {
    /// Find the number of paths through the cave
    fn find_num_paths(&self) -> usize {
        let start_cave = Cave {
            name: START_CAVE_NAME.to_string(),
        };
        let start_adjacencies = self
            .adjacencies
            .get(&start_cave)
            .expect("Input did not contain start cave");

        let mut to_visit = start_adjacencies
            .iter()
            .map(|&cave| (cave, vec![&start_cave, cave]))
            .collect::<Vec<_>>();
        let mut paths = vec![];
        while let Some((visiting, path)) = to_visit.pop() {
            if visiting.name == END_CAVE_NAME {
                paths.push(path);
                continue;
            }

            let visiting_adjancencies = self.adjacencies.get(visiting).unwrap_or_else(|| {
                panic!(
                    "could not find cave '{}' in adjacency map, but should have been able to",
                    visiting.name
                )
            });

            for adj in visiting_adjancencies.iter() {
                if let Some(next_hop) = self.generate_next_hops(adj, &path) {
                    let mut new_path = path.clone();
                    new_path.push(next_hop);

                    to_visit.push((next_hop, new_path));
                }
            }
        }

        paths.len()
    }

    /// Generate the next hop in the path, should one be possible
    fn generate_next_hops(&self, target: &'a Cave, path: &[&Cave]) -> Option<&'a Cave> {
        if target.is_big() {
            return Some(target);
        }

        let get_next_hop_if_not_in_path = |cave| (!path.contains(cave)).then(|| target);
        if target.name == START_CAVE_NAME || target.name == END_CAVE_NAME {
            return get_next_hop_if_not_in_path(&target);
        }

        match self.part {
            Part::Part1 => get_next_hop_if_not_in_path(&target),
            Part::Part2 => {
                let counts = count_times_cave_encountered(path);
                let have_gone_somewhere_twice = counts
                    .iter()
                    .any(|(cave, &count)| !cave.is_big() && count == 2);
                let num_target_visits = *counts.get(target).unwrap_or(&0);
                let target_should_be_next_hop = (have_gone_somewhere_twice
                    && num_target_visits == 0)
                    || (!have_gone_somewhere_twice && num_target_visits < 2);

                target_should_be_next_hop.then(|| target)
            }
        }
    }
}

/// Count the number of times that each cave was encountered
fn count_times_cave_encountered<'a>(path: &[&'a Cave]) -> HashMap<&'a Cave, usize> {
    let mut counts = HashMap::<&Cave, usize>::new();
    for element in path {
        *counts.entry(element).or_insert(0) += 1;
    }

    counts
}

fn parse_cave(s: &str) -> IResult<&str, Cave> {
    let (remaining, cave_name) = alt((
        take_while1(|c: char| c.is_ascii_uppercase()),
        take_while1(|c: char| c.is_ascii_lowercase()),
    ))(s)?;

    Ok((
        remaining,
        Cave {
            name: cave_name.to_string(),
        },
    ))
}

fn parse_line(line: &str) -> IResult<&str, (Cave, Cave)> {
    terminated(separated_pair(parse_cave, char('-'), parse_cave), eof)(line)
}

fn adjacencies_to_map(adjacencies: &[(Cave, Cave)]) -> HashMap<&Cave, Vec<&Cave>> {
    let mut res = HashMap::<&Cave, Vec<&Cave>>::new();
    for (start, end) in adjacencies {
        let maybe_start_bucket = res.get_mut(&start);
        if let Some(bucket) = maybe_start_bucket {
            bucket.push(end);
        } else {
            res.insert(start, vec![end]);
        }

        // The graph is not directional so we must mirror the edges
        let maybe_end_bucket = res.get_mut(&end);
        if let Some(bucket) = maybe_end_bucket {
            bucket.push(start);
        } else {
            res.insert(end, vec![start]);
        }
    }

    res
}

fn part1(adjacencies: &HashMap<&Cave, Vec<&Cave>>) -> usize {
    Puzzle {
        part: Part::Part1,
        adjacencies,
    }
    .find_num_paths()
}

fn part2(adjacencies: &HashMap<&Cave, Vec<&Cave>>) -> usize {
    Puzzle {
        part: Part::Part2,
        adjacencies,
    }
    .find_num_paths()
}

fn main() {
    let input_file_name = env::args().nth(1).expect("No input filename specified");
    let input_file = File::open(input_file_name).expect("Could not open input file");

    let adjacencies = BufReader::new(input_file)
        .lines()
        .map(|res| res.expect("Failed to read line"))
        .map(|s| {
            let (_, adjacency) = parse_line(&s).expect("Failed to read line");
            adjacency
        })
        .collect::<Vec<_>>();

    let cave_mappings = adjacencies_to_map(&adjacencies);
    println!("Part 1: {}", part1(&cave_mappings));
    println!("Part 2: {}", part2(&cave_mappings));
}
