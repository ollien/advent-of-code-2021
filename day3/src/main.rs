#![warn(clippy::all, clippy::pedantic)]
use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::ops::AddAssign;
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

// Just doing AddAssign for &mut BitCounts (hell, not even implementing Add) is a bit improper (we should implement
// all permutations), but I don't care enough just for this problem.
impl AddAssign<BitCounts> for &mut BitCounts {
    fn add_assign(&mut self, rhs: BitCounts) {
        self.0 += rhs.0;
        self.1 += rhs.1;
    }
}

impl BitCounts {
    /// Get the more common bit of the two
    fn more_common_bit(&self) -> u8 {
        if self.0 > self.1 {
            0_u8
        } else {
            1_u8
        }
    }

    /// Get the less common bit of the two
    fn less_common_bit(&self) -> u8 {
        if self.more_common_bit() == 0_u8 {
            1_u8
        } else {
            0_u8
        }
    }
}

/// Calculate a rate for the puzzle output
fn calculate_rate(bits: &[u8]) -> u32 {
    #[allow(clippy::cast_possible_truncation)]
    bits.iter()
        .rev()
        .enumerate()
        .fold(0_u32, |total, (i, &bit)| {
            // This cast is technically a truncation but I know there's a small enough number of elements
            total + (u32::from(bit) * 2_u32.pow(i as u32))
        })
}

/// Convert a string to a Vec of bits
fn string_to_bit_vec(bit_string: &str) -> Vec<u8> {
    bit_string
        .chars()
        .map(|c| match c {
            '0' => 0,
            '1' => 1,
            _ => panic!("Invalid binary string"),
        })
        .collect()
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
        .map(BitCounts::more_common_bit)
        .collect::<Vec<u8>>();

    let least_common_bits = bit_counts
        .iter()
        .map(BitCounts::less_common_bit)
        .collect::<Vec<u8>>();

    let gamma_rate = calculate_rate(&most_common_bits);
    let epsilon_rate = calculate_rate(&least_common_bits);

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

    for bit_index in 0..input_lines[0].len() {
        // Puzzle states that this is the halting point
        if remaining_values.len() == 1 {
            break;
        }

        let remaining_bit_counts = count_bits(&remaining_values)?;
        let count_at_bit_index = &remaining_bit_counts[bit_index];

        remaining_values.retain(|val| {
            // We know these will be zero or one... you can't truncate here
            #[allow(clippy::cast_possible_truncation)]
            let bit_at_position: u8 =
                val.chars()
                    .nth(bit_index)
                    // If either of these fail, our counter must have hit a serious problem
                    .expect("number of bits is different than counted number of bits")
                    .to_digit(2)
                    .expect("binary number did not have binary digit") as u8;

            let bit_to_compare = get_bit(MinMax {
                min: count_at_bit_index.less_common_bit(),
                max: count_at_bit_index.more_common_bit(),
            });

            bit_at_position == bit_to_compare
        });
    }

    // Stated by puzzle
    assert!(
        remaining_values.len() == 1,
        "Only one item should remain: {:?}",
        &remaining_values
    );

    // Yes, we allocate here unnecessarily (we could just count in the string chars directly), but _shrug_.
    // I didn't want to duplicate the logic just to avoid it for such a simple problem
    let bit_vec = string_to_bit_vec(remaining_values[0]);
    Ok(calculate_rate(&bit_vec))
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
