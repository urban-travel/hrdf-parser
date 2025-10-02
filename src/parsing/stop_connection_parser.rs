/// # METABHF file
///
/// Grouping of stops for the search. By grouping the stops, the search for transport chains takes place at all stops in the group.
///
/// There are 2 parts. file contains:
///
/// - Part One – Transitional Relationships:
///     - `*A-line`:
///         - Transition
///         - followed by the attribute code
///     - Meta stop ID
///         - Stop ID
///         - Transition time in minutes
/// - Part two – stop groups:
///     - Number of the collective term
///         - “:”
///         - Numbers of the summarised stops
///
/// ## Example (excerpt):
///
/// - The stop 8500010 = “Basel SBB” includes the actual stops (see BAHNHOF file)
///     - 8500146 = “Basel, railway station entrance Gundeldingen$<1>$Basel, railway station entrance Gundeldingen$<2>”
///     - 8578143 = “Basel, Bahnhof SBB$<1>”
///
/// `
/// ...
/// *A Y                             % *A=Übergang, Y="Fussweg" (s. ATTRIBUT-Datei)
/// 8500010 8500146 009              % Meta-HS-Nr. 8500010, HS-Nr. 8500146, Übergang-Minuten: 9
/// *A Y                             % *A=Übergang, Y="Fussweg" (s. ATTRIBUT-Datei)
/// 8500010 8578143 006              % Meta-HS-Nr. 8500010, HS-Nr. 8578143, Übergang-Minuten: 6
/// ...
/// 8389120: 8302430 8389120         % Gruppe: 8389120, umfasst: 8302430, und 8389120
/// 8500010: 8500010 8500146 8578143 % Gruppe: 8500010, umfasst: 8500010, 8500146, und 8578143
/// 8500016: 8500016 8592322         % Gruppe: 8500016, umfasst: 8500016, und 8592322
/// ...
/// `
///
///
/// 1 file(s).
/// File(s) read by the parser:
/// METABHF
use std::error::Error;

use nom::{
    IResult, Parser,
    branch::alt,
    bytes::tag,
    character::complete::multispace1,
    combinator::map,
    multi::separated_list0,
    sequence::{preceded, terminated},
};
use rustc_hash::FxHashMap;

use crate::{
    models::{Model, StopConnection},
    parsing::helpers::{
        i16_from_n_digits_parser, i32_from_n_digits_parser, read_lines, string_till_eol_parser,
    },
    storage::ResourceStorage,
    utils::AutoIncrement,
};

enum StopConnectionLine {
    Aline(String),
    MetaStopLine {
        stop_id_1: i32,
        stop_id_2: i32,
        duration: i16,
    },
    StopGourps {
        #[allow(unused)]
        group_id: i32,
        #[allow(unused)]
        stop_group: Vec<i32>,
    },
}

fn a_line_combinator(input: &str) -> IResult<&str, StopConnectionLine> {
    map(preceded(tag("*A"), string_till_eol_parser()), |s| {
        StopConnectionLine::Aline(s)
    })
    .parse(input)
}

fn meta_stop_line_combinator(input: &str) -> IResult<&str, StopConnectionLine> {
    map(
        (
            i32_from_n_digits_parser(7),
            preceded(multispace1, i32_from_n_digits_parser(7)),
            preceded(multispace1, i16_from_n_digits_parser(3)),
        ),
        |(stop_id_1, stop_id_2, duration)| StopConnectionLine::MetaStopLine {
            stop_id_1,
            stop_id_2,
            duration,
        },
    )
    .parse(input)
}

fn stop_groups_combinator(input: &str) -> IResult<&str, StopConnectionLine> {
    map(
        (
            terminated(i32_from_n_digits_parser(7), tag(":")),
            separated_list0(multispace1, i32_from_n_digits_parser(7)),
        ),
        |(group_id, stop_group)| StopConnectionLine::StopGourps {
            group_id,
            stop_group,
        },
    )
    .parse(input)
}

fn parse_line(
    line: &str,
    data: &mut FxHashMap<i32, StopConnection>,
    attributes_pk_type_converter: &FxHashMap<String, i32>,
    auto_increment: &AutoIncrement,
) -> Result<(), Box<dyn Error>> {
    let (_, stop_connection_line) = alt((
        a_line_combinator,
        stop_groups_combinator,
        meta_stop_line_combinator,
    ))
    .parse(line)
    .map_err(|e| format!("Error {e} while parsing {line}"))?;

    match stop_connection_line {
        StopConnectionLine::Aline(s) => {
            let attribute_id = *attributes_pk_type_converter
                .get(&s)
                .ok_or("Unknown legacy attribute ID: {s}")?;
            let current_instance = data.get_mut(&auto_increment.get()).ok_or(format!(
                "Connection instance {} not found.",
                auto_increment.get()
            ))?;

            current_instance.set_attribute(attribute_id);
        }
        StopConnectionLine::MetaStopLine {
            stop_id_1,
            stop_id_2,
            duration,
        } => {
            let stop_connection =
                StopConnection::new(auto_increment.next(), stop_id_1, stop_id_2, duration);
            data.insert(stop_connection.id(), stop_connection);
        }
        StopConnectionLine::StopGourps {
            group_id: _,
            stop_group: _,
        } => {
            // Do nothing for the moment
            // TODO: this lin coud be useful to look faster for connections maybe
        }
    }
    Ok(())
}

pub fn parse(
    path: &str,
    attributes_pk_type_converter: &FxHashMap<String, i32>,
) -> Result<ResourceStorage<StopConnection>, Box<dyn Error>> {
    log::info!("Parsing METABHF...");

    let auto_increment = AutoIncrement::new();
    let mut stations = FxHashMap::default();

    let station_lines = read_lines(&format!("{path}/METABHF"), 0)?;
    station_lines
        .into_iter()
        .filter(|line| !line.trim().is_empty())
        .try_for_each(|line| {
            parse_line(
                &line,
                &mut stations,
                attributes_pk_type_converter,
                &auto_increment,
            )
            .map_err(|e| format!("Error: {e}, for line: {line}"))
        })?;

    Ok(ResourceStorage::new(stations))
}
