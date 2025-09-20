/// # Journey exchange time parsing
///
/// List of journey pairs that have a special transfer relationship. File contains:
///
/// - HS no. (see BAHNHOF / STATION)
/// - Journey no. 1 (see FPLAN)
/// - TU code 1 (see BETRIEB / OPERATION_*)
/// - Journey no. 2
/// - TU code 2
/// - Transfer time in min.
/// - Traffic day bitfield (see BITFELD file)
/// - HS name
///
/// ## Example (excerpt):
///
/// `
/// 8500218 002351 000011 030351 000011 001 053724 Olten    % HS-Nr 8500218, Fahrt-Nr 002351, TU-Code 000011, Fahrt-Nr 030351, TU-Code 000011, Umsteigezeit 1, Verkehrstage 053724, HS-Name "Olten"
/// `
/// ### Remarks:
///
/// - The HS-Name is ignored here (Olten in this example).
/// - The trafic day bitfield is optional
///
/// 1 file(s).
/// File(s) read by the parser:
/// UMSTEIGZ
use std::error::Error;

use nom::{IResult, Parser, character::char, combinator::map};
use rustc_hash::FxHashSet;

use crate::{
    JourneyId,
    models::{ExchangeTimeJourney, Model},
    parsing::helpers::{
        i16_from_n_digits_parser, i32_from_n_digits_parser, optional_i32_from_n_digits_parser,
        read_lines, string_from_n_chars_parser,
    },
    storage::ResourceStorage,
    utils::AutoIncrement,
};

fn parse_exchange_journey_row(
    input: &str,
) -> IResult<&str, (i32, i32, String, i32, String, i16, bool, Option<i32>)> {
    // TODO: I haven't seen an is_guaranteed field in the doc. Check if this makes sense.
    // It is present in UMSTEIGL. Mabe a copy/paste leftover
    //
    // TODO: There is still a String after all the parsing is done that remains (a name)
    let (
        res,
        (
            stop_id,
            _,
            journey_id_1,
            _,
            administration_1,
            _,
            journey_id_2,
            _,
            administration_2,
            _,
            duration,
            is_guaranteed,
            _,
            bitfield_id,
        ),
    ) = (
        i32_from_n_digits_parser(7),
        char(' '),
        i32_from_n_digits_parser(6),
        char(' '),
        string_from_n_chars_parser(6),
        char(' '),
        i32_from_n_digits_parser(6),
        char(' '),
        string_from_n_chars_parser(6),
        char(' '),
        i16_from_n_digits_parser(3),
        map(string_from_n_chars_parser(1), |s| s == "!"),
        char(' '),
        optional_i32_from_n_digits_parser(6),
    )
        .parse(input)?;
    Ok((
        res,
        (
            stop_id,
            journey_id_1,
            administration_1,
            journey_id_2,
            administration_2,
            duration,
            is_guaranteed,
            bitfield_id,
        ),
    ))
}

fn parse_line(
    line: &str,
    auto_increment: &AutoIncrement,
    journeys_pk_type_converter: &FxHashSet<JourneyId>,
) -> Result<ExchangeTimeJourney, Box<dyn Error>> {
    let (
        _,
        (
            stop_id,
            journey_id_1,
            administration_1,
            journey_id_2,
            administration_2,
            duration,
            is_guaranteed,
            bitfield_id,
        ),
    ) = parse_exchange_journey_row(line).map_err(|e| format!("Error {e} while parsing {line}"))?;

    let _journey_id_1 = journeys_pk_type_converter
        .get(&(journey_id_1, administration_1.clone()))
        .ok_or(format!(
            "Unknown legacy ID for ({journey_id_1}, {administration_1})"
        ))?;

    let _journey_id_2 = journeys_pk_type_converter
        .get(&(journey_id_2, administration_2.clone()))
        .ok_or(format!(
            "Unknown legacy ID for ({journey_id_2}, {administration_2})"
        ))?;

    Ok(ExchangeTimeJourney::new(
        auto_increment.next(),
        stop_id,
        (journey_id_1, administration_1),
        (journey_id_2, administration_2),
        duration,
        is_guaranteed,
        bitfield_id,
    ))
}

pub fn parse(
    path: &str,
    journeys_pk_type_converter: &FxHashSet<JourneyId>,
) -> Result<ResourceStorage<ExchangeTimeJourney>, Box<dyn Error>> {
    log::info!("Parsing UMSTEIGZ...");

    let lines = read_lines(&format!("{path}/UMSTEIGZ"), 0)?;
    let auto_increment = AutoIncrement::new();
    let exchanges = lines
        .into_iter()
        .filter(|line| !line.trim().is_empty())
        .map(|line| parse_line(&line, &auto_increment, journeys_pk_type_converter))
        .collect::<Result<Vec<_>, Box<dyn Error>>>()?;
    let exchanges = ExchangeTimeJourney::vec_to_map(exchanges);

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
        let line = "8501008 023057 000011 001671 000011 002  000010 Genève";
        let (
            _res,
            (
                stop_id,
                journey_id_1,
                administration_1,
                journey_id_2,
                administration_2,
                duration,
                is_guaranteed,
                bit_field_id,
            ),
        ) = parse_exchange_journey_row(line).unwrap();
        assert_eq!(8501008, stop_id);
        assert_eq!(23057, journey_id_1);
        assert_eq!("000011", &administration_1);
        assert_eq!(1671, journey_id_2);
        assert_eq!("000011", &administration_2);
        assert_eq!(2, duration);
        assert!(!is_guaranteed);
        assert_eq!(Some(10), bit_field_id);
        let line = "8501120 001929 000011 024256 000011 999         Lausanne";
        let (
            _res,
            (
                stop_id,
                journey_id_1,
                administration_1,
                journey_id_2,
                administration_2,
                duration,
                is_guaranteed,
                bit_field_id,
            ),
        ) = parse_exchange_journey_row(line).unwrap();
        assert_eq!(8501120, stop_id);
        assert_eq!(1929, journey_id_1);
        assert_eq!("000011", &administration_1);
        assert_eq!(24256, journey_id_2);
        assert_eq!("000011", &administration_2);
        assert_eq!(999, duration);
        assert!(!is_guaranteed);
        assert_eq!(None, bit_field_id);
        let line = "8575489 000020 000801 000045 000801 004! 000019 Crana, Ponte Oscuro";
        let (
            _res,
            (
                stop_id,
                journey_id_1,
                administration_1,
                journey_id_2,
                administration_2,
                duration,
                is_guaranteed,
                bit_field_id,
            ),
        ) = parse_exchange_journey_row(line).unwrap();
        assert_eq!(8575489, stop_id);
        assert_eq!(20, journey_id_1);
        assert_eq!("000801", &administration_1);
        assert_eq!(45, journey_id_2);
        assert_eq!("000801", &administration_2);
        assert_eq!(4, duration);
        assert!(is_guaranteed);
        assert_eq!(Some(19), bit_field_id);
    }

    #[test]
    fn multiple_row_parsing() {
        let lines = vec![
            "8501008 023057 000011 001671 000011 002  000010 Genève".to_string(),
            "8501120 001929 000011 024256 000011 999         Lausanne".to_string(),
            "8575489 000020 000801 000045 000801 004! 000019 Crana, Ponte Oscuro".to_string(),
        ];

        // The journeys_pk_type_converter is dummy and created just for testing purposes
        let mut journeys_pk_type_converter: FxHashSet<JourneyId> = FxHashSet::default();
        journeys_pk_type_converter.insert((23057, "000011".to_string()));
        journeys_pk_type_converter.insert((1929, "000011".to_string()));
        journeys_pk_type_converter.insert((1671, "000011".to_string()));
        journeys_pk_type_converter.insert((24256, "000011".to_string()));
        journeys_pk_type_converter.insert((20, "000801".to_string()));
        journeys_pk_type_converter.insert((45, "000801".to_string()));

        let auto_increment = AutoIncrement::new();
        let exchanges = lines
            .into_iter()
            .filter(|line| !line.trim().is_empty())
            .map(|line| parse_line(&line, &auto_increment, &journeys_pk_type_converter))
            .collect::<Result<Vec<_>, Box<dyn Error>>>()
            .unwrap();
        let exchanges = ExchangeTimeJourney::vec_to_map(exchanges);

        // First row
        let attribute = exchanges.get(&1).unwrap();
        let reference = r#"
            {
                "id":1,
                "stop_id": 8501008,
                "journey_legacy_id_1": 23057,
                "administration_1": "000011",
                "journey_legacy_id_2": 1671,
                "administration_2": "000011",
                "duration": 2,
                "is_guaranteed": false,
                "bit_field_id": 10
            }"#;
        let (attribute, reference) = get_json_values(attribute, reference).unwrap();
        assert_eq!(attribute, reference);
        let attribute = exchanges.get(&2).unwrap();
        let reference = r#"
            {
                "id":2,
                "stop_id": 8501120,
                "journey_legacy_id_1": 1929,
                "administration_1": "000011",
                "journey_legacy_id_2": 24256,
                "administration_2": "000011",
                "duration": 999,
                "is_guaranteed": false,
                "bit_field_id": null
            }"#;
        let (attribute, reference) = get_json_values(attribute, reference).unwrap();
        assert_eq!(attribute, reference);
        let attribute = exchanges.get(&3).unwrap();
        let reference = r#"
            {
                "id":3,
                "stop_id": 8575489,
                "journey_legacy_id_1": 20,
                "administration_1": "000801",
                "journey_legacy_id_2": 45,
                "administration_2": "000801",
                "duration": 4,
                "is_guaranteed": true,
                "bit_field_id": 19
            }"#;
        let (attribute, reference) = get_json_values(attribute, reference).unwrap();
        assert_eq!(attribute, reference);
    }
}
