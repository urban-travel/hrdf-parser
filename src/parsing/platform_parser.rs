/// # List of track and bus platform information.
///
/// ## File contains:
///
/// The first part defines validities, TUs and journeys, which are associated with the track infrastructure in the second part:
/// * HS no.
/// * Journey number
/// * Transport company code
/// * Track link ID “#…”
/// * Service running times;
/// * Days of operation
///
/// ## Example (excerpt):
///
/// `
/// ...
/// 8500010 000003 000011 #0000001      053751 % HS-Nr. 8500010, Fahrt-Nr. 3, TU-Code 11 (SBB), Link #1, keine Verkehrszeit, Verkehrstage-bit: 053751 (s. BITFELD-Datei)
/// 8500010 000003 000011 #0000002      053056 % ...
/// 8500010 000003 000011 #0000003      097398 % ...
/// 8500010 000003 000011 #0000001      001345 % HS-Nr. 8500010, Fahrt-Nr. 3, TU-Code 11 (SBB), Link #1, keine Verkehrszeit, Verkehrstage-bit: 001345 (!) anders als erste Zeile!
/// ...
/// 8014413 005338 8006C5 #0000001      075277 % ...
/// 8014331 005338 8006C5 #0000003 0025 049496 % HS-Nr. 8014331, Fahrt-Nr. 5338, TU-Code 8006C5 (DB Regio), Link #3, Verkehrszeit 00:25, Verkehrstage-bit: 049496 (s. BITFELD-Datei)
/// 8014281 005339 8006C5 #0000002      080554 % ...
/// ...
/// `
///
/// The second part describes the infrastructure (tracks or bus platforms) of the stop:
/// * HS no.
/// * Track link ID “#…” linked with part 1 in combination with HS no.
/// * G = track, A = section, T = separator
///
/// ## Description
///
/// `
/// ...
/// 8500010 #0000004 G '9'  % HS-Nr. 8500010, Link #4, Gleis "9"
/// 8500010 #0000001 G '11' % HS-Nr. 8500010, Link #4, Gleis "11" -> Übereinstimmung mit Erster und vierter Zeile im Beispiel oben!, d.h. die beiden mit unterschiedlichen Gültigkeiten beziehen sich auf Gleis 11
/// 8500010 #0000003 G '12' % ...
/// ...
/// 8014330 #0000001 G '2'  % ...
/// 8014331 #0000001 G '1'  % ...
/// 8014331 #0000002 G '2'  % ...
/// 8014331 #0000003 G '3'  % HS-Nr. 8014331, Link #3, Gleis "3" -> Übereinstimmung mit zweiter Zeile im Zweiten Abschnitt im Beispiel oben!
/// 8014332 #0000002 G '1'  % ...
/// ...
/// `
///
/// This creates the overall picture by linking the two pieces of information.
///
/// IMPORTANT NOTE on *WGS and *LV95, as well as “GLEIS” vs “GLEISE”: These two files will replace the “pure” GLEIS and GLEIS_* files in Switzerland in 2024. So GLEISE_WGS and GLEISE_LV95 remain. Accordingly, we have also documented these directly here.
///
/// With the replacement, only the second part changes as follows (further details on this topic and the implementation in Switzerland can be found in the RV):
///
/// * HS no.
/// * Track link ID “#…” linked with part 1 in combination with HS no.
/// * Changed: Track = G, A = Section, g A = Swiss Location ID (SLOID), k = Coordinates (longitude, latitude, altitude)
/// * Important: contrary to the standard, track and section data are in different lines and not in one!
///
/// ## Description
/// * ‘ ‘ means no explicit designation at the location
///
/// ## Example (excerpt):
///
/// `
/// ...
/// 8500207 #0000001 G '1'                    % Hs-Nr. 8500207, Link #1, Gleis "1"
/// 8500207 #0000001 A 'AB'                   % Hs-Nr. 8500207, Link #1, Gleis-Abschnitt "AB" <-> Die verlinkte Fahrt hält an Gleis 1, Abschnitt AB (Tripel HS-Nr., Link, Bezeichnungen)
/// 8503000 #0000002 G '13'                   % ...
/// 8574200 #0000003 G ''                     % Hs-Nr. 8574200, Link #3, Gleis "" <-> Gleis hat keine explizite Bezeichnung am Ort
/// 8574200 #0000003 g A ch:1:sloid:74200:1:3 % Hs-Nr. 8574200, Link #3, SLOID "ch:1:sloid:74200:1:3" <-> Gleis "" hat SLOID wie beschrieben
/// 8574200 #0000003 k 2692827 1247287 680    % Hs-Nr. 8574200, Link #3, Koordinaten 269.. 124.. Höhe 680 <-> Gleis "" mit SLOID hat Koordinaten wie beschrieben
/// ...
/// `
///
///
/// 3 file(s).
/// File(s) read by the parser:
/// GLEIS, GLEIS_LV95, GLEIS_WGS
/// ---
/// Note: this parser collects both the Platform and JourneyPlatform resources.
use std::error::Error;

use nom::{
    Parser,
    branch::alt,
    bytes::complete::tag,
    bytes::streaming::take_until,
    character::{
        char,
        complete::{multispace0, multispace1, space1},
    },
    combinator::{map, opt},
    number::complete::double,
    sequence::{delimited, preceded},
};
use rustc_hash::{FxHashMap, FxHashSet};

use crate::{
    JourneyId, Version,
    models::{CoordinateSystem, Coordinates, JourneyPlatform, Model, Platform},
    parsing::helpers::{
        i32_from_n_digits_parser, optional_i32_from_n_digits_parser, read_lines,
        string_from_n_chars_parser, string_till_eol_parser,
    },
    storage::ResourceStorage,
    utils::{AutoIncrement, create_time_from_value},
};

enum PlatformLine {
    JourneyPlatform {
        stop_id: i32,
        journey_id: i32,
        administration: String,
        index: i32,
        time: Option<i32>,
        bit_field_id: Option<i32>,
    },
    Platform {
        stop_id: i32,
        index: i32,
        platform_name: String,
        code: Option<String>,
    },
    // Currently unused. Maybe we will want to use it at some point
    Section {
        stop_id: i32,
        index: i32,
        section_data: String,
    },
    Sloid {
        stop_id: i32,
        index: i32,
        sloid: String,
    },
    Coord {
        stop_id: i32,
        index: i32,
        x: f64,
        y: f64,
        altitude: Option<f64>,
    },
}

fn journey_platform_combinator<'a>()
-> impl Parser<&'a str, Output = PlatformLine, Error = nom::error::Error<&'a str>> {
    map(
        (
            i32_from_n_digits_parser(7),
            preceded(char(' '), i32_from_n_digits_parser(6)),
            preceded(char(' '), string_from_n_chars_parser(6)),
            preceded((space1, tag("#")), i32_from_n_digits_parser(7)),
            preceded(char(' '), optional_i32_from_n_digits_parser(4)),
            preceded(char(' '), optional_i32_from_n_digits_parser(6)),
        ),
        |(stop_id, journey_id, administration, index, time, bit_field_id)| {
            PlatformLine::JourneyPlatform {
                stop_id,
                journey_id,
                administration,
                index,
                time,
                bit_field_id,
            }
        },
    )
}

fn platform_combinator<'a>()
-> impl Parser<&'a str, Output = PlatformLine, Error = nom::error::Error<&'a str>> {
    map(
        (
            i32_from_n_digits_parser(7),
            preceded(tag(" #"), i32_from_n_digits_parser(7)),
            preceded(tag(" G "), delimited(tag("'"), take_until("'"), tag("'"))),
            preceded(
                opt(tag(" A ")),
                opt(delimited(tag("'"), take_until("'"), tag("'"))),
            ),
        ),
        |(stop_id, index, platform_name, code)| PlatformLine::Platform {
            stop_id,
            index,
            platform_name: platform_name.to_string(),
            code: code.map(String::from),
        },
    )
}

fn section_combinator<'a>()
-> impl Parser<&'a str, Output = PlatformLine, Error = nom::error::Error<&'a str>> {
    map(
        (
            i32_from_n_digits_parser(7),
            preceded(tag(" #"), i32_from_n_digits_parser(7)),
            preceded(tag(" A "), string_till_eol_parser()),
        ),
        |(stop_id, index, section_data)| PlatformLine::Section {
            stop_id,
            index,
            section_data,
        },
    )
}

fn coord_combinator<'a>()
-> impl Parser<&'a str, Output = PlatformLine, Error = nom::error::Error<&'a str>> {
    map(
        (
            i32_from_n_digits_parser(7),
            preceded(tag(" #"), i32_from_n_digits_parser(7)),
            preceded(
                alt((tag(" k"), tag(" K"))),
                (
                    preceded(multispace0, double),
                    preceded(multispace1, double),
                    opt(preceded(multispace1, double)),
                ),
            ),
        ),
        |(stop_id, index, (x, y, altitude))| PlatformLine::Coord {
            stop_id,
            index,
            x,
            y,
            altitude,
        },
    )
}

fn sloid_combinator<'a>()
-> impl Parser<&'a str, Output = PlatformLine, Error = nom::error::Error<&'a str>> {
    map(
        (
            i32_from_n_digits_parser(7),
            preceded(tag(" #"), i32_from_n_digits_parser(7)),
            alt((
                preceded(tag(" g A "), string_till_eol_parser()),
                preceded(tag(" I A "), string_till_eol_parser()),
            )),
        ),
        |(stop_id, index, sloid)| PlatformLine::Sloid {
            stop_id,
            index,
            sloid,
        },
    )
}

fn parse_line(
    line: &str,
    platforms: &mut FxHashMap<i32, Platform>,
    journey_platform: &mut FxHashMap<(i32, i32), JourneyPlatform>,
    platforms_pk_type_converter: &mut FxHashMap<(i32, i32), i32>,
    journeys_pk_type_converter: &FxHashSet<JourneyId>,
    auto_increment: &AutoIncrement,
    coordinate_system: CoordinateSystem,
) -> Result<(), Box<dyn Error>> {
    let (_, platform_row) = alt((
        journey_platform_combinator(),
        platform_combinator(),
        section_combinator(),
        sloid_combinator(),
        coord_combinator(),
    ))
    .parse(line)
    .map_err(|e| format!("Error {e} while parsing {line}"))?;

    match platform_row {
        PlatformLine::JourneyPlatform {
            stop_id,
            journey_id,
            administration,
            index,
            time,
            bit_field_id,
        } => {
            let key = (journey_id, administration.clone());
            let _journey_id = journeys_pk_type_converter.get(&key).ok_or(format!(
                "Unknown legacy journey ID: {journey_id}, {administration}"
            ))?;

            if !platforms_pk_type_converter.is_empty() {
                let platform_id = *platforms_pk_type_converter
                    .get(&(stop_id, index))
                    .ok_or(format!("Unknown legacy platform ID: ({stop_id}, {index})"))?;

                let time = time.map(|x| create_time_from_value(x as u32));

                let jp_instance = JourneyPlatform::new(
                    journey_id,
                    administration,
                    platform_id,
                    time,
                    bit_field_id,
                );
                journey_platform.insert(jp_instance.id(), jp_instance);
            }
        }
        PlatformLine::Section {
            stop_id: _,
            index: _,
            section_data: _,
        } => {
            // TODO: We should maybe use this data at some point
        }
        PlatformLine::Platform {
            stop_id,
            index,
            platform_name,
            code,
        } => {
            let id = auto_increment.next();

            platforms_pk_type_converter
                .entry((stop_id, index))
                .or_insert(id);

            // if let Some(previous) = platforms_pk_type_converter.insert((stop_id, index), id) {
            //     log::warn!(
            //         "Warning: previous id {previous} for ({stop_id}, {index}). The pair (stop_id, index), ({stop_id}, {index}), is not unique."
            //     );
            // };
            let platform_instance = Platform::new(id, platform_name, code, stop_id);
            platforms.insert(platform_instance.id(), platform_instance);
        }
        PlatformLine::Sloid {
            stop_id,
            index,
            sloid,
        } => {
            let id = platforms_pk_type_converter
                .get(&(stop_id, index))
                .ok_or(format!("Unknown legacy ID: ({stop_id}, {index})"))?;

            platforms
                .get_mut(id)
                .ok_or(format!("Unknown ID for platforms: {id}"))?
                .set_sloid(sloid);
            // TODO: We should maybe check for consistency between LV95 and GWS sloids
        }
        PlatformLine::Coord {
            stop_id,
            index,
            x,
            y,
            altitude: _,
        } => {
            let id = platforms_pk_type_converter
                .get(&(stop_id, index))
                .ok_or(format!("Unknown legacy ID: ({stop_id}, {index})"))?;

            let platform = platforms
                .get_mut(id)
                .ok_or(format!("Unknown ID for platforms: {id}"))?;

            match coordinate_system {
                c @ CoordinateSystem::LV95 => {
                    let value = Coordinates::new(c, x, y);
                    platform.set_lv95_coordinates(value);
                }
                c @ CoordinateSystem::WGS84 => {
                    // WGS84 coordinates are stored in reverse order for some unknown reason.
                    let value = Coordinates::new(c, y, x);
                    platform.set_wgs84_coordinates(value);
                }
            }
        }
    }
    Ok(())
}

pub fn parse(
    version: Version,
    path: &str,
    journeys_pk_type_converter: &FxHashSet<JourneyId>,
) -> Result<(ResourceStorage<JourneyPlatform>, ResourceStorage<Platform>), Box<dyn Error>> {
    let prefix = match version {
        Version::V_5_40_41_2_0_7 => "GLEISE",
        Version::V_5_40_41_2_0_4 | Version::V_5_40_41_2_0_5 | Version::V_5_40_41_2_0_6 => "GLEIS",
    };
    let auto_increment = AutoIncrement::new();
    let mut platforms = FxHashMap::default();
    let mut platforms_pk_type_converter = FxHashMap::default();

    let mut journey_platform = FxHashMap::default();

    log::info!("Parsing {prefix}_LV95...");
    let platforms_lv95 = read_lines(&format!("{path}/{prefix}_LV95"), 0)?;
    platforms_lv95
        .into_iter()
        .filter(|line| !line.trim().is_empty())
        .try_for_each(|line| {
            parse_line(
                &line,
                &mut platforms,
                &mut journey_platform,
                &mut platforms_pk_type_converter,
                journeys_pk_type_converter,
                &auto_increment,
                CoordinateSystem::LV95,
            )
            .map_err(|e| format!("Error: {e}, for line: {line}"))
        })?;

    log::info!("Parsing {prefix}_WGS...");
    let platforms_wgs84 = read_lines(&format!("{path}/{prefix}_WGS"), 0)?;
    platforms_wgs84
        .into_iter()
        .filter(|line| !line.trim().is_empty())
        .try_for_each(|line| {
            parse_line(
                &line,
                &mut platforms,
                &mut journey_platform,
                &mut platforms_pk_type_converter,
                journeys_pk_type_converter,
                &auto_increment,
                CoordinateSystem::WGS84,
            )
            .map_err(|e| format!("Error: {e}, for line: {line}"))
        })?;

    Ok((
        ResourceStorage::new(journey_platform),
        ResourceStorage::new(platforms),
    ))
}
