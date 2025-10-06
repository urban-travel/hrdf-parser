/// # Bitfield parsing
///
/// Day-specific definition of the validity of the timetable information.
/// The validity is defined using a bit pattern. Each day is represented by a bit.
/// In each case, 4 bits are combined to form a hexadecimal digit.
///
/// To interpret the bit field, it is important to understand how the timetable
/// years work. A timetable year usually lasts one year,
/// but the timetable year begins on the 2nd weekend in December.
/// **This means that a timetable year is not always the same length. To deal with this, the bit field is assumed to be 400 days long.**
///
/// The file contains:
///
/// - The Bitfield code
/// - The Bitfield definition
///
/// ## Example (excerpt) – Hex instead of bits:
///
/// ...
/// 000017 FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFE00 % Bitfield 17 representing the Number FF...00
/// ...
///
///
/// 1 file(s).
/// File(s) read by the parser:
/// BITFELD
use nom::{
    IResult, Parser,
    character::{char, one_of},
    combinator::map_res,
    multi::count,
    sequence::separated_pair,
};
use rustc_hash::FxHashMap;

use crate::{
    models::BitField,
    parsing::{
        error::{HResult, HrdfError, PResult, ParsingError},
        helpers::{i32_from_n_digits_parser, read_lines},
    },
    storage::ResourceStorage,
};

fn to_string(v: Vec<char>) -> String {
    v.iter().collect::<String>()
}

fn parse_bitfield_row(input: &str) -> IResult<&str, (i32, Vec<u8>)> {
    separated_pair(
        i32_from_n_digits_parser(6),
        char(' '),
        map_res(count(one_of("0123456789ABCDEF"), 96), |x| {
            convert_hex_number_to_bits(&to_string(x))
        }),
    )
    .parse(input)
}

// TODO: Add test for parse line
fn parse_line(line: &str) -> PResult<(i32, BitField)> {
    let (_, (id, bits)) = parse_bitfield_row(line)?;
    Ok((id, BitField::new(id, bits)))
}

pub fn parse(path: &str) -> HResult<ResourceStorage<BitField>> {
    log::info!("Parsing BITFELD...");
    let file = format!("{path}/BITFELD");
    let lines = read_lines(&file, 0)?;
    let bitfields = lines
        .into_iter()
        .enumerate()
        .filter(|(_, line)| !line.trim().is_empty())
        .map(|(line_number, line)| {
            parse_line(&line).map_err(|e| HrdfError::Parsing {
                error: e,
                file: String::from(&file),
                line,
                line_number,
            })
        })
        .collect::<HResult<FxHashMap<i32, BitField>>>()?;
    Ok(ResourceStorage::new(bitfields))
}

// ------------------------------------------------------------------------------------------------
// --- Helper Functions
// ------------------------------------------------------------------------------------------------

/// Converts a hexadecimal number into a list of where each item represents a bit.
fn convert_hex_number_to_bits(hex_number: &str) -> PResult<Vec<u8>> {
    let result = hex_number
        .chars()
        .map(|hex_digit| {
            hex_digit
                .to_digit(16)
                .ok_or(ParsingError::InvalidHexDigit(hex_digit))
                .map(|val| (0..4).rev().map(move |i| ((val >> i) & 1) as u8))
        })
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .flatten()
        .collect::<Vec<_>>();
    Ok(result)
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn successful_bitfield_row() {
        let input = "000017 FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF0000";
        let (_, (id, hex_number)) = parse_bitfield_row(input).unwrap();
        assert_eq!(17, id);
        assert_eq!(
            vec![
                1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
                1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
                1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
                1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
                1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
                1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
                1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
                1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
                1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
                1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
                1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
                1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
                1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
                1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0
            ],
            hex_number
        );
    }

    #[test]
    #[should_panic]
    fn failed_bitfield_row_num_short() {
        let input = "00017 FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF0000";
        let (_, (_, _)) = parse_bitfield_row(input).unwrap();
    }

    #[test]
    #[should_panic]
    fn failed_bitfield_row_num_long() {
        let input = "0900017 FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF0000";
        let (_, (_, _)) = parse_bitfield_row(input).unwrap();
    }

    #[test]
    #[should_panic]
    fn failed_bitfield_row_num_invalid() {
        let input = "0C0017 FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF0000";
        let (_, (_, _)) = parse_bitfield_row(input).unwrap();
    }

    #[test]
    #[should_panic]
    fn failed_bitfield_row_bitfield_short() {
        let input = "000017 FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF0000 ";
        let (_, (_, _)) = parse_bitfield_row(input).unwrap();
    }

    #[test]
    #[should_panic]
    fn failed_bitfield_row_bitfield_long() {
        let input = "000017 FAFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF0000";
        let (res, (_, _)) = parse_bitfield_row(input).unwrap();
        assert!(res.is_empty());
    }

    #[test]
    #[should_panic]
    fn failed_bitfield_row_bitfield_invalid() {
        let input = "000017 FbFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF0000";
        let (_, (_, _)) = parse_bitfield_row(input).unwrap();
    }

    #[test]
    #[should_panic]
    fn failed_bitfield_row_bitfield_invalid_spacing1() {
        let input = "000017  FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF0000";
        let (_, (_, _)) = parse_bitfield_row(input).unwrap();
    }

    #[test]
    #[should_panic]
    fn failed_bitfield_row_bitfield_invalid_spacing2() {
        let input = "000017,FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF0000";
        let (_, (_, _)) = parse_bitfield_row(input).unwrap();
    }

    #[test]
    #[should_panic]
    fn failed_bitfield_row_bitfield_invalid_spacing3() {
        let input = "000017 ,FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF0000";
        let (_, (_, _)) = parse_bitfield_row(input).unwrap();
    }
}
