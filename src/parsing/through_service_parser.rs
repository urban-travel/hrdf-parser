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
use std::error::Error;

use nom::{Parser, character::char, combinator::map, sequence::preceded};
use rustc_hash::{FxHashMap, FxHashSet};

use crate::{
    JourneyId,
    models::{Model, ThroughService},
    parsing::{
        ColumnDefinition, ExpectedType, FileParser, ParsedValue, RowDefinition, RowParser,
        helpers::{i32_from_n_digits_parser, read_lines, string_from_n_chars_parser},
    },
    storage::ResourceStorage,
    utils::AutoIncrement,
};

enum TroughServiceLine {
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

fn through_service_combinator<'a>()
-> impl Parser<&'a str, Output = TroughServiceLine, Error = nom::error::Error<&'a str>> {
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
        )| TroughServiceLine::ThroughService {
            journey_1_id,
            journey_1_administration,
            journey_1_stop_id,
            journey_2_id,
            journey_2_administration,
            bit_field_id,
            journey_2_stop_id,
        },
    )
}

fn parse_line(
    line: &str,
    data: &mut FxHashMap<i32, ThroughService>,
    journeys_pk_type_converter: &FxHashSet<JourneyId>,
    auto_increment: &AutoIncrement,
) -> Result<(), Box<dyn Error>> {
    let (_, through_service_line) = through_service_combinator()
        .parse(line)
        .map_err(|e| format!("Error {e} while parsing {line}"))?;

    match through_service_line {
        TroughServiceLine::ThroughService {
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
                log::info!(
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
) -> Result<ResourceStorage<ThroughService>, Box<dyn Error>> {
    log::info!("Parsing DURCHBI...");
    let auto_increment = AutoIncrement::new();
    let mut through_services = FxHashMap::default();

    let through_service_lines = read_lines(&format!("{path}/DURCHBI"), 0)?;
    through_service_lines
        .into_iter()
        .filter(|line| !line.trim().is_empty())
        .try_for_each(|line| {
            parse_line(
                &line,
                &mut through_services,
                journeys_pk_type_converter,
                &auto_increment,
            )
            .map_err(|e| format!("Error: {e}, for line: {line}"))
        })?;
    Ok(ResourceStorage::new(through_services))
}

pub fn old_parse(
    path: &str,
    journeys_pk_type_converter: &FxHashSet<JourneyId>,
) -> Result<ResourceStorage<ThroughService>, Box<dyn Error>> {
    log::info!("Parsing DURCHBI...");
    #[rustfmt::skip]
    let row_parser = RowParser::new(vec![
        // This row is used to create a ThroughService instance.
        RowDefinition::from(vec![
            ColumnDefinition::new(1, 6, ExpectedType::Integer32),
            ColumnDefinition::new(8, 13, ExpectedType::String),
            ColumnDefinition::new(15, 21, ExpectedType::Integer32),
            ColumnDefinition::new(23, 28, ExpectedType::Integer32),
            ColumnDefinition::new(30, 35, ExpectedType::String),
            ColumnDefinition::new(37, 42, ExpectedType::Integer32), // Should be INT16 according to the standard. The standard contains an error. The correct type is INT32.
            ColumnDefinition::new(44, 50, ExpectedType::Integer32), // No indication this should be
                                                                    // optional
        ]),
    ]);
    let parser = FileParser::new(&format!("{path}/DURCHBI"), row_parser)?;

    let auto_increment = AutoIncrement::new();

    let data = parser
        .parse()
        .map(|x| {
            x.and_then(|(_, _, values)| {
                create_instance(values, &auto_increment, journeys_pk_type_converter)
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    let data = ThroughService::vec_to_map(data);

    Ok(ResourceStorage::new(data))
}

// ------------------------------------------------------------------------------------------------
// --- Data Processing Functions
// ------------------------------------------------------------------------------------------------

fn create_instance(
    mut values: Vec<ParsedValue>,
    auto_increment: &AutoIncrement,
    journeys_pk_type_converter: &FxHashSet<JourneyId>,
) -> Result<ThroughService, Box<dyn Error>> {
    let journey_1_id: i32 = values.remove(0).into();
    let journey_1_administration: String = values.remove(0).into();
    let journey_1_stop_id: i32 = values.remove(0).into();
    let journey_2_id: i32 = values.remove(0).into();
    let journey_2_administration: String = values.remove(0).into();
    let bit_field_id: i32 = values.remove(0).into();
    let journey_2_stop_id: i32 = values.remove(0).into();

    // In some recent cases, the pair journey_1_id and journey_1_administration. For instance
    // 030004 and 007058 does not have a journey associated with it.
    let journey_1 =
        journeys_pk_type_converter.get(&(journey_1_id, journey_1_administration.clone()));
    if journey_1.is_none() {
        log::warn!("Unknown legacy ID for journey_1: {journey_1_id}, {journey_1_administration}");
    }

    let journey_2 =
        journeys_pk_type_converter.get(&(journey_2_id, journey_2_administration.clone()));
    if journey_2.is_none() {
        log::warn!("Unknown legacy ID for journey_2: {journey_2_id}, {journey_2_administration}");
    }

    if journey_1_stop_id != journey_2_stop_id {
        log::info!(
            "Journey 1 last stop does not match journey 2 first stop: {journey_1_stop_id}, {journey_2_stop_id}"
        );
    }

    Ok(ThroughService::new(
        auto_increment.next(),
        (journey_1_id, journey_1_administration),
        journey_1_stop_id,
        (journey_2_id, journey_2_administration),
        journey_2_stop_id,
        bit_field_id,
    ))
}
