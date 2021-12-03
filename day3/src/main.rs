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

fn calculate_rate_from_bit_arr(bits: &[u8]) -> u32 {
    bits.iter()
        .rev()
        .enumerate()
        .fold(0_u32, |total, (i, &bit)| {
            total + ((bit as u32) * 2_u32.pow(i as u32))
        })
}

fn update_count_vec_for_input_line(
    bit_counts: &mut Vec<BitCounts>,
    input_line: &str,
) -> Result<(), Error> {
    input_line.chars().enumerate().try_for_each(|(i, c)| {
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

fn part1(input_lines: &[String]) -> u32 {
    let bit_counts = input_lines
        .iter()
        .try_fold(
            Vec::<BitCounts>::new(),
            |mut counts, line| -> Result<_, Error> {
                update_count_vec_for_input_line(&mut counts, line)?;
                Ok(counts)
            },
        )
        .expect("Failed to count bits");

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

fn calculate_rating<F>(input_lines: &[String], get_bit: F) -> u32
where
    F: Fn(MinMax<u8>) -> u8,
{
    let mut remaining_values = input_lines
        .iter()
        .map(|s| s.as_str())
        .collect::<Vec<&str>>();

    for i in 0..input_lines[0].len() {
        // Puzzle states that this is the halting point
        if remaining_values.len() == 1 {
            break;
        }

        let remaining_bit_counts = remaining_values
            .iter()
            .try_fold(
                Vec::<BitCounts>::new(),
                |mut counts, line| -> Result<_, Error> {
                    update_count_vec_for_input_line(&mut counts, line)?;
                    Ok(counts)
                },
            )
            .expect("Failed to count bits");

        let counts = &remaining_bit_counts[i];
        let most_common_bit = if counts.0 > counts.1 { 0_u8 } else { 1_u8 };
        let least_common_bit = if counts.0 <= counts.1 { 0_u8 } else { 1_u8 };
        remaining_values = remaining_values
            .into_iter()
            .filter(|val| {
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
    calculate_rate_from_string(remaining_values[0])
}

fn part2(input_lines: &[String]) -> u32 {
    let oxygen_rating = calculate_rating(
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
    );

    let co2_rating = calculate_rating(
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
    );

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
