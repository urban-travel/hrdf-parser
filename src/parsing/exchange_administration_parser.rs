/// # Administration Exchange Time parsing
///
/// Transfer time between two transport companies (see
/// [https://opentransportdata.swiss/en/cookbook/hafas-rohdaten-format-hrdf/](the documentation)
///
/// The columns contain the following:
///
/// - The stop number or @
/// - The administrative designation 1
/// - The administrative designation 2
/// - The minimum transfer time between administrations
/// - The stop designations
///
/// Typical rows look like:
///
/// `
/// 8101236 007000 085000 02 Feldkirch      % HS-Nr 8101236, TU-Code 7000,    TU-Code 85000,  Mindestumsteigzeit 2, HS-Name Feldkirch
/// 8101236 81____ 007000 02 Feldkirch      % ...
/// 8500065 000037 000037 00 Ettingen, Dorf % HS-Nr*  8500065, TU-Code 000037,  TU-Code 000037, Mindestumsteigzeit 0, HS-Name Ettingen, Dorf
/// `
///
/// 1 file(s).
/// File(s) read by the parser:
/// UMSTEIGV
use nom::{IResult, Parser, character::char, sequence::preceded};
use rustc_hash::FxHashMap;

use crate::{
    error::{HResult, HrdfError},
    models::ExchangeTimeAdministration,
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

fn parse_exchange_administration_row(
    input: &str,
) -> IResult<&str, (Option<i32>, String, String, i16)> {
    let (res, (stop_id, administration_1, administration_2, duration)) = (
        optional_i32_from_n_digits_parser(7),
        preceded(char(' '), string_from_n_chars_parser(6)),
        preceded(char(' '), string_from_n_chars_parser(6)),
        preceded(char(' '), i16_from_n_digits_parser(2)),
    )
        .parse(input)?;
    Ok((res, (stop_id, administration_1, administration_2, duration)))
}

fn parse_line(
    line: &str,
    auto_increment: &AutoIncrement,
) -> PResult<(i32, ExchangeTimeAdministration)> {
    let (_, (stop_id, administration_1, administration_2, duration)) =
        parse_exchange_administration_row(line)?;
    let id = auto_increment.next();

    Ok((
        id,
        ExchangeTimeAdministration::new(id, stop_id, administration_1, administration_2, duration),
    ))
}

pub fn parse(path: &str) -> HResult<ResourceStorage<ExchangeTimeAdministration>> {
    log::info!("Parsing UMSTEIGV...");

    let file = format!("{path}/UMSTEIGV");
    let lines = read_lines(&file, 0)?;
    let auto_increment = AutoIncrement::new();
    let exchanges = lines
        .into_iter()
        .enumerate()
        .filter(|(_, line)| !line.trim().is_empty())
        .map(|(line_number, line)| {
            parse_line(&line, &auto_increment).map_err(|e| HrdfError::Parsing {
                error: e,
                file: String::from(&file),
                line,
                line_number,
            })
        })
        .collect::<HResult<FxHashMap<i32, ExchangeTimeAdministration>>>()?;

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
        let line = "1111135 sbg034 sbg034 01 Waldshut, Busbahnhof";
        let (_, (stop_id, administration_1, administration_2, duration)) =
            parse_exchange_administration_row(line).unwrap();
        assert_eq!(Some(1111135), stop_id);
        assert_eq!("sbg034", &administration_1);
        assert_eq!("sbg034", &administration_2);
        assert_eq!(1, duration);
        let line = "8501008 085000 000011 10 Genève";
        let (_, (stop_id, administration_1, administration_2, duration)) =
            parse_exchange_administration_row(line).unwrap();
        assert_eq!(Some(8501008), stop_id);
        assert_eq!("085000", &administration_1);
        assert_eq!("000011", &administration_2);
        assert_eq!(10, duration);
        let line = "@@@@@@@ 000793 000873 02";
        let (_, (stop_id, administration_1, administration_2, duration)) =
            parse_exchange_administration_row(line).unwrap();
        assert_eq!(None, stop_id);
        assert_eq!("000793", &administration_1);
        assert_eq!("000873", &administration_2);
        assert_eq!(2, duration);
        let line = "8101236 81____ 007000 02 Feldkirch";
        let (_, (stop_id, administration_1, administration_2, duration)) =
            parse_exchange_administration_row(line).unwrap();
        assert_eq!(Some(8101236), stop_id);
        assert_eq!("81____", &administration_1);
        assert_eq!("007000", &administration_2);
        assert_eq!(2, duration);
    }

    #[test]
    fn multiple_row_parsing() {
        let lines = vec![
            "1111135 sbg034 sbg034 01 Waldshut, Busbahnhof".to_string(),
            "8501008 085000 000011 10 Genève".to_string(),
            "@@@@@@@ 000793 000873 02".to_string(),
            "8101236 81____ 007000 02 Feldkirch".to_string(),
        ];
        let auto_increment = AutoIncrement::new();
        let exchanges = lines
            .into_iter()
            .filter(|line| !line.trim().is_empty())
            .map(|line| parse_line(&line, &auto_increment))
            .collect::<PResult<FxHashMap<i32, ExchangeTimeAdministration>>>()
            .unwrap();
        // First row
        let attribute = exchanges.get(&1).unwrap();
        let reference = r#"
            {
                "id":1,
                "stop_id": 1111135,
                "administration_1": "sbg034",
                "administration_2": "sbg034",
                "duration": 1
            }"#;
        let (attribute, reference) = get_json_values(attribute, reference).unwrap();
        assert_eq!(attribute, reference);
        // Second row
        let attribute = exchanges.get(&2).unwrap();
        let reference = r#"
            {
                "id":2,
                "stop_id": 8501008,
                "administration_1": "085000",
                "administration_2": "000011",
                "duration": 10
            }"#;
        let (attribute, reference) = get_json_values(attribute, reference).unwrap();
        assert_eq!(attribute, reference);
        // Third row
        let attribute = exchanges.get(&3).unwrap();
        let reference = r#"
            {
                "id":3,
                "stop_id": null,
                "administration_1": "000793",
                "administration_2": "000873",
                "duration": 2
            }"#;
        let (attribute, reference) = get_json_values(attribute, reference).unwrap();
        assert_eq!(attribute, reference);
        // Fourth row
        let attribute = exchanges.get(&4).unwrap();
        let reference = r#"
            {
                "id":4,
                "stop_id": 8101236,
                "administration_1": "81____",
                "administration_2": "007000",
                "duration": 2
            }"#;
        let (attribute, reference) = get_json_values(attribute, reference).unwrap();
        assert_eq!(attribute, reference);
    }
}
