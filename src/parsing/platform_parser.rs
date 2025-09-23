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
    bytes::tag,
    character::{char, complete::space1},
    combinator::map,
    multi::separated_list1,
    number::double,
    sequence::{preceded, separated_pair},
};
use rustc_hash::{FxHashMap, FxHashSet};

use crate::{
    JourneyId, Version,
    models::{CoordinateSystem, Coordinates, JourneyPlatform, Model, Platform},
    parsing::{
        ColumnDefinition, ExpectedType, FastRowMatcher, FileParser, ParsedValue, RowDefinition,
        RowParser,
        helpers::{
            i32_from_n_digits_parser, optional_i32_from_n_digits_parser, read_lines,
            string_from_n_chars_parser, string_till_eol_parser,
        },
    },
    storage::ResourceStorage,
    utils::{AutoIncrement, create_time_from_value},
};

const ROW_JOURNEY_PLATFORM: i32 = 1;
const ROW_PLATFORM: i32 = 2;
const ROW_SECTION: i32 = 3;
const ROW_SLOID: i32 = 4;
const ROW_COORD: i32 = 5;

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
        plaform: String,
        code: String,
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
        altitude: f64,
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
            preceded(tag(" G "), string_till_eol_parser()),
        ),
        |(stop_id, index, platform_data)| PlatformLine::Platform {
            stop_id,
            index,
            platform_data,
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
            preceded(tag(" g A "), separated_list1(char(' '), double())),
        ),
        |(stop_id, index, coords)| PlatformLine::Coord {
            stop_id,
            index,
            x: coords[0],
            y: coords[1],
            altitude: coords[2],
        },
    )
}

fn sloid_combinator<'a>()
-> impl Parser<&'a str, Output = PlatformLine, Error = nom::error::Error<&'a str>> {
    map(
        (
            i32_from_n_digits_parser(7),
            preceded(tag(" #"), i32_from_n_digits_parser(7)),
            preceded(tag(" k "), string_till_eol_parser()),
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
        } => {}
        PlatformLine::Section {
            stop_id,
            index,
            section_data,
        } => {}
        PlatformLine::Platform {
            stop_id,
            index,
            platform_data,
        } => {
            let id = auto_increment.next();
            let (code, sectors) = parse_platform_data(platform_data)?;

            if let Some(previous) = platforms_pk_type_converter.insert((stop_id, index), id) {
                log::warn!(
                    "Warning: previous id {previous} for ({stop_id}, {index}). The pair (stop_id, index), ({stop_id}, {index}), is not unique."
                );
            };

            Ok(Platform::new(id, code, sectors, stop_id))
        }
        PlatformLine::Sloid {
            stop_id,
            index,
            sloid,
        } => {}
        PlatformLine::Coord {
            stop_id,
            index,
            x,
            y,
            altitude,
        } => {}
    }
    Ok(())
}

pub fn parse(
    version: Version,
    path: &str,
    journeys_pk_type_converter: &FxHashSet<JourneyId>,
) -> Result<(ResourceStorage<JourneyPlatform>, ResourceStorage<Platform>), Box<dyn Error>> {
    log::info!("Parsing GLEIS...");
    let row_parser = construct_row_parser(version);

    let auto_increment = AutoIncrement::new();
    let mut platforms = FxHashMap::default();
    let mut platforms_pk_type_converter = FxHashMap::default();

    let mut journey_platform = FxHashMap::default();

    let lines = read_lines(&format!("{path}/LINIE"), 0)?;

    lines
        .into_iter()
        .filter(|line| !line.trim().is_empty())
        .try_for_each(|line| {
            parse_line(
                &line,
                &mut platforms,
                &mut journey_platform,
                &mut platforms_pk_type_converter,
                &journeys_pk_type_converter,
                &auto_increment,
            )
        })?;

    match version {
        Version::V_5_40_41_2_0_4 | Version::V_5_40_41_2_0_5 | Version::V_5_40_41_2_0_6 => {
            let parser = FileParser::new(&format!("{path}/GLEIS"), row_parser)?;
            for x in parser.parse() {
                let (id, bytes_read, values) = x?;
                match id {
                    ROW_JOURNEY_PLATFORM => {
                        bytes_offset += bytes_read;
                        journey_platform.push(values);
                    }
                    ROW_PLATFORM => {
                        platforms.push(create_platform(
                            values,
                            &auto_increment,
                            &mut platforms_pk_type_converter,
                        )?);
                    }
                    _ => unreachable!(),
                }
            }
        }
        Version::V_5_40_41_2_0_7 => {
            let parser = FileParser::new(&format!("{path}/GLEISE_LV95"), row_parser)?;
            for x in parser.parse() {
                let (id, bytes_read, values) = x?;
                match id {
                    ROW_JOURNEY_PLATFORM => {
                        bytes_offset += bytes_read;
                        journey_platform.push(values);
                    }
                    ROW_PLATFORM => {
                        platforms.push(create_platform(
                            values,
                            &auto_increment,
                            &mut platforms_pk_type_converter,
                        )?);
                    }
                    ROW_SECTION => {
                        // We do nothing
                        // We may want to use section at some point
                    }
                    ROW_SLOID | ROW_COORD => {
                        // We do nothing, coordinates and sloid are parsed afterwards
                    }
                    _ => unreachable!(),
                }
            }
        }
    }

    let mut platforms = Platform::vec_to_map(platforms);

    let journey_platform = journey_platform
        .into_iter()
        .map(|values| {
            create_journey_platform(
                values,
                journeys_pk_type_converter,
                &platforms_pk_type_converter,
            )
        })
        .collect::<Result<Vec<_>, _>>()?;
    let journey_platform = JourneyPlatform::vec_to_map(journey_platform);

    log::info!("Parsing GLEIS_LV95...");
    #[rustfmt::skip]
    load_coordinates_for_platforms(version, path, CoordinateSystem::LV95, bytes_offset, &platforms_pk_type_converter, &mut platforms)?;
    log::info!("Parsing GLEIS_WGS84...");
    #[rustfmt::skip]
    load_coordinates_for_platforms(version, path, CoordinateSystem::WGS84, bytes_offset, &platforms_pk_type_converter, &mut platforms)?;

    Ok((
        ResourceStorage::new(journey_platform),
        ResourceStorage::new(platforms),
    ))
}

fn construct_row_parser(version: Version) -> RowParser {
    match version {
        Version::V_5_40_41_2_0_7 => {
            RowParser::new(vec![
                // This row is used to create a JourneyPlatform instance.
                RowDefinition::new(
                    ROW_JOURNEY_PLATFORM,
                    Box::new(FastRowMatcher::new(23, 1, "#", true)),
                    vec![
                        ColumnDefinition::new(1, 7, ExpectedType::Integer32),
                        ColumnDefinition::new(9, 14, ExpectedType::Integer32),
                        ColumnDefinition::new(16, 21, ExpectedType::String),
                        ColumnDefinition::new(24, 30, ExpectedType::Integer32), // Should be 23-30, but here the # character is ignored.
                        ColumnDefinition::new(32, 35, ExpectedType::OptionInteger32),
                        ColumnDefinition::new(37, 42, ExpectedType::OptionInteger32),
                    ],
                ),
                // This row is used to create a Platform instance.
                RowDefinition::new(
                    ROW_PLATFORM,
                    Box::new(FastRowMatcher::new(18, 1, "G", true)),
                    vec![
                        ColumnDefinition::new(1, 7, ExpectedType::Integer32),
                        ColumnDefinition::new(10, 16, ExpectedType::Integer32), // Should be 9-16, but here the # character is ignored.
                        ColumnDefinition::new(18, -1, ExpectedType::String),
                    ],
                ),
                // This row is used to give set the Section
                RowDefinition::new(
                    ROW_SECTION,
                    Box::new(FastRowMatcher::new(18, 1, "A", true)),
                    vec![
                        ColumnDefinition::new(1, 7, ExpectedType::Integer32),
                        ColumnDefinition::new(10, 16, ExpectedType::Integer32), // Should be 9-16, but here the # character is ignored.
                        ColumnDefinition::new(18, -1, ExpectedType::String),
                    ],
                ),
                // This row is used to set the sloid
                RowDefinition::new(
                    ROW_SLOID,
                    Box::new(FastRowMatcher::new(18, 3, "g A", true)),
                    vec![
                        ColumnDefinition::new(1, 7, ExpectedType::Integer32),
                        ColumnDefinition::new(10, 16, ExpectedType::Integer32), // Should be 9-16, but here the # character is ignored.
                        ColumnDefinition::new(20, -1, ExpectedType::String),
                    ],
                ),
                // This row is used to set the coordinates (either lv95 either wgs84)
                RowDefinition::new(
                    ROW_COORD,
                    Box::new(FastRowMatcher::new(18, 1, "k", true)),
                    vec![
                        ColumnDefinition::new(1, 7, ExpectedType::Integer32),
                        ColumnDefinition::new(10, 16, ExpectedType::Integer32), // Should be 9-16, but here the # character is ignored.
                        ColumnDefinition::new(20, -1, ExpectedType::String),
                    ],
                ),
            ])
        }
        Version::V_5_40_41_2_0_4 | Version::V_5_40_41_2_0_5 | Version::V_5_40_41_2_0_6 => {
            RowParser::new(vec![
                // This row is used to create a JourneyPlatform instance.
                RowDefinition::new(
                    ROW_JOURNEY_PLATFORM,
                    Box::new(FastRowMatcher::new(23, 1, "#", true)),
                    vec![
                        ColumnDefinition::new(1, 7, ExpectedType::Integer32),
                        ColumnDefinition::new(9, 14, ExpectedType::Integer32),
                        ColumnDefinition::new(16, 21, ExpectedType::String),
                        ColumnDefinition::new(24, 30, ExpectedType::Integer32), // Should be 23-30, but here the # character is ignored.
                        ColumnDefinition::new(32, 35, ExpectedType::OptionInteger32),
                        ColumnDefinition::new(37, 42, ExpectedType::OptionInteger32),
                    ],
                ),
                // This row is used to create a Platform instance.
                RowDefinition::new(
                    ROW_PLATFORM,
                    Box::new(FastRowMatcher::new(18, 1, "G", true)),
                    vec![
                        ColumnDefinition::new(1, 7, ExpectedType::Integer32),
                        ColumnDefinition::new(10, 16, ExpectedType::Integer32), // Should be 9-16, but here the # character is ignored.
                        ColumnDefinition::new(18, -1, ExpectedType::String),
                    ],
                ),
                // This row contains the SLOID.
                RowDefinition::new(
                    ROW_SLOID,
                    Box::new(FastRowMatcher::new(18, 3, "I A", true)),
                    vec![
                        ColumnDefinition::new(1, 7, ExpectedType::Integer32),
                        ColumnDefinition::new(10, 16, ExpectedType::Integer32), // Should be 9-16, but here the # character is ignored.
                        ColumnDefinition::new(22, -1, ExpectedType::String), // This column has not been explicitly defined in the SBB specification.
                    ],
                ),
                // This row contains the LV95/WGS84 coordinates.
                RowDefinition::new(
                    ROW_COORD,
                    Box::new(FastRowMatcher::new(18, 1, "K", true)),
                    vec![
                        ColumnDefinition::new(1, 7, ExpectedType::Integer32),
                        ColumnDefinition::new(10, 16, ExpectedType::Integer32), // Should be 9-16, but here the # character is ignored.
                        ColumnDefinition::new(20, -1, ExpectedType::String), // This column has not been explicitly defined in the SBB specification.
                    ],
                ),
            ])
        }
    }
}

pub fn old_parse(
    version: Version,
    path: &str,
    journeys_pk_type_converter: &FxHashSet<JourneyId>,
) -> Result<(ResourceStorage<JourneyPlatform>, ResourceStorage<Platform>), Box<dyn Error>> {
    log::info!("Parsing GLEIS...");
    let row_parser = construct_row_parser(version);

    let auto_increment = AutoIncrement::new();
    let mut platforms = Vec::new();
    let mut platforms_pk_type_converter = FxHashMap::default();

    let mut bytes_offset = 0;
    let mut journey_platform = Vec::new();

    match version {
        Version::V_5_40_41_2_0_4 | Version::V_5_40_41_2_0_5 | Version::V_5_40_41_2_0_6 => {
            let parser = FileParser::new(&format!("{path}/GLEIS"), row_parser)?;
            for x in parser.parse() {
                let (id, bytes_read, values) = x?;
                match id {
                    ROW_JOURNEY_PLATFORM => {
                        bytes_offset += bytes_read;
                        journey_platform.push(values);
                    }
                    ROW_PLATFORM => {
                        platforms.push(create_platform(
                            values,
                            &auto_increment,
                            &mut platforms_pk_type_converter,
                        )?);
                    }
                    _ => unreachable!(),
                }
            }
        }
        Version::V_5_40_41_2_0_7 => {
            let parser = FileParser::new(&format!("{path}/GLEISE_LV95"), row_parser)?;
            for x in parser.parse() {
                let (id, bytes_read, values) = x?;
                match id {
                    ROW_JOURNEY_PLATFORM => {
                        bytes_offset += bytes_read;
                        journey_platform.push(values);
                    }
                    ROW_PLATFORM => {
                        platforms.push(create_platform(
                            values,
                            &auto_increment,
                            &mut platforms_pk_type_converter,
                        )?);
                    }
                    ROW_SECTION => {
                        // We do nothing
                        // We may want to use section at some point
                    }
                    ROW_SLOID | ROW_COORD => {
                        // We do nothing, coordinates and sloid are parsed afterwards
                    }
                    _ => unreachable!(),
                }
            }
        }
    }

    let mut platforms = Platform::vec_to_map(platforms);

    let journey_platform = journey_platform
        .into_iter()
        .map(|values| {
            create_journey_platform(
                values,
                journeys_pk_type_converter,
                &platforms_pk_type_converter,
            )
        })
        .collect::<Result<Vec<_>, _>>()?;
    let journey_platform = JourneyPlatform::vec_to_map(journey_platform);

    log::info!("Parsing GLEIS_LV95...");
    #[rustfmt::skip]
    load_coordinates_for_platforms(version, path, CoordinateSystem::LV95, bytes_offset, &platforms_pk_type_converter, &mut platforms)?;
    log::info!("Parsing GLEIS_WGS84...");
    #[rustfmt::skip]
    load_coordinates_for_platforms(version, path, CoordinateSystem::WGS84, bytes_offset, &platforms_pk_type_converter, &mut platforms)?;

    Ok((
        ResourceStorage::new(journey_platform),
        ResourceStorage::new(platforms),
    ))
}

fn load_coordinates_for_platforms(
    version: Version,
    path: &str,
    coordinate_system: CoordinateSystem,
    bytes_offset: u64,
    pk_type_converter: &FxHashMap<(i32, i32), i32>,
    data: &mut FxHashMap<i32, Platform>,
) -> Result<(), Box<dyn Error>> {
    let row_parser = construct_row_parser(version);
    let filename = match (version, coordinate_system) {
        (
            Version::V_5_40_41_2_0_4 | Version::V_5_40_41_2_0_5 | Version::V_5_40_41_2_0_6,
            CoordinateSystem::LV95,
        ) => "GLEIS_LV95",
        (
            Version::V_5_40_41_2_0_4 | Version::V_5_40_41_2_0_5 | Version::V_5_40_41_2_0_6,
            CoordinateSystem::WGS84,
        ) => "GLEIS_WGS",
        (Version::V_5_40_41_2_0_7, CoordinateSystem::LV95) => "GLEISE_LV95",
        (Version::V_5_40_41_2_0_7, CoordinateSystem::WGS84) => "GLEISE_WGS",
    };
    let parser =
        FileParser::new_with_bytes_offset(&format!("{path}/{filename}"), row_parser, bytes_offset)?;

    match version {
        Version::V_5_40_41_2_0_4 | Version::V_5_40_41_2_0_5 | Version::V_5_40_41_2_0_6 => {
            parser.parse().try_for_each(|x| {
                let (id, _, values) = x?;
                match id {
                    ROW_JOURNEY_PLATFORM | ROW_PLATFORM => {
                        // this one has normally already been parsed
                    }
                    ROW_SLOID => {
                        platform_set_sloid(values, coordinate_system, pk_type_converter, data)?
                    }
                    ROW_COORD => platform_set_coordinates(
                        values,
                        coordinate_system,
                        pk_type_converter,
                        data,
                    )?,
                    _ => unreachable!(),
                }
                Ok(())
            })
        }
        Version::V_5_40_41_2_0_7 => {
            parser.parse().try_for_each(|x| {
                let (id, _, values) = x?;
                match id {
                    ROW_JOURNEY_PLATFORM | ROW_PLATFORM | ROW_SECTION => {
                        // This should already have been treated
                    }
                    ROW_SLOID => {
                        platform_set_sloid(values, coordinate_system, pk_type_converter, data)?
                    }
                    ROW_COORD => platform_set_coordinates(
                        values,
                        coordinate_system,
                        pk_type_converter,
                        data,
                    )?,
                    _ => unreachable!(),
                }
                Ok(())
            })
        }
    }
}

// ------------------------------------------------------------------------------------------------
// --- Helper Functions
// ------------------------------------------------------------------------------------------------

fn create_journey_platform(
    mut values: Vec<ParsedValue>,
    journeys_pk_type_converter: &FxHashSet<JourneyId>,
    platforms_pk_type_converter: &FxHashMap<(i32, i32), i32>,
) -> Result<JourneyPlatform, Box<dyn Error>> {
    let stop_id: i32 = values.remove(0).into();
    let journey_id: i32 = values.remove(0).into();
    let administration: String = values.remove(0).into();
    let index: i32 = values.remove(0).into();
    let time: Option<i32> = values.remove(0).into();
    let bit_field_id: Option<i32> = values.remove(0).into();

    let key = (journey_id, administration.clone());
    let _journey_id = journeys_pk_type_converter.get(&key).ok_or(format!(
        "Unknown legacy journey ID: {journey_id}, {administration}"
    ))?;

    let platform_id = *platforms_pk_type_converter
        .get(&(stop_id, index))
        .ok_or("Unknown legacy platform ID")?;

    let time = time.map(|x| create_time_from_value(x as u32));

    Ok(JourneyPlatform::new(
        journey_id,
        administration,
        platform_id,
        time,
        bit_field_id,
    ))
}

fn create_platform(
    mut values: Vec<ParsedValue>,
    auto_increment: &AutoIncrement,
    platforms_pk_type_converter: &mut FxHashMap<(i32, i32), i32>,
) -> Result<Platform, Box<dyn Error>> {
    let stop_id: i32 = values.remove(0).into();
    let index: i32 = values.remove(0).into();
    let platform_data: String = values.remove(0).into();

    let id = auto_increment.next();
    let (code, sectors) = parse_platform_data(platform_data)?;

    if let Some(previous) = platforms_pk_type_converter.insert((stop_id, index), id) {
        log::warn!(
            "Warning: previous id {previous} for ({stop_id}, {index}). The pair (stop_id, index), ({stop_id}, {index}), is not unique."
        );
    };

    Ok(Platform::new(id, code, sectors, stop_id))
}

fn platform_set_sloid(
    mut values: Vec<ParsedValue>,
    coordinate_system: CoordinateSystem,
    pk_type_converter: &FxHashMap<(i32, i32), i32>,
    data: &mut FxHashMap<i32, Platform>,
) -> Result<(), Box<dyn Error>> {
    // The SLOID is processed only when loading LV95 coordinates.
    if coordinate_system == CoordinateSystem::LV95 {
        let stop_id: i32 = values.remove(0).into();
        let index: i32 = values.remove(0).into();
        let sloid: String = values.remove(0).into();

        let id = pk_type_converter
            .get(&(stop_id, index))
            .ok_or("Unknown legacy ID")?;

        data.get_mut(id).ok_or("Unknown ID")?.set_sloid(sloid);
    }

    Ok(())
}

fn platform_set_coordinates(
    mut values: Vec<ParsedValue>,
    coordinate_system: CoordinateSystem,
    pk_type_converter: &FxHashMap<(i32, i32), i32>,
    data: &mut FxHashMap<i32, Platform>,
) -> Result<(), Box<dyn Error>> {
    let stop_id: i32 = values.remove(0).into();
    let index: i32 = values.remove(0).into();

    let floats: Vec<_> = String::from(values.remove(0))
        .split_whitespace()
        .map(|v| v.parse::<f64>().unwrap())
        .collect();
    let mut xy1 = floats[0];
    let mut xy2 = floats[1];

    if coordinate_system == CoordinateSystem::WGS84 {
        // WGS84 coordinates are stored in reverse order for some unknown reason.
        (xy1, xy2) = (xy2, xy1);
    }

    let coordinate = Coordinates::new(coordinate_system, xy1, xy2);

    let id = &pk_type_converter
        .get(&(stop_id, index))
        .ok_or("Unknown legacy ID")?;
    let platform = data.get_mut(id).ok_or("Unknown ID")?;

    match coordinate_system {
        CoordinateSystem::LV95 => platform.set_lv95_coordinates(coordinate),
        CoordinateSystem::WGS84 => platform.set_wgs84_coordinates(coordinate),
    }

    Ok(())
}

// ------------------------------------------------------------------------------------------------
// --- Helper Functions
// ------------------------------------------------------------------------------------------------

fn parse_platform_data(
    mut platform_data: String,
) -> Result<(String, Option<String>), Box<dyn Error>> {
    platform_data = format!("{platform_data} ");
    let data = platform_data.split("' ").filter(|&s| !s.is_empty()).fold(
        FxHashMap::default(),
        |mut acc, item| {
            let parts: Vec<&str> = item.split(" '").collect();
            acc.insert(parts[0], parts[1]);
            acc
        },
    );

    // There should always be a G entry.
    let code = data
        .get("G")
        .ok_or("Entry of type \"G\" missing.")?
        .to_string();
    let sectors = data.get("A").map(|s| s.to_string());

    Ok((code, sectors))
}
