#![warn(clippy::all, clippy::pedantic)]
use itertools::Itertools;
use std::env;
use std::fmt;
use std::fmt::Debug;
use std::fmt::Formatter;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::mem;

use nom::{
    branch::alt,
    character::complete::char,
    character::complete::digit1,
    combinator::{eof, map_res},
    sequence::{delimited, separated_pair, terminated},
    IResult,
};
use petgraph::dot::{Config, Dot};
use petgraph::graph::NodeIndex;
use petgraph::stable_graph::StableDiGraph;

#[derive(Clone, Debug)]
enum InputPair {
    Pair(Box<InputPair>, Box<InputPair>),
    Leaf(u32),
}

#[derive(Clone, Copy, Debug)]
enum PairNode {
    PairRoot,
    Leaf(u32),
}

#[derive(Clone, Copy, Debug)]
enum Direction {
    Left,
    Right,
}

#[derive(Clone, Copy, Debug)]
enum EdgeType {
    Parent,
    Child(Direction),
}

#[derive(Clone)]
struct ProblemTree {
    graph: StableDiGraph<PairNode, EdgeType>,
    root_idx: NodeIndex,
}

impl Direction {
    fn get_other(self) -> Self {
        match self {
            Self::Left => Self::Right,
            Self::Right => Self::Left,
        }
    }
}

impl ProblemTree {
    fn build(left: &InputPair, right: &InputPair) -> Self {
        let mut graph = StableDiGraph::new();
        let root_idx = graph.add_node(PairNode::PairRoot);
        let mut tree = ProblemTree { graph, root_idx };
        tree.insert_input_pair(left, Direction::Left, root_idx);
        tree.insert_input_pair(right, Direction::Right, root_idx);

        tree
    }

    fn insert_input_pair(&mut self, pair: &InputPair, direction: Direction, parent_idx: NodeIndex) {
        match pair {
            &InputPair::Leaf(n) => {
                self.insert_leaf(n, direction, parent_idx);
            }
            InputPair::Pair(left, right) => {
                let root_idx = self.graph.add_node(PairNode::PairRoot);
                // We're inserting twice into the directional graph so we can differentiate between parent/child relationships
                self.graph
                    .add_edge(parent_idx, root_idx, EdgeType::Child(direction));
                self.graph.add_edge(root_idx, parent_idx, EdgeType::Parent);
                self.insert_input_pair(left, Direction::Left, root_idx);
                self.insert_input_pair(right, Direction::Right, root_idx);
            }
        }
    }

    /// inserts a pair to the tree that will be a sibling to the root, with a new root being planted in this tree
    fn insert_root_sibling_input_pair(&mut self, pair: &InputPair, direction: Direction) {
        let new_root_idx = self.graph.add_node(PairNode::PairRoot);
        let old_root_idx = self.root_idx;
        self.root_idx = new_root_idx;
        self.graph.add_edge(
            new_root_idx,
            old_root_idx,
            EdgeType::Child(direction.get_other()),
        );
        self.graph
            .add_edge(old_root_idx, new_root_idx, EdgeType::Parent);

        self.insert_input_pair(pair, direction, new_root_idx);
    }

    fn insert_leaf(&mut self, value: u32, direction: Direction, parent_idx: NodeIndex) {
        let leaf_idx = self.graph.add_node(PairNode::Leaf(value));
        // We're inserting twice into the directional graph so we can differentiate between parent/child relationships
        self.graph
            .add_edge(parent_idx, leaf_idx, EdgeType::Child(direction));
        self.graph.add_edge(leaf_idx, parent_idx, EdgeType::Parent);
    }

    fn magnitude(&mut self) -> u32 {
        let mut to_visit = vec![(1, self.root_idx)];
        let mut total = 0;
        while let Some((n, visiting_idx)) = to_visit.pop() {
            let visiting = self.graph.node_weight(visiting_idx).unwrap();
            let neighbors = self.graph.neighbors(visiting_idx);

            match visiting {
                PairNode::PairRoot => {
                    for neighbor in neighbors {
                        let edge_idx = self.graph.find_edge(visiting_idx, neighbor).unwrap();
                        let edge_type = self.graph.edge_weight(edge_idx).unwrap();
                        match edge_type {
                            EdgeType::Child(Direction::Left) => to_visit.push((3 * n, neighbor)),
                            EdgeType::Child(Direction::Right) => to_visit.push((2 * n, neighbor)),
                            EdgeType::Parent => (),
                        }
                    }
                }
                PairNode::Leaf(visiting_value) => {
                    total += n * visiting_value;
                    // for neighbor in neighbors {
                    //     let edge_idx = self.graph.find_edge(visiting_idx, neighbor).unwrap();
                    //     let edge_type = self.graph.edge_weight(edge_idx).unwrap();
                    // }
                }
            };
        }

        total
    }

    fn reduce(&mut self) {
        let mut performed_action: Option<bool> = None;
        while performed_action.unwrap_or(true) {
            performed_action = Some(false);
            let explode_candidate = self.find_node_to_explode();
            let split_candidate = self.find_node_to_split();

            if let Some(to_explode) = explode_candidate {
                performed_action = Some(true);
                self.explode_in_relative_direction(to_explode, Direction::Left);
                self.explode_in_relative_direction(to_explode, Direction::Right);

                let neighbors = self.graph.neighbors(to_explode).collect::<Vec<_>>();
                for neighbor in neighbors {
                    if let PairNode::Leaf(_) = self.graph.node_weight(neighbor).unwrap() {
                        self.graph.remove_node(dbg!(neighbor));
                    }
                }

                let to_explode_weight = self.graph.node_weight_mut(to_explode).unwrap();
                *to_explode_weight = PairNode::Leaf(0);
                // print_tree(self);
                dbg!(&self);
                // std::io::stdin().read_line(&mut String::new());
            }

            if performed_action.unwrap_or(false) {
                continue;
            }

            if let Some(to_split) = split_candidate {
                performed_action = Some(true);
                self.split_node(to_split);

                dbg!(&self);
                // std::io::stdin().read_line(&mut String::new());
            }
        }
    }

    fn find_node_to_explode(&self) -> Option<NodeIndex> {
        self.find_node_to_reduce_below_or_at(self.root_idx, 0, |node, depth| {
            depth >= 4 && matches!(node, PairNode::PairRoot)
        })
    }

    fn find_node_to_split(&self) -> Option<NodeIndex> {
        self.find_node_to_reduce_below_or_at(self.root_idx, 0, |node, _| {
            if let PairNode::Leaf(n) = node {
                n >= 10
            } else {
                false
            }
        })
    }

    fn find_node_to_reduce_below_or_at<F>(
        &self,
        below_idx: NodeIndex,
        node_depth: usize,
        criteria: F,
    ) -> Option<NodeIndex>
    where
        // Takes the node itself and its depth
        F: Copy + Fn(PairNode, usize) -> bool,
    {
        let find_from_child = |child_idx| {
            let node = self
                .graph
                .node_weight(child_idx)
                .expect("got a child that didn't exist in the graph");

            // if let PairNode::Leaf(_) = node {
            //     return None;
            // }

            let next_depth = node_depth + 1;
            // println!(
            //     "{:?} (id={:?}) @ {} => {}",
            //     node, child_idx, node_depth, next_depth
            // );

            self.find_node_to_reduce_below_or_at(child_idx, next_depth, criteria)
        };

        let below_type = self.graph.node_weight(below_idx).unwrap();
        let left_child = self.get_child(below_idx, Direction::Left);
        let right_child = self.get_child(below_idx, Direction::Right);
        let left_candidate = left_child.and_then(find_from_child);
        let right_candidate = right_child.and_then(find_from_child);

        if left_candidate.is_some() {
            // dbg!(self.graph.node_weight(
            //     self.get_child(left_candidate.unwrap(), Direction::Left)
            //         .unwrap()
            // ));

            left_candidate
        } else if right_candidate.is_some() {
            right_candidate
        } else if criteria(*below_type, node_depth) {
            Some(below_idx)
        } else {
            None
        }
    }

    fn get_parent(&self, node_idx: NodeIndex) -> Option<NodeIndex> {
        // There must be zero or one by construction of the graph
        self.graph.neighbors(node_idx).find(|&neighbor_idx| {
            // This must exist by the fact that we've been returned a neighbor
            let edge = self.graph.find_edge(node_idx, neighbor_idx).unwrap();
            let edge_type = self.graph.edge_weight(edge).unwrap();

            matches!(edge_type, EdgeType::Parent)
        })
    }

    fn get_child(&self, node_idx: NodeIndex, direction: Direction) -> Option<NodeIndex> {
        // There should only be one or zero in the iterator, by the construction of the graph.
        self.graph.neighbors(node_idx).find(|&neighbor_idx| {
            // This must exist by the fact that we've been returned a neighbor
            let edge = self.graph.find_edge(node_idx, neighbor_idx).unwrap();
            let edge_type = self.graph.edge_weight(edge).unwrap();
            if let EdgeType::Child(child_direction) = edge_type {
                mem::discriminant(child_direction) == mem::discriminant(&direction)
            } else {
                false
            }
        })
    }

    fn explode_in_relative_direction(
        &mut self,
        to_explode: NodeIndex,
        direction: Direction,
    ) -> bool {
        let explode_value = self
            .graph
            .node_weight(self.get_child(to_explode, direction).unwrap())
            .map(|node| {
                if let PairNode::Leaf(n) = node {
                    *n
                } else {
                    // TODO: Probably shouldn't panic but I don't want to make errors right now
                    panic!("got a non-leaf when looking for explode node");
                }
            })
            .unwrap();

        println!("Exploding: {:?} (id={:?})", explode_value, to_explode);

        let relative_direction_node_candidate =
            self.get_leaf_in_relative_direction(to_explode, direction);

        if let Some(relative_direction_node_idx) = relative_direction_node_candidate {
            if relative_direction_node_idx == to_explode {
                return false;
            }

            let relative_direction_node = self
                .graph
                .node_weight_mut(relative_direction_node_idx)
                .unwrap();
            // println!("{:?} {:?}", relative_direction_node_idx, to_explode);
            if let PairNode::Leaf(n) = relative_direction_node {
                println!(
                    "{:?}: {} (id={:?})",
                    direction, *n, relative_direction_node_idx
                );
                *n += explode_value;
            } else {
                panic!("got a non-leaf from directional leaf lookup");
            }

            true
        } else {
            false
        }
    }

    fn split_node(&mut self, node_idx: NodeIndex) {
        // TODO: Probably shouldn't panic but I don't want to write errors right now
        let node = self.graph.node_weight_mut(node_idx).unwrap();
        let n = if let PairNode::Leaf(n) = node {
            *n
        } else {
            // TODO: _definitely_ shouldn't panic but I don't want to write errors right now
            panic!("cannot explode non-leaf")
        };

        let (left_value, right_value) = get_split_values(n);
        *node = PairNode::PairRoot;
        self.insert_leaf(left_value, Direction::Left, node_idx);
        self.insert_leaf(right_value, Direction::Right, node_idx);
    }

    fn get_leaf_in_relative_direction(
        &self,
        node_idx: NodeIndex,
        direction: Direction,
    ) -> Option<NodeIndex> {
        let mut prev_cursor = node_idx;
        let mut cursor = self.get_parent(node_idx)?;
        let have_found_root_like_node = |prev_cursor, cursor| {
            let cursor_left_child = self.get_child(cursor, Direction::Left).unwrap();
            let cursor_right_child = self.get_child(cursor, Direction::Right).unwrap();
            let cursor_left_child_node = self.graph.node_weight(cursor_left_child).unwrap();
            let cursor_right_child_node = self.graph.node_weight(cursor_right_child).unwrap();

            match direction {
                Direction::Right => {
                    if cursor_right_child == prev_cursor {
                        return false;
                    }
                }
                Direction::Left => {
                    if cursor_left_child == prev_cursor {
                        return false;
                    }
                }
            }

            matches!(cursor_left_child_node, PairNode::PairRoot)
                && matches!(cursor_right_child_node, PairNode::PairRoot)
        };

        while !have_found_root_like_node(prev_cursor, cursor) {
            if let Some(directional_child) = self.get_child(cursor, direction) {
                let directional_child_node = self.graph.node_weight(directional_child).unwrap();
                if directional_child != cursor
                    && matches!(directional_child_node, PairNode::Leaf(_))
                {
                    return Some(directional_child);
                }
            }

            prev_cursor = cursor;
            cursor = self.get_parent(cursor)?;
        }

        // once we hit the "root", we need to start going downwards by one level, and descend as far as possible
        // in the opposite direction.
        let flip_around_node = self.get_child(cursor, direction)?;
        if flip_around_node == prev_cursor {
            return None;
        }

        cursor = flip_around_node;
        loop {
            let child_candidate = self.get_child(cursor, direction.get_other());
            if let Some(child) = child_candidate {
                cursor = child;
            } else {
                return Some(cursor);
            }
        }
    }

    fn debug_tree(&self, formatter: &mut Formatter<'_>, node_idx: NodeIndex) -> fmt::Result {
        let node_candidate = self.graph.node_weight(node_idx);
        if node_candidate.is_none() {
            return Err(fmt::Error);
        }

        let node = node_candidate.unwrap();
        match node {
            PairNode::Leaf(n) => write!(formatter, "{}", n)?,
            PairNode::PairRoot => {
                let left_node_idx_candidate = self.get_child(node_idx, Direction::Left);
                let right_node_idx_candidate = self.get_child(node_idx, Direction::Right);
                if left_node_idx_candidate.is_none() || left_node_idx_candidate.is_none() {
                    return Err(fmt::Error);
                }

                let left_node_idx = left_node_idx_candidate.unwrap();
                let right_node_idx = right_node_idx_candidate.unwrap();
                write!(formatter, "[")?;
                self.debug_tree(formatter, left_node_idx)?;
                write!(formatter, ",")?;
                self.debug_tree(formatter, right_node_idx)?;
                write!(formatter, "]")?;
            }
        };

        Ok(())
    }
}

impl Debug for ProblemTree {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        self.debug_tree(formatter, self.root_idx)
    }
}

fn get_split_values(n: u32) -> (u32, u32) {
    let left = n / 2;
    let right = if n % 2 == 0 { n / 2 } else { n / 2 + 1 };

    println!("Splitting: {} => {}, {}", n, left, right);

    (left, right)
}

fn parse_snailfish_problem_leaf(chunk: &str) -> IResult<&str, InputPair> {
    let (remaining, n) = map_res(digit1, str::parse)(chunk)?;
    Ok((remaining, InputPair::Leaf(n)))
}

fn parse_snailfish_problem_pair(chunk: &str) -> IResult<&str, InputPair> {
    let (remaining, (pair1, pair2)) = delimited(
        char('['),
        separated_pair(
            alt((parse_snailfish_problem_pair, parse_snailfish_problem_leaf)),
            char(','),
            alt((parse_snailfish_problem_pair, parse_snailfish_problem_leaf)),
        ),
        char(']'),
    )(chunk)?;

    let res_pair = InputPair::Pair(Box::new(pair1), Box::new(pair2));

    Ok((remaining, res_pair))
}

fn parse_snailfish_problem(input: &str) -> IResult<&str, InputPair> {
    terminated(parse_snailfish_problem_pair, eof)(input)
}

fn part1(input_pairs: &[InputPair]) -> u32 {
    let mut problem_tree = if let InputPair::Pair(left, right) = &input_pairs[0] {
        ProblemTree::build(&*left, &*right)
    } else {
        panic!("input pair should be a pair");
    };

    problem_tree.reduce();

    for input_pair in &input_pairs[1..] {
        problem_tree.insert_root_sibling_input_pair(input_pair, Direction::Right);
        problem_tree.reduce();
        dbg!(&problem_tree);
        // print_tree(&problem_tree);
        // std::io::stdin().read_line(&mut String::new());
    }
    // print_tree(&problem_tree);
    dbg!(&problem_tree);

    problem_tree.magnitude()
}

fn part2(input_pairs: &[InputPair]) -> u32 {
    input_pairs
        .into_iter()
        .permutations(2)
        .map(|pairs| {
            let pair1 = pairs[0];
            let pair2 = pairs[1];

            let mut tree1 = if let InputPair::Pair(left, right) = pair1 {
                ProblemTree::build(&*left, &*right)
            } else {
                panic!("input pair should be a pair");
            };

            tree1.insert_root_sibling_input_pair(pair2, Direction::Right);
            tree1.reduce();

            tree1.magnitude()
        })
        .max()
        .expect("should be at least one input pair")
}

fn print_tree(problem_tree: &ProblemTree) {
    println!("{:?}", Dot::with_config(&problem_tree.graph, &[]));
    // let mut to_visit = vec![(0, problem_tree.root_idx)];
    // while let Some((indentation, idx)) = to_visit.pop() {
    //     let node = problem_tree.graph.node_weight(idx).unwrap();
    //     match node {
    //         PairNode::Leaf(n) => println!(
    //             "{}{}",
    //             (0..=indentation).map(|_| "  ").collect::<String>(),
    //             n
    //         ),
    //         PairNode::PairRoot => {
    //             for neighbor in problem_tree.graph.neighbors(idx) {
    //                 let edge_idx = problem_tree.graph.find_edge(idx, neighbor).unwrap();
    //                 let edge_type = problem_tree.graph.edge_weight(edge_idx).unwrap();
    //                 if let EdgeType::Child(_) = edge_type {
    //                     to_visit.push((indentation + 1, neighbor));
    //                 }
    //             }
    //         }
    //     }
    // }
}

fn main() {
    let input_file_name = env::args().nth(1).expect("No input filename specified");
    let input_file = File::open(input_file_name).expect("Could not open input file");
    let all_pairs = BufReader::new(input_file)
        .lines()
        .map(|res| res.expect("Failed to read line"))
        .filter_map(|line| {
            if line.is_empty() {
                return None;
            }

            let (_, input_pair) =
                parse_snailfish_problem(&line).expect("Failed to parse input line");
            Some(input_pair)
        })
        .collect::<Vec<_>>();
    // .explode(|left, right| InputPair::Pair(Box::new(left), Box::new(right)))
    // .expect("did not find input problem");

    println!("Part 1: {}", part1(&all_pairs));
    println!("Part 2: {}", part2(&all_pairs));
}
