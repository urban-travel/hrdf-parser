// 3 file(s).
// File(s) read by the parser:
// GLEIS, GLEIS_LV95, GLEIS_WGS
// ---
// Note: this parser collects both the Platform and JourneyPlatform resources.

use rustc_hash::{FxHashMap, FxHashSet};

use crate::{
    JourneyId, Result, Version,
    error::ErrorKind,
    models::{CoordinateSystem, Coordinates, JourneyPlatform, Model, Platform},
    parsing::{
        ColumnDefinition, ExpectedType, FastRowMatcher, FileParser, ParsedValue, RowDefinition,
        RowParser,
    },
    storage::ResourceStorage,
    utils::{AutoIncrement, create_time_from_value},
};

const ROW_JOURNEY_PLATFORM: i32 = 1;
const ROW_PLATFORM: i32 = 2;
const ROW_SECTION: i32 = 3;
const ROW_SLOID: i32 = 4;
const ROW_COORD: i32 = 5;

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

pub fn parse(
    version: Version,
    path: &str,
    journeys_pk_type_converter: &FxHashSet<JourneyId>,
) -> Result<(ResourceStorage<JourneyPlatform>, ResourceStorage<Platform>)> {
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
        .collect::<Result<Vec<_>>>()?;
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
) -> Result<()> {
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
) -> Result<JourneyPlatform> {
    let stop_id: i32 = values.remove(0).into();
    let journey_id: i32 = values.remove(0).into();
    let administration: String = values.remove(0).into();
    let index: i32 = values.remove(0).into();
    let time: Option<i32> = values.remove(0).into();
    let bit_field_id: Option<i32> = values.remove(0).into();

    let _journey_id = journeys_pk_type_converter
        .get(&(journey_id, administration.clone()))
        .ok_or(ErrorKind::UnknownLegacyIdAdmin {
            name: "journey",
            id: journey_id,
            admin: administration.clone(),
        })?;

    let platform_id = *platforms_pk_type_converter.get(&(stop_id, index)).ok_or(
        ErrorKind::UnknownLegacyIdIndex {
            name: "platform",
            id: stop_id,
            index,
        },
    )?;

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
) -> Result<Platform> {
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
) -> Result<()> {
    // The SLOID is processed only when loading LV95 coordinates.
    if coordinate_system == CoordinateSystem::LV95 {
        let stop_id: i32 = values.remove(0).into();
        let index: i32 = values.remove(0).into();
        let sloid: String = values.remove(0).into();

        let id =
            pk_type_converter
                .get(&(stop_id, index))
                .ok_or(ErrorKind::UnknownLegacyIdIndex {
                    name: "stop",
                    id: stop_id,
                    index,
                })?;

        data.get_mut(id)
            .ok_or(ErrorKind::UnknownId(*id))?
            .set_sloid(sloid);
    }

    Ok(())
}

fn platform_set_coordinates(
    mut values: Vec<ParsedValue>,
    coordinate_system: CoordinateSystem,
    pk_type_converter: &FxHashMap<(i32, i32), i32>,
    data: &mut FxHashMap<i32, Platform>,
) -> Result<()> {
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

    let id = pk_type_converter
        .get(&(stop_id, index))
        .ok_or(ErrorKind::UnknownLegacyIdIndex {
            name: "stop",
            id: stop_id,
            index,
        })?;
    let platform = data.get_mut(id).ok_or(ErrorKind::UnknownId(*id))?;

    match coordinate_system {
        CoordinateSystem::LV95 => platform.set_lv95_coordinates(coordinate),
        CoordinateSystem::WGS84 => platform.set_wgs84_coordinates(coordinate),
    }

    Ok(())
}

// ------------------------------------------------------------------------------------------------
// --- Helper Functions
// ------------------------------------------------------------------------------------------------

fn parse_platform_data(mut platform_data: String) -> Result<(String, Option<String>)> {
    platform_data = format!("{} ", platform_data);
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
        .ok_or(ErrorKind::EntryMissing { typ: "G" })?
        .to_string();
    let sectors = data.get("A").map(|s| s.to_string());

    Ok((code, sectors))
}
