#![warn(clippy::all, clippy::pedantic)]
use nom::{
    bits,
    combinator::eof,
    multi::{many0, many_m_n},
    sequence::{preceded, terminated, tuple},
    ErrorConvert, IResult,
};
use std::env;
use std::fs;
use std::iter;
use std::num::ParseIntError;

const TYPE_ID_SIZE: usize = 3;
const VERSION_SIZE: usize = 3;
const LITERAL_GROUP_SIZE: usize = 4;
const LENGTH_MODE_TAG: u8 = 0;
const NUMBER_OF_SUBPACKETS_MODE_TAG: u8 = 1;

const LITERAL_TYPE_ID: u8 = 4;
const SUM_TYPE_ID: u8 = 0;
const PRODUCT_TYPE_ID: u8 = 1;
const MINIMUM_TYPE_ID: u8 = 2;
const MAXIMUM_TYPE_ID: u8 = 3;
const GREATER_THAN_TYPE_ID: u8 = 5;
const LESS_THAN_TYPE_ID: u8 = 6;
const EQUAL_TO_TYPE_ID: u8 = 7;

#[derive(Debug, Clone)]
enum PacketParseErrorKind {
    Nom(nom::error::ErrorKind),
    SubpacketLengthTooLong(usize),
}

// We have fields here that are good error info, but not used otherwise
#[allow(dead_code)]
#[derive(Debug, Clone)]
struct PacketParseError {
    data: (Vec<u8>, usize),
    kind: PacketParseErrorKind,
    next: Box<Option<PacketParseError>>,
}

#[derive(Debug, Clone)]
enum Data {
    // Literals can be aribrary length but probably won't be more than a u64...
    Literal(u64),
    Operator {
        type_id: u8,
        sub_packets: Vec<Packet>,
    },
}

#[derive(Debug, Clone)]
struct Packet {
    version: u8,
    data: Data,
}

#[derive(Debug, Clone, Copy)]
struct RawPacketHeader {
    version: u8,
    type_id: u8,
}

impl PacketParseError {
    fn from_bits_error(input: (&[u8], usize), kind: PacketParseErrorKind) -> Self {
        let copied_input = input.0.iter().copied().collect();
        Self {
            data: (copied_input, input.1),
            kind,
            next: Box::new(None),
        }
    }
}

impl nom::error::ParseError<(&[u8], usize)> for PacketParseError {
    fn from_error_kind(input: (&[u8], usize), kind: nom::error::ErrorKind) -> Self {
        Self::from_bits_error(input, PacketParseErrorKind::Nom(kind))
    }

    fn append(input: (&[u8], usize), kind: nom::error::ErrorKind, other: Self) -> Self {
        let mut err = Self::from_error_kind(input, kind);
        err.next = Box::new(Some(other));

        err
    }
}

impl nom::error::ParseError<&[u8]> for PacketParseError {
    fn from_error_kind(input: &[u8], kind: nom::error::ErrorKind) -> Self {
        Self::from_bits_error((input, 0), PacketParseErrorKind::Nom(kind))
    }

    fn append(input: &[u8], kind: nom::error::ErrorKind, other: Self) -> Self {
        let mut err = Self::from_error_kind(input, kind);
        err.next = Box::new(Some(other));

        err
    }
}

// This is such a stupid hack, but Nom needs the ability to call ErrorConvert from one type to another
// when going from bits to bytes. This satisfies that interface
impl ErrorConvert<PacketParseError> for PacketParseError {
    fn convert(self) -> PacketParseError {
        self
    }
}

fn parse_version(data: (&[u8], usize)) -> IResult<(&[u8], usize), u8, PacketParseError> {
    bits::complete::take(VERSION_SIZE)(data)
}

fn parse_type_id(data: (&[u8], usize)) -> IResult<(&[u8], usize), u8, PacketParseError> {
    bits::complete::take(TYPE_ID_SIZE)(data)
}

fn parse_literal(data: (&[u8], usize)) -> IResult<(&[u8], usize), u64, PacketParseError> {
    let (remaining, (groups, last_group)) = tuple((
        many0(preceded(
            bits::complete::tag(1, 1_usize),
            bits::complete::take(LITERAL_GROUP_SIZE),
        )),
        preceded(
            bits::complete::tag(0, 1_usize),
            bits::complete::take(LITERAL_GROUP_SIZE),
        ),
    ))(data)?;

    let literal = groups
        .into_iter()
        .chain(iter::once(last_group))
        .fold(0_u64, |total, group: u8| {
            (total << LITERAL_GROUP_SIZE) | u64::from(group)
        });

    Ok((remaining, literal))
}

fn parse_operator_data(
    data: (&[u8], usize),
) -> IResult<(&[u8], usize), Vec<Packet>, PacketParseError> {
    let (remaining, length_tag) = bits::complete::take::<_, u8, _, _>(1_usize)(data)?;
    let length = if length_tag == LENGTH_MODE_TAG {
        15_usize
    } else {
        11_usize
    };

    let (after_mode_data, mode_data) = bits::complete::take(length)(remaining)?;
    if length_tag == NUMBER_OF_SUBPACKETS_MODE_TAG {
        many_m_n(mode_data, mode_data, parse_packet)(after_mode_data)
    } else {
        let mut length_remaining = mode_data;
        let mut after_packets = after_mode_data;
        let mut packets = vec![];
        while length_remaining > 0 {
            let (packet_remaining, packet) = parse_packet(after_packets)?;
            let length_left_after_packet = packet_remaining.0.len() * 8 - packet_remaining.1;
            let length_left_after_old_after = after_packets.0.len() * 8 - after_packets.1;
            let length_read = length_left_after_old_after - length_left_after_packet;
            if length_read > length_remaining {
                let copied_input = after_packets.0.iter().copied().collect();
                let err = PacketParseError {
                    data: (copied_input, after_packets.1),
                    kind: PacketParseErrorKind::SubpacketLengthTooLong(length_read),
                    next: Box::new(None),
                };

                return Err(nom::Err::Error(err));
            }

            packets.push(packet);
            after_packets = packet_remaining;
            length_remaining -= length_read;
        }

        Ok((after_packets, packets))
    }
}

fn parse_header_components(
    data: (&[u8], usize),
) -> IResult<(&[u8], usize), RawPacketHeader, PacketParseError> {
    let (after_header, (version, type_id)) = tuple((parse_version, parse_type_id))(data)?;

    let header = RawPacketHeader { version, type_id };

    Ok((after_header, header))
}

fn parse_packet(data: (&[u8], usize)) -> IResult<(&[u8], usize), Packet, PacketParseError> {
    let (after_header, header) = parse_header_components(data)?;
    let (after_data, packet_data) = if header.type_id == LITERAL_TYPE_ID {
        let (remaining, literal) = parse_literal(after_header)?;
        (remaining, Data::Literal(literal))
    } else {
        let (remaining, sub_packets) = parse_operator_data(after_header)?;
        (
            remaining,
            Data::Operator {
                type_id: header.type_id,
                sub_packets,
            },
        )
    };

    let packet = Packet {
        version: header.version,
        data: packet_data,
    };

    Ok((after_data, packet))
}

/// Parse the root level packet
fn parse_packet_stream(data: &[u8]) -> IResult<&[u8], Packet, PacketParseError> {
    terminated(bits(parse_packet), eof)(data)
}

/// Converts the input string (which is hex) to bytes we can process
fn convert_input_to_bytes(input: &str) -> Result<Vec<u8>, ParseIntError> {
    // https://stackoverflow.com/a/52992629
    (0..input.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&input[i..=i + 1], 16))
        .collect()
}

fn part1(packet: &Packet) -> u32 {
    let packet_version = u32::from(packet.version);
    let subpacket_total = match &packet.data {
        Data::Literal(_) => 0,
        Data::Operator {
            type_id: _,
            sub_packets,
        } => sub_packets.iter().map(part1).sum(),
    };

    subpacket_total + packet_version
}

fn part2(packet: &Packet) -> u64 {
    let evalute_operator = |type_id, sub_packets: &[Packet]| -> u64 {
        let evalutated_subpacket_iter = sub_packets.iter().map(part2);
        match type_id {
            SUM_TYPE_ID => evalutated_subpacket_iter.sum(),
            PRODUCT_TYPE_ID => evalutated_subpacket_iter.product(),
            MINIMUM_TYPE_ID => evalutated_subpacket_iter.min().expect("Puzzle guarantees minimum packets will have at least one element, but the opposite was encountered"),
            MAXIMUM_TYPE_ID => evalutated_subpacket_iter.max().expect("Puzzle guarantees maximum packets will have at least one element, but the opposite was encountered"),
            EQUAL_TO_TYPE_ID | GREATER_THAN_TYPE_ID | LESS_THAN_TYPE_ID  => {
                let operations = evalutated_subpacket_iter.collect::<Vec<_>>();
                assert_eq!(operations.len(), 2, "Puzzle guarantees comparison operations will have two subpackets, but we encountered one with {}", operations.len());
                let comparison_res = match type_id {
                    EQUAL_TO_TYPE_ID => operations[0] == operations[1],
                    GREATER_THAN_TYPE_ID => operations[0] > operations[1],
                    LESS_THAN_TYPE_ID => operations[0] < operations[1],
                    _ => panic!("somehow matched that the type id was a comparison operator, but did not encounter one"),
                };

                if comparison_res { 1 } else { 0 }
            }
            _ => panic!("Unexpected operator id {}", type_id),
        }
    };

    match &packet.data {
        &Data::Literal(n) => n,
        Data::Operator {
            type_id,
            sub_packets,
        } => evalute_operator(*type_id, sub_packets),
    }
}

fn main() {
    let input_file_name = env::args().nth(1).expect("No input filename specified");
    let full_input = fs::read_to_string(input_file_name).expect("Could not read input file");
    let input = full_input.trim();
    let input_bytes = convert_input_to_bytes(input).expect("Could not convert input to bytes");
    let (remaining, input_packet) =
        parse_packet_stream(&input_bytes).expect("Failed to parse input");
    assert!(
        remaining.is_empty(),
        "parser was supposed to guarantee we parsed the full input, but it did not"
    );

    println!("Part 1: {}", part1(&input_packet));
    println!("Part 2: {}", part2(&input_packet));
}
