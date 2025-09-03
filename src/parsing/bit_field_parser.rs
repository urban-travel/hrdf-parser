/// # Fitfield parsing
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
/// ## Example (excerpt) – Hex insetead of bits:
///
/// ...
/// 000017 FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFE00 % Bitfield 17 representing the Number FF...00
/// ...
///
///
/// 1 file(s).
/// File(s) read by the parser:
/// BITFELD
use std::{
    error::Error,
    fs::File,
    io::{self, Read, Seek},
};

use nom::{
    IResult, Parser,
    bytes::{take, take_while_m_n},
    character::{
        complete::{hex_digit1, space1},
        digit1, one_of,
    },
    combinator::{map, map_res},
    multi::count,
    number::be_i32,
    sequence::separated_pair,
};
use rustc_hash::FxHashMap;

use crate::{
    models::{BitField, Model},
    parsing::{ColumnDefinition, ExpectedType, FileParser, ParsedValue, RowDefinition, RowParser},
    storage::ResourceStorage,
};

fn bitfield_row_parser() -> RowParser {
    RowParser::new(vec![
        // This row is used to create a BitField instance.
        RowDefinition::from(vec![
            ColumnDefinition::new(1, 6, ExpectedType::Integer32),
            ColumnDefinition::new(8, 103, ExpectedType::String),
        ]),
    ])
}

fn to_string(v: Vec<char>) -> String {
    v.iter().collect::<String>()
}

fn parse_bitfield_row(input: &str) -> IResult<&str, (i32, Vec<u8>)> {
    separated_pair(
        map_res(count(one_of("0123456789"), 6), |chars| {
            to_string(chars).parse::<i32>()
        }),
        space1,
        map_res(count(one_of("0123456789ABCDEF"), 96), |x| {
            convert_hex_number_to_bits(&to_string(x))
        }),
    )
    .parse(input)
}

fn read_lines(path: &str, bytes_offset: u64) -> io::Result<Vec<String>> {
    let mut file = File::open(path)?;
    file.seek(io::SeekFrom::Start(bytes_offset))?;
    let mut reader = io::BufReader::new(file);
    let mut contents = String::new();
    reader.read_to_string(&mut contents)?;
    let lines = contents.lines().map(String::from).collect();
    Ok(lines)
}

fn bitfield_row_converter(parser: FileParser) -> Result<FxHashMap<i32, BitField>, Box<dyn Error>> {
    let data = parser
        .parse()
        .map(|x| x.and_then(|(_, _, values)| create_instance(values)))
        .collect::<Result<Vec<_>, _>>()?;
    let data = BitField::vec_to_map(data);
    Ok(data)
}

pub fn parse(path: &str) -> Result<ResourceStorage<BitField>, Box<dyn Error>> {
    log::info!("Parsing BITFELD...");
    let lines = read_lines(&format!("{path}/BITFELD"), 0)?;
    let bitfields = lines
        .into_iter()
        .filter(|line| !line.trim().is_empty())
        .map(|line| {
            let (_, (id, bits)) = parse_bitfield_row(&line)
                .map_err(|e| format!("Failed to parse line '{}': {}", line, e))?;
            Ok(BitField::new(id, bits))
        })
        .collect::<Result<Vec<_>, Box<dyn Error>>>()?;
    let data = BitField::vec_to_map(bitfields);
    Ok(ResourceStorage::new(data))
}

// ------------------------------------------------------------------------------------------------
// --- Data Processing Functions
// ------------------------------------------------------------------------------------------------

fn row_from_parsed_values(mut values: Vec<ParsedValue>) -> (i32, String) {
    let id: i32 = values.remove(0).into();
    let hex_number: String = values.remove(0).into();
    (id, hex_number)
}

fn create_instance(values: Vec<ParsedValue>) -> Result<BitField, Box<dyn Error>> {
    let (id, hex_number) = row_from_parsed_values(values);

    let bits = convert_hex_number_to_bits(&hex_number)?;

    Ok(BitField::new(id, bits))
}

// ------------------------------------------------------------------------------------------------
// --- Helper Functions
// ------------------------------------------------------------------------------------------------

/// Converts a hexadecimal number into a list of where each item represents a bit.
fn convert_hex_number_to_bits(hex_number: &str) -> Result<Vec<u8>, Box<dyn Error>> {
    let result = hex_number
        .chars()
        .map(|hex_digit| {
            hex_digit
                .to_digit(16)
                .ok_or("Invalid hexadecimal digit")
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
    use crate::parsing::tests::get_json_values;
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
    fn row_parser_v207() {
        let rows = vec![
            "000017 FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF0000".to_string(),
            "425152 FFFFFFFFEFFFFFFFFFFBF7EBD7BF5FFFBFFFFFFFEFBFDFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF0000".to_string()
        ];
        let parser = FileParser {
            row_parser: bitfield_row_parser(),
            rows,
        };
        let mut parser_iterator = parser.parse();
        let (_, _, parsed_values) = parser_iterator.next().unwrap().unwrap();
        let (id, hex_number) = row_from_parsed_values(parsed_values);
        assert_eq!(17, id);
        assert_eq!(
            "FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF0000",
            &hex_number
        );
        let (_, _, parsed_values) = parser_iterator.next().unwrap().unwrap();
        let (id, hex_number) = row_from_parsed_values(parsed_values);
        assert_eq!(425152, id);
        assert_eq!(
            "FFFFFFFFEFFFFFFFFFFBF7EBD7BF5FFFBFFFFFFFEFBFDFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF0000",
            &hex_number
        );
    }

    #[test]
    fn type_converter_v207() {
        let rows = vec![
            "000017 FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF0000".to_string(),
            "425152 FFFFFFFFEFFFFFFFFFFBF7EBD7BF5FFFBFFFFFFFEFBFDFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF0000".to_string()
        ];
        let parser = FileParser {
            row_parser: bitfield_row_parser(),
            rows,
        };
        let data = bitfield_row_converter(parser).unwrap();
        // First row (id: 1)
        let attribute = data.get(&17).unwrap();
        let reference = r#"
            {
                "id": 17,
                "bits": [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]
            }"#;
        let (attribute, reference) = get_json_values(attribute, reference).unwrap();
        assert_eq!(attribute, reference);
        // Second row (id: 2)
        let attribute = data.get(&425152).unwrap();
        let reference = r#"
            {
                "id": 425152,
                "bits": [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 1, 1, 1, 1, 1, 1, 0, 1, 1, 1, 1, 1, 1, 0, 1, 0, 1, 1, 1, 1, 0, 1, 0, 1, 1, 1, 1, 0, 1, 1, 1, 1, 1, 1, 0, 1, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 1, 1, 1, 1, 1, 0, 1, 1, 1, 1, 1, 1, 1, 1, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]
            }"#;
        let (attribute, reference) = get_json_values(attribute, reference).unwrap();
        assert_eq!(attribute, reference);
    }
}
