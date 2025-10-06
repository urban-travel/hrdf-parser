/// # Through Service parser
///
/// - List of ride pairs that form a contiguous run. Travellers can remain seated.
///
/// - This construct is used, among other things, for the formation of the wing trains. File contains:
///     - Journey no. 1
///     - TU code 1
///     - Last stop 1
///     - Journey no. 2
///     - TU code 2
///     - Traffic days (see BITFELD file)
///     - First stop 2
///
/// ## Example (excerpt):
///
/// `
/// ...
/// 000001 000871 8576671 024064 000871 000010 8576671 % Fahrt 1, TU 871, letzte HS 8576671, Fahrt 24064, TU 871, Bitfeld 10, erste HS 8576671
/// 000001 000882 8581701 000041 000882 063787 8581701 % ...
/// 000002 000181 8530625 000003 000181 000000 8530625 % Fahrt 2, TU 181, letzte HS 8530625, Fahrt 3,     TU 181, Bitfeld 0,  erste HS 8530625
/// 000002 000194 8503674 000004 000194 000001 8503674 % ...
/// 000002 000812 8591817 000003 000812 000000 8591817 % ...
/// 000002 000882 8581701 000042 000882 063786 8581701 % ...
/// 000003 000181 8530625 000004 000181 000000 8530625 % Fahrt 3, TU 181, letzte HS 8530625, Fahrt 4, TU 181, Bitfeld 0, erste HS 8530625
/// 000003 000801 8507230 000004 000801 000000 8507230 % ... % Rivera, Passo del Ceneri
/// 000003 000812 8591817 000004 000812 000000 8591817 % ...
/// ...
/// `
///
/// 1 file(s).
/// File(s) read by the parser:
/// DURCHBI
use nom::{IResult, Parser, character::char, combinator::map, sequence::preceded};
use rustc_hash::{FxHashMap, FxHashSet};

use crate::{
    JourneyId,
    models::{Model, ThroughService},
    parsing::{
        error::{HResult, HrdfError, PResult},
        helpers::{i32_from_n_digits_parser, read_lines, string_from_n_chars_parser},
    },
    storage::ResourceStorage,
    utils::AutoIncrement,
};

enum ThroughServiceLine {
    ThroughService {
        journey_1_id: i32,
        journey_1_administration: String,
        journey_1_stop_id: i32,
        journey_2_id: i32,
        journey_2_administration: String,
        bit_field_id: i32,
        journey_2_stop_id: i32,
    },
}

fn through_service_combinator(input: &str) -> IResult<&str, ThroughServiceLine> {
    map(
        (
            i32_from_n_digits_parser(6),
            preceded(char(' '), string_from_n_chars_parser(6)),
            preceded(char(' '), i32_from_n_digits_parser(7)),
            preceded(char(' '), i32_from_n_digits_parser(6)),
            preceded(char(' '), string_from_n_chars_parser(6)),
            preceded(char(' '), i32_from_n_digits_parser(6)), // Should be INT16 according to the standard. The standard contains an error. The correct type is INT32.
            preceded(char(' '), i32_from_n_digits_parser(7)), // No indication
                                                              // this should be optional
        ),
        |(
            journey_1_id,
            journey_1_administration,
            journey_1_stop_id,
            journey_2_id,
            journey_2_administration,
            bit_field_id,
            journey_2_stop_id,
        )| ThroughServiceLine::ThroughService {
            journey_1_id,
            journey_1_administration,
            journey_1_stop_id,
            journey_2_id,
            journey_2_administration,
            bit_field_id,
            journey_2_stop_id,
        },
    )
    .parse(input)
}

fn parse_line(
    line: &str,
    data: &mut FxHashMap<i32, ThroughService>,
    journeys_pk_type_converter: &FxHashSet<JourneyId>,
    auto_increment: &AutoIncrement,
) -> PResult<()> {
    let (_, through_service_line) = through_service_combinator(line)?;

    match through_service_line {
        ThroughServiceLine::ThroughService {
            journey_1_id,
            journey_1_administration,
            journey_1_stop_id,
            journey_2_id,
            journey_2_administration,
            bit_field_id,
            journey_2_stop_id,
        } => {
            let journey_1 =
                journeys_pk_type_converter.get(&(journey_1_id, journey_1_administration.clone()));
            if journey_1.is_none() {
                log::warn!(
                    "Unknown legacy ID for journey_1: {journey_1_id}, {journey_1_administration}"
                );
            }

            let journey_2 =
                journeys_pk_type_converter.get(&(journey_2_id, journey_2_administration.clone()));
            if journey_2.is_none() {
                log::warn!(
                    "Unknown legacy ID for journey_2: {journey_2_id}, {journey_2_administration}"
                );
            }

            if journey_1_stop_id != journey_2_stop_id {
                log::warn!(
                    "Journey 1 last stop does not match journey 2 first stop: {journey_1_stop_id}, {journey_2_stop_id}"
                );
            }

            let ts = ThroughService::new(
                auto_increment.next(),
                (journey_1_id, journey_1_administration),
                journey_1_stop_id,
                (journey_2_id, journey_2_administration),
                journey_2_stop_id,
                bit_field_id,
            );
            data.insert(ts.id(), ts);
        }
    }
    Ok(())
}

pub fn parse(
    path: &str,
    journeys_pk_type_converter: &FxHashSet<JourneyId>,
) -> HResult<ResourceStorage<ThroughService>> {
    log::info!("Parsing DURCHBI...");
    let auto_increment = AutoIncrement::new();
    let mut through_services = FxHashMap::default();

    let file = format!("{path}/DURCHBI");
    let through_service_lines = read_lines(&file, 0)?;
    through_service_lines
        .into_iter()
        .enumerate()
        .filter(|(_, line)| !line.trim().is_empty())
        .try_for_each(|(line_number, line)| {
            parse_line(
                &line,
                &mut through_services,
                journeys_pk_type_converter,
                &auto_increment,
            )
            .map_err(|e| HrdfError::Parsing {
                error: e,
                file: String::from(&file),
                line,
                line_number,
            })
        })?;
    Ok(ResourceStorage::new(through_services))
}

#[cfg(test)]
mod tests {
    use crate::parsing::tests::get_json_values;

    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_through_service_combinator_basic() {
        let input = "000001 000871 8576671 024064 000871 000010 8576671";
        let result = through_service_combinator(input);
        assert!(result.is_ok());
        let (_, ts_line) = result.unwrap();
        match ts_line {
            ThroughServiceLine::ThroughService {
                journey_1_id,
                journey_1_administration,
                journey_1_stop_id,
                journey_2_id,
                journey_2_administration,
                bit_field_id,
                journey_2_stop_id,
            } => {
                assert_eq!(journey_1_id, 1);
                assert_eq!(journey_1_administration, "000871");
                assert_eq!(journey_1_stop_id, 8576671);
                assert_eq!(journey_2_id, 24064);
                assert_eq!(journey_2_administration, "000871");
                assert_eq!(bit_field_id, 10);
                assert_eq!(journey_2_stop_id, 8576671);
            }
        }
    }

    #[test]
    fn test_through_service_combinator_zero_bitfield() {
        let input = "000002 000181 8530625 000003 000181 000000 8530625";
        let result = through_service_combinator(input);
        assert!(result.is_ok());
        let (_, ts_line) = result.unwrap();
        match ts_line {
            ThroughServiceLine::ThroughService {
                journey_1_id,
                journey_1_administration,
                journey_1_stop_id,
                journey_2_id,
                journey_2_administration,
                bit_field_id,
                journey_2_stop_id,
            } => {
                assert_eq!(journey_1_id, 2);
                assert_eq!(journey_1_administration, "000181");
                assert_eq!(journey_1_stop_id, 8530625);
                assert_eq!(journey_2_id, 3);
                assert_eq!(journey_2_administration, "000181");
                assert_eq!(bit_field_id, 0);
                assert_eq!(journey_2_stop_id, 8530625);
            }
        }
    }

    #[test]
    fn test_through_service_combinator_different_stops() {
        let input = "000002 000194 8503674 000004 000194 000001 8503674";
        let result = through_service_combinator(input);
        assert!(result.is_ok());
        let (_, ts_line) = result.unwrap();
        match ts_line {
            ThroughServiceLine::ThroughService {
                journey_1_id,
                journey_2_id,
                bit_field_id,
                ..
            } => {
                assert_eq!(journey_1_id, 2);
                assert_eq!(journey_2_id, 4);
                assert_eq!(bit_field_id, 1);
            }
        }
    }

    #[test]
    fn test_through_service_combinator_large_bitfield() {
        let input = "000001 000882 8581701 000041 000882 063787 8581701";
        let result = through_service_combinator(input);
        assert!(result.is_ok());
        let (_, ts_line) = result.unwrap();
        match ts_line {
            ThroughServiceLine::ThroughService {
                journey_1_id,
                journey_2_id,
                bit_field_id,
                ..
            } => {
                assert_eq!(journey_1_id, 1);
                assert_eq!(journey_2_id, 41);
                assert_eq!(bit_field_id, 63787);
            }
        }
    }

    #[test]
    fn test_parse_line_creates_through_service() {
        let mut data = FxHashMap::default();
        let mut journeys = FxHashSet::default();
        journeys.insert((1, "000871".to_string()));
        journeys.insert((24064, "000871".to_string()));
        let auto_increment = AutoIncrement::new();

        parse_line(
            "000001 000871 8576671 024064 000871 000010 8576671",
            &mut data,
            &journeys,
            &auto_increment,
        )
        .unwrap();

        assert_eq!(data.len(), 1);
        let ts = data.get(&1).unwrap();

        let reference = r#"{
            "id":1,
            "journey_1_id":[1,"000871"],
            "journey_1_stop_id":8576671,
            "journey_2_id":[24064,"000871"],
            "journey_2_stop_id":8576671,
            "bit_field_id":10
        }"#;
        let (ts, reference) = get_json_values(ts, reference).unwrap();
        assert_eq!(ts, reference);
    }

    #[test]
    fn test_parse_line_missing_journey_logs_warning() {
        let mut data = FxHashMap::default();
        let journeys = FxHashSet::default(); // Empty - journeys not found
        let auto_increment = AutoIncrement::new();

        // Should still succeed but log warnings
        parse_line(
            "000001 000871 8576671 024064 000871 000010 8576671",
            &mut data,
            &journeys,
            &auto_increment,
        )
        .unwrap();

        // Still creates the through service despite missing journeys
        assert_eq!(data.len(), 1);
        let ts = data.get(&1).unwrap();
        let reference = r#"{
            "id":1,
            "journey_1_id":[1,"000871"],
            "journey_1_stop_id":8576671,
            "journey_2_id":[24064,"000871"],
            "journey_2_stop_id":8576671,
            "bit_field_id":10
        }"#;
        let (ts, reference) = get_json_values(ts, reference).unwrap();
        assert_eq!(ts, reference);
    }

    #[test]
    fn test_parse_line_multiple_through_services() {
        let mut data = FxHashMap::default();
        let mut journeys = FxHashSet::default();
        journeys.insert((1, "000871".to_string()));
        journeys.insert((24064, "000871".to_string()));
        journeys.insert((2, "000181".to_string()));
        journeys.insert((3, "000181".to_string()));
        let auto_increment = AutoIncrement::new();

        parse_line(
            "000001 000871 8576671 024064 000871 000010 8576671",
            &mut data,
            &journeys,
            &auto_increment,
        )
        .unwrap();

        parse_line(
            "000002 000181 8530625 000003 000181 000000 8530625",
            &mut data,
            &journeys,
            &auto_increment,
        )
        .unwrap();

        assert_eq!(data.len(), 2);
        let ts = data.get(&1).unwrap();
        let reference = r#"{
            "id":1,
            "journey_1_id":[1,"000871"],
            "journey_1_stop_id":8576671,
            "journey_2_id":[24064,"000871"],
            "journey_2_stop_id":8576671,
            "bit_field_id":10
        }"#;
        let (ts, reference) = get_json_values(ts, reference).unwrap();
        assert_eq!(ts, reference);
        let ts = data.get(&2).unwrap();
        let reference = r#"{
            "id":2,
            "journey_1_id":[2,"000181"],
            "journey_1_stop_id":8530625,
            "journey_2_id":[3,"000181"],
            "journey_2_stop_id":8530625,
            "bit_field_id":0
        }"#;
        let (ts, reference) = get_json_values(ts, reference).unwrap();
        assert_eq!(ts, reference);
    }

    #[test]
    fn test_parse_line_matching_stops() {
        let mut data = FxHashMap::default();
        let journeys = FxHashSet::default();
        let auto_increment = AutoIncrement::new();

        // Same stop ID for journey 1 last stop and journey 2 first stop
        parse_line(
            "000002 000181 8530625 000003 000181 000000 8530625",
            &mut data,
            &journeys,
            &auto_increment,
        )
        .unwrap();

        let reference = r#"{
            "id":1,
            "journey_1_id":[2,"000181"],
            "journey_1_stop_id":8530625,
            "journey_2_id":[3,"000181"],
            "journey_2_stop_id":8530625,
            "bit_field_id":0
        }"#;
        let ts = data.get(&1).unwrap();
        // Both stops should be the same
        assert_eq!(ts.journey_1_stop_id(), 8530625);
        assert_eq!(ts.journey_2_stop_id(), 8530625);
        // Check the rest as well
        let (ts, reference) = get_json_values(ts, reference).unwrap();
        assert_eq!(ts, reference);
    }
}
