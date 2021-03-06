//! This solution is very messy, but after the toil it took to get right, I feel a bit lazy cleaning it up.
//! Sorry :(
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
use petgraph::graph::NodeIndex;
use petgraph::stable_graph::StableDiGraph;
use thiserror::Error;

#[derive(Error, Debug)]
enum Error {
    #[error("could not find node with index {0:?}")]
    NodeNotFound(NodeIndex),
    #[error("expected a leaf, but got {0:?}")]
    ExpectedLeaf(PairNode),
    #[error("expected a pair root, but got {0:?}")]
    ExpectedPairRoot(PairNode),
}

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
                }
            };
        }

        total
    }

    fn reduce(&mut self) -> Result<(), Error> {
        let mut performed_action: Option<bool> = None;
        while performed_action.unwrap_or(true) {
            performed_action = Some(false);
            let explode_candidate = self.find_node_to_explode()?;
            let split_candidate = self.find_node_to_split()?;

            if let Some(to_explode) = explode_candidate {
                performed_action = Some(true);
                self.explode_in_relative_direction(to_explode, Direction::Left)?;
                self.explode_in_relative_direction(to_explode, Direction::Right)?;

                let neighbors = self.graph.neighbors(to_explode).collect::<Vec<_>>();
                for neighbor in neighbors {
                    if let PairNode::Leaf(_) = self.graph.node_weight(neighbor).unwrap() {
                        self.graph.remove_node(neighbor);
                    }
                }

                let to_explode_weight = self.graph.node_weight_mut(to_explode).unwrap();
                *to_explode_weight = PairNode::Leaf(0);
            }

            if performed_action.unwrap_or(false) {
                continue;
            }

            if let Some(to_split) = split_candidate {
                performed_action = Some(true);
                self.split_node(to_split)?;
            }
        }

        Ok(())
    }

    fn find_node_to_explode(&self) -> Result<Option<NodeIndex>, Error> {
        self.find_node_to_reduce_below_or_at(self.root_idx, 0, |node, depth| {
            depth >= 4 && matches!(node, PairNode::PairRoot)
        })
    }

    fn find_node_to_split(&self) -> Result<Option<NodeIndex>, Error> {
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
    ) -> Result<Option<NodeIndex>, Error>
    where
        // Takes the node itself and its depth
        F: Copy + Fn(PairNode, usize) -> bool,
    {
        let find_from_child = |child_idx| {
            let next_depth = node_depth + 1;
            self.find_node_to_reduce_below_or_at(child_idx, next_depth, criteria)
        };

        let below_type = self.graph.node_weight(below_idx).unwrap();
        let left_child = self.get_child(below_idx, Direction::Left)?;
        let right_child = self.get_child(below_idx, Direction::Right)?;
        let left_candidate = left_child.map(find_from_child);
        let right_candidate = right_child.map(find_from_child);

        if let Some(Err(left_err)) = left_candidate {
            return Err(left_err);
        } else if let Some(Err(right_err)) = right_candidate {
            return Err(right_err);
        }

        if let Some(Ok(Some(left_res))) = left_candidate {
            Ok(Some(left_res))
        } else if let Some(Ok(Some(right_res))) = right_candidate {
            Ok(Some(right_res))
        } else if criteria(*below_type, node_depth) {
            Ok(Some(below_idx))
        } else {
            Ok(None)
        }
    }

    fn get_parent(&self, node_idx: NodeIndex) -> Result<Option<NodeIndex>, Error> {
        if self.graph.node_weight(node_idx).is_none() {
            return Err(Error::NodeNotFound(node_idx));
        }

        // There must be zero or one by construction of the graph
        let parent_candidate = self.graph.neighbors(node_idx).find(|&neighbor_idx| {
            // This must exist by the fact that we've been returned a neighbor
            let edge = self.graph.find_edge(node_idx, neighbor_idx).unwrap();
            let edge_type = self.graph.edge_weight(edge).unwrap();

            matches!(edge_type, EdgeType::Parent)
        });

        Ok(parent_candidate)
    }

    fn get_child(
        &self,
        node_idx: NodeIndex,
        direction: Direction,
    ) -> Result<Option<NodeIndex>, Error> {
        if self.graph.node_weight(node_idx).is_none() {
            return Err(Error::NodeNotFound(node_idx));
        }

        // There should only be one or zero in the iterator, by the construction of the graph.
        let child_candidate = self.graph.neighbors(node_idx).find(|&neighbor_idx| {
            // This must exist by the fact that we've been returned a neighbor
            let edge = self.graph.find_edge(node_idx, neighbor_idx).unwrap();
            let edge_type = self.graph.edge_weight(edge).unwrap();
            if let EdgeType::Child(child_direction) = edge_type {
                mem::discriminant(child_direction) == mem::discriminant(&direction)
            } else {
                false
            }
        });

        Ok(child_candidate)
    }

    fn explode_in_relative_direction(
        &mut self,
        to_explode: NodeIndex,
        direction: Direction,
    ) -> Result<(), Error> {
        let explosion_child_idx_candidate = self.get_child(to_explode, direction)?;
        if explosion_child_idx_candidate.is_none() {
            // If this would have failed get_child would have returned an err
            let to_explode_node = self.graph.node_weight(to_explode).unwrap();
            return Err(Error::ExpectedPairRoot(*to_explode_node));
        }
        let explode_value = self
            .graph
            .node_weight(self.get_child(to_explode, direction)?.unwrap())
            .map(|node| {
                if let PairNode::Leaf(n) = node {
                    Ok(*n)
                } else {
                    Err(Error::ExpectedLeaf(*node))
                }
            })
            .unwrap()?;

        let relative_direction_node_candidate =
            self.get_leaf_in_relative_direction(to_explode, direction)?;

        if let Some(relative_direction_node_idx) = relative_direction_node_candidate {
            if relative_direction_node_idx == to_explode {
                return Ok(());
            }

            let relative_direction_node = self
                .graph
                .node_weight_mut(relative_direction_node_idx)
                .unwrap();
            if let PairNode::Leaf(n) = relative_direction_node {
                *n += explode_value;
            } else {
                panic!("got a non-leaf from directional leaf lookup");
            }
        }

        Ok(())
    }

    fn split_node(&mut self, node_idx: NodeIndex) -> Result<(), Error> {
        let node = self
            .graph
            .node_weight_mut(node_idx)
            .ok_or(Error::NodeNotFound(node_idx))?;

        let n = if let PairNode::Leaf(n) = node {
            *n
        } else {
            return Err(Error::ExpectedLeaf(*node));
        };

        let (left_value, right_value) = get_split_values(n);
        *node = PairNode::PairRoot;
        self.insert_leaf(left_value, Direction::Left, node_idx);
        self.insert_leaf(right_value, Direction::Right, node_idx);

        Ok(())
    }

    fn get_leaf_in_relative_direction(
        &self,
        node_idx: NodeIndex,
        direction: Direction,
    ) -> Result<Option<NodeIndex>, Error> {
        let mut prev_cursor = node_idx;
        // this can still work with the root, but we must start with it
        let mut cursor = self.get_parent(node_idx)?.unwrap_or(node_idx);

        // Find any node where there is a right and left pair node as children (but do not allow us to find one
        // that backtracks us to where we just were)
        let have_found_root_like_node = |prev_cursor, cursor| {
            let cursor_left_child = self.get_child(cursor, Direction::Left)?.unwrap();
            let cursor_right_child = self.get_child(cursor, Direction::Right)?.unwrap();
            let cursor_left_child_node = self.graph.node_weight(cursor_left_child).unwrap();
            let cursor_right_child_node = self.graph.node_weight(cursor_right_child).unwrap();

            match direction {
                Direction::Right => {
                    if cursor_right_child == prev_cursor {
                        return Ok(false);
                    }
                }
                Direction::Left => {
                    if cursor_left_child == prev_cursor {
                        return Ok(false);
                    }
                }
            }

            let found = matches!(cursor_left_child_node, PairNode::PairRoot)
                && matches!(cursor_right_child_node, PairNode::PairRoot);

            Ok(found)
        };

        while !have_found_root_like_node(prev_cursor, cursor)? {
            if let Some(directional_child) = self.get_child(cursor, direction)? {
                let directional_child_node = self.graph.node_weight(directional_child).unwrap();
                if directional_child != cursor
                    && matches!(directional_child_node, PairNode::Leaf(_))
                {
                    return Ok(Some(directional_child));
                }
            }

            let cursor_candidate = self.get_parent(cursor)?;
            if cursor_candidate.is_none() {
                return Ok(None);
            }

            prev_cursor = cursor;
            cursor = cursor_candidate.unwrap();
        }

        // once we hit the "root", we need to start going downwards by one level, and descend as far as possible
        // in the opposite direction.
        //
        // By construction, the child must exist (the only way it can't is if we only have a root node, which can't
        // happen with a valid input).
        let flip_around_node = self.get_child(cursor, direction)?.unwrap();
        if flip_around_node == prev_cursor {
            return Ok(None);
        }

        cursor = flip_around_node;
        loop {
            let child_candidate = self.get_child(cursor, direction.get_other())?;
            if let Some(child) = child_candidate {
                cursor = child;
            } else {
                return Ok(Some(cursor));
            }
        }
    }

    /// Helper for implementing the Debug trait
    fn debug_tree(&self, formatter: &mut Formatter<'_>, node_idx: NodeIndex) -> fmt::Result {
        let node_candidate = self.graph.node_weight(node_idx);
        if node_candidate.is_none() {
            return Err(fmt::Error);
        }

        let node = node_candidate.unwrap();
        match node {
            PairNode::Leaf(n) => write!(formatter, "{}", n)?,
            PairNode::PairRoot => {
                // By construction, we must have both children
                let left_node_idx_candidate = self.get_child(node_idx, Direction::Left).unwrap();
                let right_node_idx_candidate = self.get_child(node_idx, Direction::Right).unwrap();
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

    problem_tree.reduce().expect("Failed to perform reduction");

    for input_pair in &input_pairs[1..] {
        problem_tree.insert_root_sibling_input_pair(input_pair, Direction::Right);
        problem_tree.reduce().expect("Failed to perform reduction");
    }

    problem_tree.magnitude()
}

fn part2(input_pairs: &[InputPair]) -> u32 {
    input_pairs
        .iter()
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
            tree1.reduce().expect("Failed to perform reduction");

            tree1.magnitude()
        })
        .max()
        .expect("should be at least two input pairs")
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

    println!("Part 1: {}", part1(&all_pairs));
    println!("Part 2: {}", part2(&all_pairs));
}
