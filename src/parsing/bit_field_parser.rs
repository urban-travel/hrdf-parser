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
use rustc_hash::FxHashMap;

use crate::{
    Result,
    error::ErrorKind,
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

fn bitfield_row_converter(parser: FileParser) -> Result<FxHashMap<i32, BitField>> {
    let data = parser
        .parse()
        .map(|x| x.and_then(|(_, _, values)| create_instance(values)))
        .collect::<Result<Vec<_>>>()?;
    let data = BitField::vec_to_map(data);
    Ok(data)
}

pub fn parse(path: &str) -> Result<ResourceStorage<BitField>> {
    log::info!("Parsing BITFELD...");
    #[rustfmt::skip]
    let row_parser = bitfield_row_parser();
    let parser = FileParser::new(&format!("{path}/BITFELD"), row_parser)?;

    let data = bitfield_row_converter(parser)?;

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

fn create_instance(values: Vec<ParsedValue>) -> Result<BitField> {
    let (id, hex_number) = row_from_parsed_values(values);

    let bits = convert_hex_number_to_bits(hex_number)?;

    Ok(BitField::new(id, bits))
}

// ------------------------------------------------------------------------------------------------
// --- Helper Functions
// ------------------------------------------------------------------------------------------------

/// Converts a hexadecimal number into a list of where each item represents a bit.
fn convert_hex_number_to_bits(hex_number: String) -> Result<Vec<u8>> {
    let result = hex_number
        .chars()
        .map(|hex_digit| {
            hex_digit
                .to_digit(16)
                .ok_or(ErrorKind::InvalidHexaDigit.into())
                .map(|val| (0..4).rev().map(move |i| ((val >> i) & 1) as u8))
        })
        .collect::<Result<Vec<_>>>()?
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
