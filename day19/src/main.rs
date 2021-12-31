#![warn(clippy::all, clippy::pedantic)]
use itertools::Itertools;
use nom::{
    bytes::complete::tag,
    character::{complete::char, complete::digit1},
    combinator::{eof, map_res, opt, recognize},
    error::ParseError,
    multi::{many0, separated_list1},
    sequence::{pair, preceded, terminated, tuple},
    IResult, Parser,
};
use std::collections::BTreeMap;
use std::env;
use std::fmt::Debug;
use std::fs;
use std::str::FromStr;

#[derive(Debug, Clone, Copy)]
enum Axis {
    X,
    Y,
    Z,
}

#[derive(Debug, Clone, Copy)]
enum Direction {
    Negative(Axis),
    Positive(Axis),
}

impl Direction {
    fn rotate_90_degrees_about_x(self) -> Self {
        match self {
            Direction::Positive(Axis::Z) => Direction::Positive(Axis::Y),
            Direction::Positive(Axis::Y) => Direction::Negative(Axis::Z),
            Direction::Negative(Axis::Z) => Direction::Negative(Axis::Y),
            Direction::Negative(Axis::Y) => Direction::Positive(Axis::Z),
            Direction::Positive(Axis::X) => Direction::Positive(Axis::X),
            Direction::Negative(Axis::X) => Direction::Negative(Axis::X),
        }
    }

    fn rotate_90_degrees_about_y(self) -> Self {
        match self {
            Direction::Positive(Axis::X) => Direction::Positive(Axis::Z),
            Direction::Positive(Axis::Z) => Direction::Negative(Axis::X),
            Direction::Negative(Axis::X) => Direction::Negative(Axis::Z),
            Direction::Negative(Axis::Z) => Direction::Positive(Axis::X),
            Direction::Positive(Axis::Y) => Direction::Positive(Axis::Y),
            Direction::Negative(Axis::Y) => Direction::Negative(Axis::Y),
        }
    }

    fn rotate_90_degrees_about_z(self) -> Self {
        match self {
            Direction::Positive(Axis::X) => Direction::Positive(Axis::Y),
            Direction::Positive(Axis::Y) => Direction::Negative(Axis::X),
            Direction::Negative(Axis::X) => Direction::Negative(Axis::Y),
            Direction::Negative(Axis::Y) => Direction::Positive(Axis::X),
            Direction::Positive(Axis::Z) => Direction::Positive(Axis::Z),
            Direction::Negative(Axis::Z) => Direction::Negative(Axis::Z),
        }
    }
}

#[derive(Clone)]
struct Scanner {
    scanned_points: Vec<(i32, i32, i32)>,
    facing: Direction,
    up: Direction,
}

impl Scanner {
    fn new(scanned_points: Vec<(i32, i32, i32)>) -> Self {
        Self {
            scanned_points,
            facing: Direction::Positive(Axis::X),
            up: Direction::Positive(Axis::Y),
        }
    }

    fn translate(&mut self, (x, y, z): (i32, i32, i32)) {
        for (scanned_x, scanned_y, scanned_z) in &mut self.scanned_points {
            *scanned_x += x;
            *scanned_y += y;
            *scanned_z += z;
        }
    }

    // This is kind of dumb, but for the purposes about this problem is fine.
    // In a more complete program I might write one function for every possible axis,
    // we don't need that for this problem.

    fn rotate_90_degrees_about_x(&mut self) {
        self.scanned_points = self
            .scanned_points
            .iter()
            .map(|&(x, y, z)| (x, z, -y))
            .collect();

        self.facing = self.facing.rotate_90_degrees_about_x();
        self.up = self.up.rotate_90_degrees_about_x();
    }

    fn rotate_90_degrees_about_y(&mut self) {
        self.scanned_points = self
            .scanned_points
            .iter()
            .map(|&(x, y, z)| (-z, y, x))
            .collect();

        self.facing = self.facing.rotate_90_degrees_about_y();
        self.up = self.up.rotate_90_degrees_about_y();
    }

    fn rotate_90_degrees_about_z(&mut self) {
        self.scanned_points = self
            .scanned_points
            .iter()
            .map(|&(x, y, z)| (y, -x, z))
            .collect();

        self.facing = self.facing.rotate_90_degrees_about_z();
        self.up = self.up.rotate_90_degrees_about_z();
    }

    fn rotate_up_vector_90_degrees(&mut self) {
        match self.facing {
            Direction::Negative(Axis::X) | Direction::Positive(Axis::X) => {
                self.rotate_90_degrees_about_x();
            }
            Direction::Negative(Axis::Y) | Direction::Positive(Axis::Y) => {
                self.rotate_90_degrees_about_y();
            }
            Direction::Negative(Axis::Z) | Direction::Positive(Axis::Z) => {
                self.rotate_90_degrees_about_z();
            }
        }
    }

    fn generate_all_rotations(&self) -> Vec<Scanner> {
        // Could probably be an iterator, but it's tedious to convert this
        let mut res = vec![];
        let mut scanner = self.clone();
        for _ in 0..4 {
            scanner.rotate_90_degrees_about_y();
            for _ in 0..4 {
                scanner.rotate_up_vector_90_degrees();
                res.push(scanner.clone());
            }
        }

        scanner.rotate_90_degrees_about_z();
        for _ in 0..4 {
            scanner.rotate_up_vector_90_degrees();
            res.push(scanner.clone());
        }

        scanner.rotate_90_degrees_about_z();
        scanner.rotate_90_degrees_about_z();
        for _ in 0..4 {
            scanner.rotate_up_vector_90_degrees();
            res.push(scanner.clone());
        }

        res
    }
}

impl Debug for Scanner {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(formatter, "Facing: {:?}", self.facing)?;
        writeln!(formatter, "Up: {:?}", self.up)?;
        writeln!(formatter, "Scanned:")?;
        for (x, y, z) in self.scanned_points.iter().sorted() {
            writeln!(formatter, "  ({}, {}, {})", x, y, z)?;
        }

        Ok(())
    }
}

fn parse_header_line(chunk: &str) -> IResult<&str, &str> {
    recognize(tuple((tag("--- scanner "), digit1, tag(" ---\n"))))(chunk)
}

fn parse_number(chunk: &str) -> IResult<&str, i32> {
    map_res(
        pair(opt(char('-')), digit1),
        |(negative, raw_num): (_, &str)| -> Result<i32, <i32 as FromStr>::Err> {
            let factor = negative.map_or(1, |_| -1);
            let num = raw_num.parse::<i32>()?;

            Ok(factor * num)
        },
    )(chunk)
}

fn separated_triplet<I, P1, P2, P3, P4, P5, O1, O2, O3, O4, O5, E: ParseError<I>>(
    mut first: P1,
    mut sep1: P2,
    mut second: P3,
    mut sep2: P4,
    mut third: P5,
) -> impl FnMut(I) -> IResult<I, (O1, O3, O5), E>
where
    P1: Parser<I, O1, E>,
    P2: Parser<I, O2, E>,
    P3: Parser<I, O3, E>,
    P4: Parser<I, O4, E>,
    P5: Parser<I, O5, E>,
{
    move |input: I| {
        let (after_p1, output1) = first.parse(input)?;
        let (after_p2, _) = sep1.parse(after_p1)?;
        let (after_p3, output2) = second.parse(after_p2)?;
        let (after_p4, _) = sep2.parse(after_p3)?;
        let (after, output3) = third.parse(after_p4)?;

        Ok((after, (output1, output2, output3)))
    }
}

fn parse_coordinate_line(chunk: &str) -> IResult<&str, (i32, i32, i32)> {
    separated_triplet(
        parse_number,
        char(','),
        parse_number,
        char(','),
        parse_number,
    )(chunk)
}

fn parse_scanner_block(chunk: &str) -> IResult<&str, Scanner> {
    let (remaining, coords) = preceded(
        parse_header_line,
        separated_list1(char('\n'), parse_coordinate_line),
    )(chunk)?;

    let scanner = Scanner::new(coords);
    Ok((remaining, scanner))
}

fn parse_input(input: &str) -> IResult<&str, Vec<Scanner>> {
    terminated(
        separated_list1(tag("\n\n"), parse_scanner_block),
        pair(opt(many0(char('\n'))), eof),
    )(input)
}

fn get_most_common_element<I: Iterator<Item = T>, T: Ord>(iterator: I) -> Option<(T, usize)> {
    let mut map = BTreeMap::<T, usize>::new();
    for item in iterator {
        *map.entry(item).or_default() += 1;
    }

    map.into_iter()
        .max_by(|(_, count1), (_, count2)| count1.cmp(count2))
}

fn find_scanner_positions(input_scanners: &[Scanner]) -> Vec<(Scanner, (i32, i32, i32))> {
    let mut scanners = input_scanners.to_vec();

    let mut scanner_positions = BTreeMap::<usize, (i32, i32, i32)>::new();
    scanner_positions.insert(0, (0, 0, 0));

    while scanner_positions.len() < scanners.len() {
        for i in 0..scanners.len() {
            for j in 0..scanners.len() {
                if i == j
                    || scanner_positions.contains_key(&j)
                    || !scanner_positions.contains_key(&i)
                {
                    continue;
                }

                // yes this is messy but here are the elemeents
                // (the position of the scanner, the number of common positions, the rotated scanner)
                let mut max: Option<((i32, i32, i32), usize, Scanner)> = None;
                for rotated_scanner2 in scanners[j].generate_all_rotations() {
                    let differences = scanners[i].scanned_points.iter().flat_map(|(x1, y1, z1)| {
                        rotated_scanner2
                            .scanned_points
                            .iter()
                            .map(move |(x2, y2, z2)| (x1 - x2, y1 - y2, z1 - z2))
                    });
                    let most_common_candidate = get_most_common_element(differences);
                    let (value, n) = most_common_candidate.unwrap();

                    if n >= 12 && (max.is_none() || n > max.as_ref().unwrap().1) {
                        max = Some((value, n, rotated_scanner2.clone()));
                    }
                }

                if max.is_none() {
                    // hopefully we get it later..
                    continue;
                }

                let (relative_scanner_pos, _, updated_scanner2) = max.unwrap();

                // Already asserted to exist
                let (scanner1_x, scanner1_y, scanner1_z) = scanner_positions.get(&i).unwrap();
                let (scanner2_rel_x, scanner2_rel_y, scanner2_rel_z) = relative_scanner_pos;
                let scanner2_pos = (
                    scanner1_x + scanner2_rel_x,
                    scanner1_y + scanner2_rel_y,
                    scanner1_z + scanner2_rel_z,
                );

                scanner_positions.insert(j, scanner2_pos);
                scanners[j] = updated_scanner2;
            }
        }
    }

    for (i, scanner_pos) in &mut scanner_positions {
        scanners[*i].translate(*scanner_pos);
    }

    scanner_positions
        .into_iter()
        .map(|(idx, position)| (scanners[idx].clone(), position))
        .collect()
}

fn manhattan_distance((x1, y1, z1): (i32, i32, i32), (x2, y2, z2): (i32, i32, i32)) -> i32 {
    (x2 - x1).abs() + (y2 - y1).abs() + (z2 - z1).abs()
}

fn part1(scanner_positions: &[(Scanner, (i32, i32, i32))]) -> usize {
    scanner_positions
        .iter()
        .flat_map(|(scanner, _)| scanner.scanned_points.iter())
        .sorted()
        .dedup()
        .count()
}

fn part2(scanner_positions: &[(Scanner, (i32, i32, i32))]) -> i32 {
    scanner_positions
        .iter()
        .map(|&(_, position)| position)
        .permutations(2)
        .map(|permutation| manhattan_distance(permutation[0], permutation[1]))
        .max()
        .unwrap()
}

fn main() {
    let input_file_name = env::args().nth(1).expect("No input filename specified");
    let input = fs::read_to_string(input_file_name).expect("Failed to read input file");

    let (_, parsed_input) = parse_input(&input).expect("Failed to parse input");
    // We need this for both parts and it's expensive (especially if we don't optimize our compilation)
    let scanner_positions = find_scanner_positions(&parsed_input);

    println!("Part 1: {}", part1(&scanner_positions));
    println!("Part 2: {}", part2(&scanner_positions));
}
