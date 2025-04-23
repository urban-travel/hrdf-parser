// 1 file(s).
// File(s) read by the parser:
// RICHTUNG
/**
 * This file contains direction informations: namely the Direction ID (that is also contained in
 * the FPLAN file) and the direction Text which gives the last stop of the traject. From
 * [https://opentransportdata.swiss/en/cookbook/hafas-rohdaten-format-hrdf/#Technical_description_What_is_in_the_HRDF_files_contents](HRDF the docs) we have:
 *
 * `R000011 Esslingen    % Richtung 11 nach Esslingen`
 *
 * that the direction 11 (R is for Richtung) travels to Esslingen
 */
use std::error::Error;

use rustc_hash::FxHashMap;

use crate::{
    models::{Direction, Model},
    parsing::{ColumnDefinition, ExpectedType, FileParser, ParsedValue, RowDefinition, RowParser},
    storage::ResourceStorage,
};

type DirectionAndTypeConverter = (ResourceStorage<Direction>, FxHashMap<String, i32>);

fn direction_row_parser() -> RowParser {
    RowParser::new(vec![
        // This row is used to create a Direction instance.
        RowDefinition::from(vec![
            ColumnDefinition::new(1, 7, ExpectedType::String),
            ColumnDefinition::new(9, -1, ExpectedType::String),
        ]),
    ])
}

pub fn parse(path: &str) -> Result<DirectionAndTypeConverter, Box<dyn Error>> {
    log::info!("Parsing RICHTUNG...");
    let row_parser = direction_row_parser();
    let parser = FileParser::new(&format!("{path}/RICHTUNG"), row_parser)?;

    let mut pk_type_converter = FxHashMap::default();

    let data = parser
        .parse()
        .map(|x| x.and_then(|(_, _, values)| create_instance(values, &mut pk_type_converter)))
        .collect::<Result<Vec<_>, _>>()?;
    let data = Direction::vec_to_map(data);

    Ok((ResourceStorage::new(data), pk_type_converter))
}

// ------------------------------------------------------------------------------------------------
// --- Data Processing Functions
// ------------------------------------------------------------------------------------------------

fn create_instance(
    mut values: Vec<ParsedValue>,
    pk_type_converter: &mut FxHashMap<String, i32>,
) -> Result<Direction, Box<dyn Error>> {
    let legacy_id: String = values.remove(0).into();
    let name: String = values.remove(0).into();

    let id = remove_first_char(&legacy_id).parse::<i32>()?;

    pk_type_converter.insert(legacy_id, id);
    Ok(Direction::new(id, name))
}

// ------------------------------------------------------------------------------------------------
// --- Helper Functions
// ------------------------------------------------------------------------------------------------

fn remove_first_char(value: &str) -> &str {
    let mut chars = value.chars();
    chars.next();
    chars.as_str()
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;
    use crate::parsing::tests::get_json_values;
    use pretty_assertions::assert_eq;

    #[test]
    fn parser_row_d_v207() {
        let rows = vec!["R000008 Winterthur".to_string()];
        let parser = FileParser {
            row_parser: direction_row_parser(),
            rows,
        };
        let mut parser_iterator = parser.parse();
        // let (id, _, mut parsed_values) = parser_iterator.next().unwrap().unwrap();
        // let legacy_id: String = parsed_values.remove(0).into();
        // assert_eq!("VR", &legacy_id);
        // let description: String = parsed_values.remove(0).into();
        // assert_eq!("VELOS: Reservation obligatory", &description);
    }
}
