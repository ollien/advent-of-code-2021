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

#[derive(Debug, Hash, Clone, PartialEq, Eq)]
struct Cave {
    name: String,
}

impl Cave {
    fn is_big(&self) -> bool {
        self.name.chars().any(|c| c.is_ascii_uppercase())
    }
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
    let start_cave = Cave {
        name: START_CAVE_NAME.to_string(),
    };
    let start_adjacencies = adjacencies
        .get(&start_cave)
        .expect("Input did not contain start cave");

    let mut to_visit = start_adjacencies
        .iter()
        .map(|cave| (cave, vec![&start_cave, cave]))
        .collect::<Vec<_>>();
    let mut paths = vec![];
    while let Some((visiting, path)) = to_visit.pop() {
        if visiting.name == END_CAVE_NAME {
            paths.push(path);
            continue;
        }

        let visiting_adjancencies = adjacencies.get(visiting).unwrap_or_else(|| {
            panic!(
                "could not find cave '{}' in adjacency map, but should have been able to",
                visiting.name
            )
        });

        for adj in visiting_adjancencies.iter() {
            let have_visited = !adj.is_big() && path.contains(adj);
            if have_visited {
                continue;
            }

            let mut new_path = path.clone();
            new_path.push(adj);

            to_visit.push((adj, new_path));
        }
    }

    paths.len()
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
}
