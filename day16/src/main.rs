use nom::bits;
use nom::combinator::eof;
use nom::multi::many0;
use nom::multi::many1;
use nom::multi::many_m_n;
use nom::sequence::preceded;
use nom::sequence::terminated;
use nom::sequence::tuple;
use nom::IResult;
use std::env;
use std::fs;
use std::iter;
use std::num::ParseIntError;

const TYPE_ID_SIZE: usize = 3;
const VERSION_SIZE: usize = 3;
const LITERAL_GROUP_SIZE: usize = 4;
const LITERAL_TYPE_ID: u8 = 4;
const LENGTH_MODE_TAG: u8 = 0;
const NUMBER_OF_SUBPACKETS_MODE_TAG: u8 = 1;

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

fn parse_version(data: (&[u8], usize)) -> IResult<(&[u8], usize), u8> {
    bits::complete::take(VERSION_SIZE)(data)
}

fn parse_type_id(data: (&[u8], usize)) -> IResult<(&[u8], usize), u8> {
    bits::complete::take(TYPE_ID_SIZE)(data)
}

fn parse_literal(data: (&[u8], usize)) -> IResult<(&[u8], usize), u64> {
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
            (total << LITERAL_GROUP_SIZE) | (group as u64)
        });

    Ok((remaining, literal))
}

fn parse_operator_data(data: (&[u8], usize)) -> IResult<(&[u8], usize), Vec<Packet>> {
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
            // TODO: This should not be a panic; it's implicit here.
            packets.push(packet);
            let length_left_after_packet = packet_remaining.0.len() * 8 - packet_remaining.1;
            let length_left_after_old_after = after_packets.0.len() * 8 - after_packets.1;
            let length_read = length_left_after_old_after - length_left_after_packet;
            after_packets = packet_remaining;
            length_remaining -= length_read;
        }

        Ok((after_packets, packets))
    }
}

fn parse_header_components(data: (&[u8], usize)) -> IResult<(&[u8], usize), RawPacketHeader> {
    let (after_header, (version, type_id)) = tuple((parse_version, parse_type_id))(data)?;

    let header = RawPacketHeader { version, type_id };

    Ok((after_header, header))
}

fn parse_packet(data: (&[u8], usize)) -> IResult<(&[u8], usize), Packet> {
    let (after_header, header) = parse_header_components(data)?;
    let (after_data, packet_data) = match header.type_id {
        LITERAL_TYPE_ID => {
            let (remaining, literal) = parse_literal(after_header)?;
            (remaining, Data::Literal(literal))
        }
        _ => {
            let (remaining, sub_packets) = parse_operator_data(after_header)?;
            (
                remaining,
                Data::Operator {
                    type_id: header.type_id,
                    sub_packets,
                },
            )
        }
    };

    let packet = Packet {
        version: header.version,
        data: packet_data,
    };

    Ok((after_data, packet))
}

fn parse_packet_stream(data: &[u8]) -> IResult<&[u8], Vec<Packet>> {
    terminated(many1(bits(parse_packet)), eof)(data)
}

fn convert_input_to_bytes(input: &str) -> Result<Vec<u8>, ParseIntError> {
    // https://stackoverflow.com/a/52992629
    (0..input.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&input[i..=i + 1], 16))
        .collect()
}

fn part1(packets: &[Packet]) -> u32 {
    packets
        .iter()
        .map(|packet| {
            let packet_version = u32::from(packet.version);
            let subpacket_total = match &packet.data {
                Data::Literal(_) => 0,
                Data::Operator {
                    type_id: _,
                    sub_packets,
                } => part1(sub_packets),
            };

            subpacket_total + packet_version
        })
        .sum()
}

fn main() {
    let input_file_name = env::args().nth(1).expect("No input filename specified");
    let full_input = fs::read_to_string(input_file_name).expect("Could not read input file");
    let input = full_input.trim();
    let input_bytes = convert_input_to_bytes(input).expect("Could not convert input to bytes");
    let (remaining, input_packets) =
        parse_packet_stream(&input_bytes).expect("Failed to parse input");
    assert!(
        remaining.is_empty(),
        "parser was supposed to guarantee we parsed the full input, but it did not"
    );

    println!("Part 1: {}", part1(&input_packets));
}
