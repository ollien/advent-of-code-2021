#![warn(clippy::all, clippy::pedantic)]
use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::ops::{Add, AddAssign};
use thiserror::Error;

#[derive(Debug, Error)]
enum Error {
    #[error("Unexpected char '{0}'")]
    InvalidChar(char),
}

struct MinMax<T> {
    min: T,
    max: T,
}

#[derive(Debug)]
struct BitCounts(u32, u32);

impl TryFrom<char> for BitCounts {
    type Error = Error;
    fn try_from(c: char) -> Result<Self, Self::Error> {
        match c {
            '0' => Ok(Self(1, 0)),
            '1' => Ok(Self(0, 1)),
            _ => Err(Error::InvalidChar(c)),
        }
    }
}

impl Add<BitCounts> for BitCounts {
    type Output = BitCounts;
    fn add(self, rhs: BitCounts) -> Self::Output {
        Self(self.0 + rhs.0, self.1 + rhs.1)
    }
}

// Just doing AddAssign for &mut BitCounts is a bit improper, but I don't care enough just for advent of code
impl AddAssign<BitCounts> for &mut BitCounts {
    fn add_assign(&mut self, rhs: BitCounts) {
        self.0 += rhs.0;
        self.1 += rhs.1;
    }
}

fn calculate_rate_from_string(bit_string: &str) -> u32 {
    let bit_arr = bit_string
        .chars()
        .map(|c| match c {
            '0' => 0,
            '1' => 1,
            _ => panic!("Invalid binary string"),
        })
        .collect::<Vec<u8>>();

    calculate_rate_from_bit_arr(&bit_arr)
}

/// Calculate a rate for the puzzle output
fn calculate_rate_from_bit_arr(bits: &[u8]) -> u32 {
    #[allow(clippy::cast_possible_truncation)]
    bits.iter()
        .rev()
        .enumerate()
        .fold(0_u32, |total, (i, &bit)| {
            // This cast is technically a truncation but I know there's a small enough number of elements
            total + (u32::from(bit) * 2_u32.pow(i as u32))
        })
}

fn update_bit_count_vec_for_bit_string(
    bit_counts: &mut Vec<BitCounts>,
    bit_string: &str,
) -> Result<(), Error> {
    bit_string.chars().enumerate().try_for_each(|(i, c)| {
        match bit_counts.get_mut(i) {
            None => {
                let element = BitCounts::try_from(c)?;
                bit_counts.push(element);
            }
            Some(mut current_count) => {
                current_count += BitCounts::try_from(c)?;
            }
        }
        Ok(())
    })?;

    Ok(())
}

/// Count the number of bits in each position for every bit string. The return value is the number of zeroes and ones
/// in each position
fn count_bits<S: AsRef<str>>(bit_strings: &[S]) -> Result<Vec<BitCounts>, Error> {
    bit_strings.iter().try_fold(
        Vec::<BitCounts>::new(),
        |mut counts, line| -> Result<_, Error> {
            update_bit_count_vec_for_bit_string(&mut counts, line.as_ref())?;
            Ok(counts)
        },
    )
}

fn part1(input_lines: &[String]) -> u32 {
    let bit_counts = count_bits(input_lines).expect("Failed to count bits");

    let most_common_bits = bit_counts
        .iter()
        .map(|counts| if counts.0 > counts.1 { 0_u8 } else { 1_u8 })
        .collect::<Vec<u8>>();

    let least_common_bits = bit_counts
        .iter()
        .map(|counts| if counts.0 <= counts.1 { 0_u8 } else { 1_u8 })
        .collect::<Vec<u8>>();

    let gamma_rate = calculate_rate_from_bit_arr(&most_common_bits);
    let epsilon_rate = calculate_rate_from_bit_arr(&least_common_bits);

    gamma_rate * epsilon_rate
}

/// Calculate a rating (part 2), using the bit returned by `get_bit` to determine if an element should be discarded
fn calculate_part2_rating<F>(input_lines: &[String], get_bit: F) -> Result<u32, Error>
where
    F: Fn(MinMax<u8>) -> u8,
{
    let mut remaining_values = input_lines
        .iter()
        .map(String::as_str)
        .collect::<Vec<&str>>();

    for i in 0..input_lines[0].len() {
        // Puzzle states that this is the halting point
        if remaining_values.len() == 1 {
            break;
        }

        let remaining_bit_counts = count_bits(&remaining_values)?;
        let counts = &remaining_bit_counts[i];
        let most_common_bit = if counts.0 > counts.1 { 0_u8 } else { 1_u8 };
        let least_common_bit = if counts.0 <= counts.1 { 0_u8 } else { 1_u8 };

        remaining_values = remaining_values
            .into_iter()
            .filter(|val| {
                // We know these will be zero or one... you can't truncate here
                #[allow(clippy::cast_possible_truncation)]
                let bit_at_position: u8 = val
                    .chars()
                    .nth(i)
                    // If either of these fail, our counter must have hit a serious problem
                    .expect("number of bits is different than counted number of bits")
                    .to_digit(2)
                    .expect("binary number did not have binary digit")
                    as u8;

                let bit_to_compare = get_bit(MinMax {
                    min: least_common_bit,
                    max: most_common_bit,
                });

                bit_at_position == bit_to_compare
            })
            .collect::<Vec<&str>>();
    }

    // Stated by puzzle
    assert!(
        remaining_values.len() == 1,
        "Only one item should remain: {:?}",
        &remaining_values
    );

    Ok(calculate_rate_from_string(remaining_values[0]))
}

fn part2(input_lines: &[String]) -> u32 {
    let oxygen_rating = calculate_part2_rating(
        input_lines,
        |MinMax {
             min: least_common,
             max: most_common,
         }| {
            if most_common == least_common {
                1
            } else {
                most_common
            }
        },
    )
    .expect("Failed to calculate oxygen rating");

    let co2_rating = calculate_part2_rating(
        input_lines,
        |MinMax {
             min: least_common,
             max: most_common,
         }| {
            if most_common == least_common {
                0
            } else {
                least_common
            }
        },
    )
    .expect("Failed to calculate CO2 rating");

    oxygen_rating * co2_rating
}

fn main() {
    let input_file_name = env::args().nth(1).expect("No input filename specified");
    let input_file = File::open(input_file_name).expect("Could not open input file");

    // vec of the count of each bit, by position
    let input_lines = BufReader::new(input_file)
        .lines()
        .map(|res| res.expect("Failed to read line"))
        .collect::<Vec<String>>();

    println!("Part 1: {}", part1(&input_lines));
    println!("Part 2: {}", part2(&input_lines));
}
