/// # Line Exchange Time parser
///
/// For more informations see
/// [https://opentransportdata.swiss/en/cookbook/hafas-rohdaten-format-hrdf/#Technical_description_What_is_in_the_HRDF_files_contents](the HRDF documentation).
///
/// Transfer time per category of service and/or line. The file contains:
///
/// - Stop number
/// - Administration 1 (see BETRIEB / OPERATION file)
/// - Type (offer category) 1
/// - Line 1 (* = quasi-interchange times)
/// - Direction 1 (* = all directions)
/// - Administration 2 (see BETRIEB / OPERATION file),
/// - Type (offer category) 2,
/// - Line 2 (* = quasi-interchange times),
/// - Direction 2 (* = all directions),
/// - Transfer time in min.
/// - “!” for guaranteed changeover
/// - Name of stop
///
/// ## Remarks
///
/// The name of the stop is ignored here
///
/// Example (excerpt):
///
/// `
/// 1111145 sbg034 B   7322 H sbg034 TX  7322 H 000! Waldkirch (WT), Rathaus % HS-Nr 1111145, TU-Code sbg034, Angebotskategorie B, Linie 1, Richtung Hin, ...
/// 8500010 000011 EXT *    * 000011 TER *    * 010  Basel SBB               % HS-Nr 8500010, TU-Code 11, Angebotskategorie EXT, alle Linien, alle Richtungen, ...
/// `
///
/// 1 file(s).
/// File(s) read by the parser:
/// UMSTEIGL
use std::str::FromStr;

use rustc_hash::FxHashMap;

use crate::{
    Result,
    error::ErrorKind,
    models::{DirectionType, ExchangeTimeLine, LineInfo, Model},
    parsing::{ColumnDefinition, ExpectedType, FileParser, ParsedValue, RowDefinition, RowParser},
    storage::ResourceStorage,
    utils::AutoIncrement,
};

fn exchange_line_row_parser() -> RowParser {
    RowParser::new(vec![
        // This row is used to create a LineExchangeTime instance.
        RowDefinition::from(vec![
            ColumnDefinition::new(1, 7, ExpectedType::OptionInteger32),
            ColumnDefinition::new(9, 14, ExpectedType::String),
            ColumnDefinition::new(16, 18, ExpectedType::String),
            ColumnDefinition::new(20, 27, ExpectedType::String),
            ColumnDefinition::new(29, 29, ExpectedType::String),
            ColumnDefinition::new(31, 36, ExpectedType::String),
            ColumnDefinition::new(38, 40, ExpectedType::String),
            ColumnDefinition::new(42, 49, ExpectedType::String),
            ColumnDefinition::new(51, 51, ExpectedType::String),
            ColumnDefinition::new(53, 55, ExpectedType::Integer16),
            ColumnDefinition::new(56, 56, ExpectedType::String),
        ]),
    ])
}
fn exchange_line_row_converter(
    parser: FileParser,
    transport_types_pk_type_converter: &FxHashMap<String, i32>,
) -> Result<FxHashMap<i32, ExchangeTimeLine>> {
    let auto_increment = AutoIncrement::new();

    let data = parser
        .parse()
        .map(|x| {
            x.and_then(|(_, _, values)| {
                create_instance(values, &auto_increment, transport_types_pk_type_converter)
            })
        })
        .collect::<Result<Vec<_>>>()?;
    let data = ExchangeTimeLine::vec_to_map(data);
    Ok(data)
}

pub fn parse(
    path: &str,
    transport_types_pk_type_converter: &FxHashMap<String, i32>,
) -> Result<ResourceStorage<ExchangeTimeLine>> {
    log::info!("Parsing UMSTEIGL...");

    let row_parser = exchange_line_row_parser();
    let parser = FileParser::new(&format!("{path}/UMSTEIGL"), row_parser)?;
    let data = exchange_line_row_converter(parser, transport_types_pk_type_converter)?;

    Ok(ResourceStorage::new(data))
}

// ------------------------------------------------------------------------------------------------
// --- Data Processing Functions
// ------------------------------------------------------------------------------------------------

fn create_instance(
    mut values: Vec<ParsedValue>,
    auto_increment: &AutoIncrement,
    transport_types_pk_type_converter: &FxHashMap<String, i32>,
) -> Result<ExchangeTimeLine> {
    let stop_id: Option<i32> = values.remove(0).into();
    let administration_1: String = values.remove(0).into();
    let transport_type_id_1: String = values.remove(0).into();
    let line_id_1: String = values.remove(0).into();
    let direction_1: String = values.remove(0).into();
    let administration_2: String = values.remove(0).into();
    let transport_type_id_2: String = values.remove(0).into();
    let line_id_2: String = values.remove(0).into();
    let direction_2: String = values.remove(0).into();
    let duration: i16 = values.remove(0).into();
    let is_guaranteed: String = values.remove(0).into();

    let transport_type_id_1 = *transport_types_pk_type_converter
        .get(&transport_type_id_1)
        .ok_or(ErrorKind::UnknownLegacyId(
            "transport_type",
            transport_type_id_1,
        ))?;

    let line_id_1 = if line_id_1 == "*" {
        None
    } else {
        Some(line_id_1)
    };

    let direction_1 = if direction_1 == "*" {
        None
    } else {
        Some(DirectionType::from_str(&direction_1)?)
    };

    let transport_type_id_2 = *transport_types_pk_type_converter
        .get(&transport_type_id_2)
        .ok_or(ErrorKind::UnknownLegacyId(
            "transport_type",
            transport_type_id_2,
        ))?;

    let line_id_2 = if line_id_2 == "*" {
        None
    } else {
        Some(line_id_2)
    };

    let direction_2 = if direction_2 == "*" {
        None
    } else {
        Some(DirectionType::from_str(&direction_2)?)
    };

    let is_guaranteed = is_guaranteed == "!";

    let line_1 = LineInfo::new(
        administration_1,
        transport_type_id_1,
        line_id_1,
        direction_1,
    );
    let line_2 = LineInfo::new(
        administration_2,
        transport_type_id_2,
        line_id_2,
        direction_2,
    );

    Ok(ExchangeTimeLine::new(
        auto_increment.next(),
        stop_id,
        line_1,
        line_2,
        duration,
        is_guaranteed,
    ))
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
            "8301113 000011 S   *        * 007000 B   *        * 003  Luino (I)".to_string(),
            "1111135 sbg034 B   7339     H sbg034 TX  7341     H 000! Waldshut, Busbahnhof"
                .to_string(),
            "8509002 000011 RE  *        * 000065 S   12       * 008  Landquart".to_string(),
            "8580522 003849 T   #0000482 * 003849 T   #0000488 * 003  Zürich, Escher-Wyss-Platz"
                .to_string(),
        ];
        let parser = FileParser {
            row_parser: exchange_line_row_parser(),
            rows,
        };
        let mut parser_iterator = parser.parse();
        // First row
        // "8301113 000011 S   *        * 007000 B   *        * 003  Luino (I)",
        let (_, _, mut parsed_values) = parser_iterator.next().unwrap().unwrap();
        let stop_id: Option<i32> = parsed_values.remove(0).into();
        assert_eq!(Some(8301113), stop_id);
        let administration_1: String = parsed_values.remove(0).into();
        assert_eq!("000011", &administration_1);
        let transport_type_id_1: String = parsed_values.remove(0).into();
        assert_eq!("S", &transport_type_id_1);
        let line_id_1: String = parsed_values.remove(0).into();
        assert_eq!("*", &line_id_1);
        let direction_1: String = parsed_values.remove(0).into();
        assert_eq!("*", &direction_1);
        let administration_2: String = parsed_values.remove(0).into();
        assert_eq!("007000", &administration_2);
        let transport_type_id_2: String = parsed_values.remove(0).into();
        assert_eq!("B", &transport_type_id_2);
        let line_id_2: String = parsed_values.remove(0).into();
        assert_eq!("*", &line_id_2);
        let direction_2: String = parsed_values.remove(0).into();
        assert_eq!("*", &direction_2);
        let duration: i16 = parsed_values.remove(0).into();
        assert_eq!(3, duration);
        let is_guaranteed: String = parsed_values.remove(0).into();
        assert_eq!("", &is_guaranteed);
        // Second row
        // "1111135 sbg034 B   7339     H sbg034 TX  7341     H 000! Waldshut, Busbahnhof"
        let (_, _, mut parsed_values) = parser_iterator.next().unwrap().unwrap();
        let stop_id: Option<i32> = parsed_values.remove(0).into();
        assert_eq!(Some(1111135), stop_id);
        let administration_1: String = parsed_values.remove(0).into();
        assert_eq!("sbg034", &administration_1);
        let transport_type_id_1: String = parsed_values.remove(0).into();
        assert_eq!("B", &transport_type_id_1);
        let line_id_1: String = parsed_values.remove(0).into();
        assert_eq!("7339", &line_id_1);
        let direction_1: String = parsed_values.remove(0).into();
        assert_eq!("H", &direction_1);
        let administration_2: String = parsed_values.remove(0).into();
        assert_eq!("sbg034", &administration_2);
        let transport_type_id_2: String = parsed_values.remove(0).into();
        assert_eq!("TX", &transport_type_id_2);
        let line_id_2: String = parsed_values.remove(0).into();
        assert_eq!("7341", &line_id_2);
        let direction_2: String = parsed_values.remove(0).into();
        assert_eq!("H", &direction_2);
        let duration: i16 = parsed_values.remove(0).into();
        assert_eq!(0, duration);
        let is_guaranteed: String = parsed_values.remove(0).into();
        assert_eq!("!", &is_guaranteed);
        // Third row
        // "8509002 000011 RE  *        * 000065 S   12       * 008  Landquart".to_string(),
        let (_, _, mut parsed_values) = parser_iterator.next().unwrap().unwrap();
        let stop_id: Option<i32> = parsed_values.remove(0).into();
        assert_eq!(Some(8509002), stop_id);
        let administration_1: String = parsed_values.remove(0).into();
        assert_eq!("000011", &administration_1);
        let transport_type_id_1: String = parsed_values.remove(0).into();
        assert_eq!("RE", &transport_type_id_1);
        let line_id_1: String = parsed_values.remove(0).into();
        assert_eq!("*", &line_id_1);
        let direction_1: String = parsed_values.remove(0).into();
        assert_eq!("*", &direction_1);
        let administration_2: String = parsed_values.remove(0).into();
        assert_eq!("000065", &administration_2);
        let transport_type_id_2: String = parsed_values.remove(0).into();
        assert_eq!("S", &transport_type_id_2);
        let line_id_2: String = parsed_values.remove(0).into();
        assert_eq!("12", &line_id_2);
        let direction_2: String = parsed_values.remove(0).into();
        assert_eq!("*", &direction_2);
        let duration: i16 = parsed_values.remove(0).into();
        assert_eq!(8, duration);
        let is_guaranteed: String = parsed_values.remove(0).into();
        assert_eq!("", &is_guaranteed);
        // Fourth row
        // "8580522 003849 T   #0000482 * 003849 T   #0000488 * 003  Zürich, Escher-Wyss-Platz"
        let (_, _, mut parsed_values) = parser_iterator.next().unwrap().unwrap();
        let stop_id: Option<i32> = parsed_values.remove(0).into();
        assert_eq!(Some(8580522), stop_id);
        let administration_1: String = parsed_values.remove(0).into();
        assert_eq!("003849", &administration_1);
        let transport_type_id_1: String = parsed_values.remove(0).into();
        assert_eq!("T", &transport_type_id_1);
        let line_id_1: String = parsed_values.remove(0).into();
        assert_eq!("#0000482", &line_id_1);
        let direction_1: String = parsed_values.remove(0).into();
        assert_eq!("*", &direction_1);
        let administration_2: String = parsed_values.remove(0).into();
        assert_eq!("003849", &administration_2);
        let transport_type_id_2: String = parsed_values.remove(0).into();
        assert_eq!("T", &transport_type_id_2);
        let line_id_2: String = parsed_values.remove(0).into();
        assert_eq!("#0000488", &line_id_2);
        let direction_2: String = parsed_values.remove(0).into();
        assert_eq!("*", &direction_2);
        let duration: i16 = parsed_values.remove(0).into();
        assert_eq!(3, duration);
        let is_guaranteed: String = parsed_values.remove(0).into();
        assert_eq!("", &is_guaranteed);
    }

    #[test]
    fn type_converter_v207() {
        let rows = vec![
            "8301113 000011 S   *        * 007000 B   *        * 003  Luino (I)".to_string(),
            "1111135 sbg034 B   7339     H sbg034 TX  7341     H 000! Waldshut, Busbahnhof"
                .to_string(),
            "8509002 000011 RE  *        * 000065 S   12       * 008  Landquart".to_string(),
            "8580522 003849 T   #0000482 * 003849 T   #0000488 * 003  Zürich, Escher-Wyss-Platz"
                .to_string(),
        ];
        let parser = FileParser {
            row_parser: exchange_line_row_parser(),
            rows,
        };

        // The transport_types_pk_type_converter is dummy and created just for testing purposes
        let mut transport_types_pk_type_converter: FxHashMap<String, i32> = FxHashMap::default();
        transport_types_pk_type_converter.insert("S".to_string(), 1);
        transport_types_pk_type_converter.insert("B".to_string(), 2);
        transport_types_pk_type_converter.insert("TX".to_string(), 3);
        transport_types_pk_type_converter.insert("RE".to_string(), 4);
        transport_types_pk_type_converter.insert("T".to_string(), 5);

        let data = exchange_line_row_converter(parser, &transport_types_pk_type_converter).unwrap();
        // Id 1
        let attribute = data.get(&1).unwrap();
        let reference = r#"
             {
                 "id": 1,
                 "stop_id": 8301113,
                 "line_1": {
                    "administration": "000011",
                    "transport_type_id": 1,
                    "line_id": null,
                    "direction": null
                 },
                 "line_2": {
                    "administration": "007000",
                    "transport_type_id": 2,
                    "line_id": null,
                    "direction": null
                 },
                 "duration": 3,
                 "is_guaranteed": false
             }"#;
        let (attribute, reference) = get_json_values(attribute, reference).unwrap();
        assert_eq!(attribute, reference);
        // Id 2
        let attribute = data.get(&2).unwrap();
        let reference = r#"
             {
                 "id": 2,
                 "stop_id": 1111135,
                 "line_1": {
                    "administration": "sbg034",
                    "transport_type_id": 2,
                    "line_id": "7339",
                    "direction": "Return"
                 },
                 "line_2": {
                    "administration": "sbg034",
                    "transport_type_id": 3,
                    "line_id": "7341",
                    "direction": "Return"
                 },
                 "duration": 0,
                 "is_guaranteed": true
             }"#;
        let (attribute, reference) = get_json_values(attribute, reference).unwrap();
        assert_eq!(attribute, reference);
        // Id 3
        let attribute = data.get(&3).unwrap();
        let reference = r#"
             {
                 "id": 3,
                 "stop_id": 8509002,
                 "line_1": {
                    "administration": "000011",
                    "transport_type_id": 4,
                    "line_id": null,
                    "direction": null
                 },
                 "line_2": {
                    "administration": "000065",
                    "transport_type_id": 1,
                    "line_id": "12",
                    "direction": null
                 },
                 "duration": 8,
                 "is_guaranteed": false
             }"#;
        let (attribute, reference) = get_json_values(attribute, reference).unwrap();
        assert_eq!(attribute, reference);
        // Id 4
        let attribute = data.get(&4).unwrap();
        let reference = r###"
             {
                 "id": 4,
                 "stop_id": 8580522,
                 "line_1": {
                    "administration": "003849",
                    "transport_type_id": 5,
                    "line_id": "#0000482",
                    "direction": null
                 },
                 "line_2": {
                    "administration": "003849",
                    "transport_type_id": 5,
                    "line_id": "#0000488",
                    "direction": null
                 },
                 "duration": 3,
                 "is_guaranteed": false
             }"###;
        let (attribute, reference) = get_json_values(attribute, reference).unwrap();
        assert_eq!(attribute, reference);
    }
}
