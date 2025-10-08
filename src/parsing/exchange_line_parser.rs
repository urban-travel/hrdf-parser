/// # Line Exchange Time parser
///
/// For more information see
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
use std::{path::Path, str::FromStr};

use nom::{IResult, Parser, character::char, combinator::map, sequence::preceded};
use rustc_hash::FxHashMap;

use crate::{
    error::{HResult, HrdfError},
    models::{DirectionType, ExchangeTimeLine, LineInfo},
    parsing::{
        error::PResult,
        helpers::{
            i16_from_n_digits_parser, optional_i32_from_n_digits_parser, read_lines,
            string_from_n_chars_parser,
        },
    },
    storage::ResourceStorage,
    utils::AutoIncrement,
};

type ExchangeTimeLineRow = (
    Option<i32>,
    String,
    String,
    String,
    String,
    String,
    String,
    String,
    String,
    i16,
    bool,
);

fn parse_exchange_line_row(input: &str) -> IResult<&str, ExchangeTimeLineRow> {
    // TODO: I haven't seen an is_guaranteed field in the doc. Check if this makes sense.
    // It is present in UMSTEIGL. Maybe a copy/paste leftover
    //
    // TODO: There is still a String after all the parsing is done that remains (a name)
    let (
        res,
        (
            stop_id,
            administration_1,
            transport_type_1,
            line_id_1,
            direction_1,
            administration_2,
            transport_type_2,
            line_id_2,
            direction_2,
            duration,
            is_guaranteed,
        ),
    ) = (
        optional_i32_from_n_digits_parser(7),
        preceded(char(' '), string_from_n_chars_parser(6)),
        preceded(char(' '), string_from_n_chars_parser(3)),
        preceded(char(' '), string_from_n_chars_parser(8)),
        preceded(char(' '), string_from_n_chars_parser(1)),
        preceded(char(' '), string_from_n_chars_parser(6)),
        preceded(char(' '), string_from_n_chars_parser(3)),
        preceded(char(' '), string_from_n_chars_parser(8)),
        preceded(char(' '), string_from_n_chars_parser(1)),
        preceded(char(' '), i16_from_n_digits_parser(3)),
        map(string_from_n_chars_parser(1), |s| s == "!"),
    )
        .parse(input)?;
    Ok((
        res,
        (
            stop_id,
            administration_1,
            transport_type_1,
            line_id_1,
            direction_1,
            administration_2,
            transport_type_2,
            line_id_2,
            direction_2,
            duration,
            is_guaranteed,
        ),
    ))
}

fn parse_line(
    line: &str,
    auto_increment: &AutoIncrement,
    transport_types_pk_type_converter: &FxHashMap<String, i32>,
) -> PResult<(i32, ExchangeTimeLine)> {
    let (
        _res,
        (
            stop_id,
            administration_1,
            transport_type_id_1,
            line_id_1,
            direction_1,
            administration_2,
            transport_type_id_2,
            line_id_2,
            direction_2,
            duration,
            is_guaranteed,
        ),
    ) = parse_exchange_line_row(line)?;

    let transport_type_id_1 = *transport_types_pk_type_converter
        .get(&transport_type_id_1)
        .ok_or("Unknown legacy ID for transport_type_1 {transport_type_id_1}")?;

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
        .ok_or("Unknown legacy ID for transport_type_id_2 {transport_type_id_2}")?;

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

    let id = auto_increment.next();

    Ok((
        id,
        ExchangeTimeLine::new(id, stop_id, line_1, line_2, duration, is_guaranteed),
    ))
}

pub fn parse(
    path: &Path,
    transport_types_pk_type_converter: &FxHashMap<String, i32>,
) -> HResult<ResourceStorage<ExchangeTimeLine>> {
    log::info!("Parsing UMSTEIGL...");
    let file = path.join("UMSTEIGL");
    let lines = read_lines(&file, 0)?;
    let auto_increment = AutoIncrement::new();
    let exchanges = lines
        .into_iter()
        .enumerate()
        .filter(|(_, line)| !line.trim().is_empty())
        .map(|(line_number, line)| {
            parse_line(&line, &auto_increment, transport_types_pk_type_converter).map_err(|e| {
                HrdfError::Parsing {
                    error: e,
                    file: String::from(file.to_string_lossy()),
                    line,
                    line_number,
                }
            })
        })
        .collect::<HResult<FxHashMap<_, _>>>()?;

    Ok(ResourceStorage::new(exchanges))
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;
    use crate::parsing::tests::get_json_values;
    use pretty_assertions::assert_eq;

    #[test]
    fn row_parser() {
        let line = "8301113 000011 S   *        * 007000 B   *        * 003  Luino (I)";
        let (
            _res,
            (
                stop_id,
                administration_1,
                transport_type_id_1,
                line_id_1,
                direction_1,
                administration_2,
                transport_type_id_2,
                line_id_2,
                direction_2,
                duration,
                is_guaranteed,
            ),
        ) = parse_exchange_line_row(line).unwrap();

        // "8301113 000011 S   *        * 007000 B   *        * 003  Luino (I)",
        assert_eq!(Some(8301113), stop_id);
        assert_eq!("000011", &administration_1);
        assert_eq!("S", &transport_type_id_1);
        assert_eq!("*", &line_id_1);
        assert_eq!("*", &direction_1);
        assert_eq!("007000", &administration_2);
        assert_eq!("B", &transport_type_id_2);
        assert_eq!("*", &line_id_2);
        assert_eq!("*", &direction_2);
        assert_eq!(3, duration);
        assert!(!is_guaranteed);

        let line = "1111135 sbg034 B   7339     H sbg034 TX  7341     H 000! Waldshut, Busbahnhof";
        let (
            _res,
            (
                stop_id,
                administration_1,
                transport_type_id_1,
                line_id_1,
                direction_1,
                administration_2,
                transport_type_id_2,
                line_id_2,
                direction_2,
                duration,
                is_guaranteed,
            ),
        ) = parse_exchange_line_row(line).unwrap();
        // Second row
        // "1111135 sbg034 B   7339     H sbg034 TX  7341     H 000! Waldshut, Busbahnhof"
        assert_eq!(Some(1111135), stop_id);
        assert_eq!("sbg034", &administration_1);
        assert_eq!("B", &transport_type_id_1);
        assert_eq!("7339", &line_id_1);
        assert_eq!("H", &direction_1);
        assert_eq!("sbg034", &administration_2);
        assert_eq!("TX", &transport_type_id_2);
        assert_eq!("7341", &line_id_2);
        assert_eq!("H", &direction_2);
        assert_eq!(0, duration);
        assert!(is_guaranteed);

        let line = "8509002 000011 RE  *        * 000065 S   12       * 008  Landquart";
        let (
            _res,
            (
                stop_id,
                administration_1,
                transport_type_id_1,
                line_id_1,
                direction_1,
                administration_2,
                transport_type_id_2,
                line_id_2,
                direction_2,
                duration,
                is_guaranteed,
            ),
        ) = parse_exchange_line_row(line).unwrap();
        // Third row
        // "8509002 000011 RE  *        * 000065 S   12       * 008  Landquart".to_string(),
        assert_eq!(Some(8509002), stop_id);
        assert_eq!("000011", &administration_1);
        assert_eq!("RE", &transport_type_id_1);
        assert_eq!("*", &line_id_1);
        assert_eq!("*", &direction_1);
        assert_eq!("000065", &administration_2);
        assert_eq!("S", &transport_type_id_2);
        assert_eq!("12", &line_id_2);
        assert_eq!("*", &direction_2);
        assert_eq!(8, duration);
        assert!(!is_guaranteed);

        let line =
            "8580522 003849 T   #0000482 * 003849 T   #0000488 * 003  Zürich, Escher-Wyss-Platz";
        let (
            _res,
            (
                stop_id,
                administration_1,
                transport_type_id_1,
                line_id_1,
                direction_1,
                administration_2,
                transport_type_id_2,
                line_id_2,
                direction_2,
                duration,
                is_guaranteed,
            ),
        ) = parse_exchange_line_row(line).unwrap();
        // Fourth row
        // "8580522 003849 T   #0000482 * 003849 T   #0000488 * 003  Zürich, Escher-Wyss-Platz"
        assert_eq!(Some(8580522), stop_id);
        assert_eq!("003849", &administration_1);
        assert_eq!("T", &transport_type_id_1);
        assert_eq!("#0000482", &line_id_1);
        assert_eq!("*", &direction_1);
        assert_eq!("003849", &administration_2);
        assert_eq!("T", &transport_type_id_2);
        assert_eq!("#0000488", &line_id_2);
        assert_eq!("*", &direction_2);
        assert_eq!(3, duration);
        assert!(!is_guaranteed);
    }

    #[test]
    fn multiline_parser() {
        let rows = vec![
            "8301113 000011 S   *        * 007000 B   *        * 003  Luino (I)".to_string(),
            "1111135 sbg034 B   7339     H sbg034 TX  7341     H 000! Waldshut, Busbahnhof"
                .to_string(),
            "8509002 000011 RE  *        * 000065 S   12       * 008  Landquart".to_string(),
            "8580522 003849 T   #0000482 * 003849 T   #0000488 * 003  Zürich, Escher-Wyss-Platz"
                .to_string(),
        ];

        // The transport_types_pk_type_converter is dummy and created just for testing purposes
        let mut transport_types_pk_type_converter: FxHashMap<String, i32> = FxHashMap::default();
        transport_types_pk_type_converter.insert("S".to_string(), 1);
        transport_types_pk_type_converter.insert("B".to_string(), 2);
        transport_types_pk_type_converter.insert("TX".to_string(), 3);
        transport_types_pk_type_converter.insert("RE".to_string(), 4);
        transport_types_pk_type_converter.insert("T".to_string(), 5);
        let auto_increment = AutoIncrement::new();
        let exchanges = rows
            .into_iter()
            .enumerate()
            .filter(|(_, line)| !line.trim().is_empty())
            .map(|(line_number, line)| {
                parse_line(&line, &auto_increment, &transport_types_pk_type_converter).map_err(
                    |e| HrdfError::Parsing {
                        error: e,
                        file: String::default(),
                        line,
                        line_number,
                    },
                )
            })
            .collect::<HResult<FxHashMap<_, _>>>()
            .unwrap();

        // Id 1
        let attribute = exchanges.get(&1).unwrap();
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
        let attribute = exchanges.get(&2).unwrap();
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
        let attribute = exchanges.get(&3).unwrap();
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
        let attribute = exchanges.get(&4).unwrap();
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
