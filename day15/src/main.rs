#![warn(clippy::all, clippy::pedantic)]
// Needed for auto_ops to work properly
#[allow(clippy::wildcard_imports)]
use std::collections::{BinaryHeap, HashMap};
use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader};

/// Represesents -1/0/1 for the purposes of calculating adjacencies
// exists strictly to work around the limitation that I can't have a negative usize, nor do the additions
// without annoying conversions
#[derive(Clone, Copy, Debug)]
enum AdjacencyDelta {
    NegativeOne,
    Zero,
    One,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
struct SearchPosition {
    risk: u32,
    position: (usize, usize),
}
impl PartialOrd for SearchPosition {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for SearchPosition {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.risk.cmp(&other.risk).reverse()
    }
}

impl AdjacencyDelta {
    fn try_add(self, size: usize) -> Option<usize> {
        match self {
            AdjacencyDelta::NegativeOne => {
                if size == 0 {
                    None
                } else {
                    Some(size - 1)
                }
            }
            AdjacencyDelta::Zero => Some(size),
            AdjacencyDelta::One => Some(size + 1),
        }
    }
}

fn get_adjacent_indices(
    input: &[Vec<u8>],
    (row_cursor, col_cursor): (usize, usize),
) -> Vec<(usize, usize)> {
    let adj_deltas = [
        AdjacencyDelta::NegativeOne,
        AdjacencyDelta::Zero,
        AdjacencyDelta::One,
    ];

    let mut res = vec![];
    for d_row in adj_deltas {
        for d_col in adj_deltas {
            // We can't go diagonally
            // if mem::discriminant(&d_row) == mem::discriminant(&d_col) {
            if !(matches!(d_row, AdjacencyDelta::Zero) || matches!(d_col, AdjacencyDelta::Zero)) {
                continue;
            }

            let next_row = d_row.try_add(row_cursor);
            let next_col = d_col.try_add(col_cursor);
            if let (Some(row), Some(col)) = (next_row, next_col) {
                if row < input.len() && col < input[0].len() {
                    res.push((row, col));
                }
            }
        }
    }

    res
}

/// Find the final cost from source to target within the given input board.
/// The `node_parents` map must provide a valid ancestry from start to finish
/// using Djikstra's algorithm. If there is no path, None is returned.
fn find_cost_from_path(
    input: &[Vec<u8>],
    source: (usize, usize),
    target: (usize, usize),
    node_parents: &HashMap<(usize, usize), (usize, usize)>,
) -> Option<u32> {
    let mut cost_cursor = target;
    let mut cost = 0;

    while cost_cursor != source {
        cost += u32::from(input[cost_cursor.0][cost_cursor.1]);
        let parent = node_parents.get(&cost_cursor)?;

        cost_cursor = *parent;
    }

    Some(cost)
}

/// Part 1 will search for the solved path using Djikstra's algorithm
fn part1(input: &[Vec<u8>]) -> u32 {
    let mut risks = HashMap::<(usize, usize), u32>::new();
    let mut node_parents = HashMap::<(usize, usize), (usize, usize)>::new();
    risks.insert((0, 0), 0);

    let mut visit_queue = BinaryHeap::<SearchPosition>::new();
    visit_queue.push(SearchPosition {
        position: (0, 0),
        risk: 0,
    });

    let target_pos = (input.len() - 1, input[0].len() - 1);

    while let Some(visiting_node) = visit_queue.pop() {
        if visiting_node.position == target_pos {
            break;
        }

        // From Wikipedia:
        //
        // Yet another alternative is to add nodes unconditionally to the priority queue and to instead check after
        // extraction that no shorter connection was found yet. This can be done by additionally extracting the
        // associated priority p from the queue and only processing further if p == dist[u] inside the while Q
        // is not empty loop.
        if visiting_node.risk != *risks.get(&visiting_node.position).unwrap() {
            continue;
        }

        for neighbor_pos in get_adjacent_indices(input, visiting_node.position) {
            let (neighbor_row, neighbor_col) = neighbor_pos;
            let neighbor_risk = input[neighbor_row][neighbor_col];
            let risk_candidate = visiting_node.risk + u32::from(neighbor_risk);
            if !risks.contains_key(&neighbor_pos)
                || risk_candidate < *risks.get(&neighbor_pos).unwrap()
            {
                risks.insert(neighbor_pos, risk_candidate);
                node_parents.insert(neighbor_pos, visiting_node.position);
                visit_queue.push(SearchPosition {
                    risk: risk_candidate,
                    position: neighbor_pos,
                });
            }
        }
    }

    find_cost_from_path(input, (0, 0), target_pos, &node_parents)
        .expect("No cost could be calculated; invalid ancestry map is likely")
}

/// Create an iterator that iterates over the given slice n times
fn iterate_slice_n_times<T>(slice: &[T], n: usize) -> impl Iterator<Item = &T> {
    let num_to_take = slice.len() * n;
    slice.iter().cycle().take(num_to_take)
}

/// Generate the expanded board for part 2
fn generate_expanded_board(input: &[Vec<u8>]) -> Vec<Vec<u8>> {
    let mut expanded_input = vec![];
    for (i, row) in iterate_slice_n_times(input, 5).enumerate() {
        let mut res_row = vec![];
        for (j, original_risk) in iterate_slice_n_times(row, 5).enumerate() {
            let row_tile = u8::try_from(i / input.len()).unwrap();
            let col_tile = u8::try_from(j / input.len()).unwrap();

            let risk_offset = row_tile + col_tile;
            let new_risk_candidate = original_risk + risk_offset;
            let wrapped_risk = (new_risk_candidate - 1) % 9 + 1;

            res_row.push(wrapped_risk);
        }

        expanded_input.push(res_row);
    }

    expanded_input
}

fn part2(input: &[Vec<u8>]) -> u32 {
    let expanded_input = generate_expanded_board(input);
    part1(&expanded_input)
}

fn main() {
    let input_file_name = env::args().nth(1).expect("No input filename specified");
    let input_file = File::open(input_file_name).expect("Could not open input file");
    let input_lines = BufReader::new(input_file)
        .lines()
        .map(|res| res.expect("Failed to read line"))
        .map(|s| {
            s.chars()
                .map(|c| {
                    c.to_digit(10)
                        .unwrap_or_else(|| panic!("Expected all chars to be digits, found {}", c))
                        .try_into()
                        // This is a 0-9, so we will always fit into a u8
                        .unwrap()
                })
                .collect::<Vec<u8>>()
        })
        .collect::<Vec<_>>();

    let first_row_length = input_lines.get(0).expect("input must be non-empty").len();
    assert!(
        input_lines.iter().all(|row| row.len() == first_row_length),
        "All input lines must be the same length"
    );

    println!("Part 1: {}", part1(&input_lines));
    println!("Part 2: {}", part2(&input_lines));
}
