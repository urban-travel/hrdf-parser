// 8 file(s).
// File(s) read by the parser:
// BAHNHOF, BFKOORD_LV95, BFKOORD_WGS, BFPRIOS, KMINFO, UMSTEIGB, BHFART_60
// ---
// Files not used by the parser:
// BHFART
use std::vec;

use rustc_hash::FxHashMap;

use crate::{
    Result,
    error::{Error, ErrorKind},
    models::{CoordinateSystem, Coordinates, Model, Stop, Version},
    parsing::{
        ColumnDefinition, ExpectedType, FastRowMatcher, FileParser, ParsedValue, RowDefinition,
        RowParser,
    },
    storage::ResourceStorage,
};

type StopStorageAndExchangeTimes = (ResourceStorage<Stop>, (i16, i16));

pub fn parse(version: Version, path: &str) -> Result<StopStorageAndExchangeTimes> {
    log::info!("Parsing BAHNHOF...");
    #[rustfmt::skip]
    let row_parser = RowParser::new(vec![
        // This row is used to create a Stop instance.
        RowDefinition::from(vec![
            ColumnDefinition::new(1, 7, ExpectedType::Integer32),
            ColumnDefinition::new(13, -1, ExpectedType::String), // Should be 13-62, but some entries go beyond column 62.
        ]),
    ]);
    let parser = FileParser::new(&format!("{path}/BAHNHOF"), row_parser)?;

    let data = parser
        .parse()
        .map(|x| x.map(|(_, _, values)| create_instance(values))?)
        .collect::<Result<Vec<_>>>()?;
    let mut data = Stop::vec_to_map(data);

    log::info!("Parsing BFKOORD_LV95...");
    load_coordinates(version, path, CoordinateSystem::LV95, &mut data)?;
    log::info!("Parsing BFKOORD_WGS...");
    load_coordinates(version, path, CoordinateSystem::WGS84, &mut data)?;
    log::info!("Parsing BFPRIOS...");
    load_exchange_priorities(path, &mut data)?;
    log::info!("Parsing KMINFO...");
    load_exchange_flags(path, &mut data)?;
    log::info!("Parsing UMSTEIGB...");
    let default_exchange_time = load_exchange_times(path, &mut data)?;
    log::info!("Parsing BHFART...");
    load_descriptions(version, path, &mut data)?;

    Ok((ResourceStorage::new(data), default_exchange_time))
}

fn load_coordinates(
    version: Version,
    path: &str,
    coordinate_system: CoordinateSystem,
    data: &mut FxHashMap<i32, Stop>,
) -> Result<()> {
    #[rustfmt::skip]
    let row_parser = RowParser::new(vec![
        // This row contains the LV95/WGS84 coordinates.
        RowDefinition::from(
            match version {
                Version::V_5_40_41_2_0_4 => vec![
                    ColumnDefinition::new(1, 7, ExpectedType::Integer32),
                    ColumnDefinition::new(9, 18, ExpectedType::Float),
                    ColumnDefinition::new(20, 29, ExpectedType::Float),
                    ColumnDefinition::new(31, 36, ExpectedType::Integer16),
                ],
                Version::V_5_40_41_2_0_5 | Version::V_5_40_41_2_0_6 | Version::V_5_40_41_2_0_7 => vec![
                    ColumnDefinition::new(1, 7, ExpectedType::Integer32),
                    ColumnDefinition::new(9, 19, ExpectedType::Float),
                    ColumnDefinition::new(21, 31, ExpectedType::Float),
                    ColumnDefinition::new(33, 39, ExpectedType::Integer16),
                ],
            }
        ),
    ]);
    let filename = match coordinate_system {
        CoordinateSystem::LV95 => "BFKOORD_LV95",
        CoordinateSystem::WGS84 => "BFKOORD_WGS",
    };
    let parser = FileParser::new(&format!("{path}/{filename}"), row_parser)?;

    parser.parse().try_for_each(|x| {
        let (_, _, values) = x?;
        set_coordinates(values, coordinate_system, data)?;
        Ok(())
    })
}

fn load_exchange_priorities(path: &str, data: &mut FxHashMap<i32, Stop>) -> Result<()> {
    #[rustfmt::skip]
    let row_parser = RowParser::new(vec![
        // This row contains the changing priority.
        RowDefinition::from(vec![
            ColumnDefinition::new(1, 7, ExpectedType::Integer32),
            ColumnDefinition::new(9, 10, ExpectedType::Integer16),
        ]),
    ]);
    let parser = FileParser::new(&format!("{path}/BFPRIOS"), row_parser)?;

    parser.parse().try_for_each(|x| {
        let (_, _, values) = x?;
        set_exchange_priority(values, data)?;
        Ok(())
    })
}

fn load_exchange_flags(path: &str, data: &mut FxHashMap<i32, Stop>) -> Result<()> {
    #[rustfmt::skip]
    let row_parser = RowParser::new(vec![
        // This row contains the changing flag.
        RowDefinition::from(vec![
            ColumnDefinition::new(1, 7, ExpectedType::Integer32),
            ColumnDefinition::new(9, 13, ExpectedType::Integer16),
        ]),
    ]);
    let parser = FileParser::new(&format!("{path}/KMINFO"), row_parser)?;

    parser.parse().try_for_each(|x| {
        let (_, _, values) = x?;
        set_exchange_flag(values, data)?;
        Ok(())
    })
}

fn load_exchange_times(path: &str, data: &mut FxHashMap<i32, Stop>) -> Result<(i16, i16)> {
    #[rustfmt::skip]
    let row_parser = RowParser::new(vec![
        // This row contains the changing time.
        RowDefinition::from(vec![
            ColumnDefinition::new(1, 7, ExpectedType::Integer32),
            ColumnDefinition::new(9, 10, ExpectedType::Integer16),
            ColumnDefinition::new(12, 13, ExpectedType::Integer16),
        ]),
    ]);
    let parser = FileParser::new(&format!("{path}/UMSTEIGB"), row_parser)?;

    let mut default_exchange_time = (0, 0);

    parser.parse().try_for_each(|x| {
        let (_, _, values) = x?;
        if let Some(x) = set_exchange_time(values, data)? {
            default_exchange_time = x;
        }
        Ok::<_, Error>(())
    })?;

    Ok(default_exchange_time)
}

fn load_descriptions(version: Version, path: &str, data: &mut FxHashMap<i32, Stop>) -> Result<()> {
    const ROW_A: i32 = 1;
    const ROW_B: i32 = 2;
    const ROW_C: i32 = 3;
    const ROW_D: i32 = 4;
    const ROW_E: i32 = 5;
    const ROW_F: i32 = 6;

    #[rustfmt::skip]
    let row_parser = RowParser::new(vec![
        // This row is ignored.
        RowDefinition::new(ROW_A, Box::new(FastRowMatcher::new(1, 1, "%", true)), Vec::new()),
        // This row contains the restrictions.
        RowDefinition::new(ROW_B, Box::new(FastRowMatcher::new(9, 1, "B", true)), vec![
            ColumnDefinition::new(1, 7, ExpectedType::Integer32),
            ColumnDefinition::new(11, 12, ExpectedType::Integer16),
        ]),
        // This row contains the SLOID.
        RowDefinition::new(ROW_C, Box::new(FastRowMatcher::new(11, 1, "A", true)), vec![
            ColumnDefinition::new(1, 7, ExpectedType::Integer32),
            ColumnDefinition::new(13, -1, ExpectedType::String),
        ]),
        // This row contains the boarding areas.
        RowDefinition::new(ROW_D, Box::new(FastRowMatcher::new(11, 1, "a", true)), vec![
            ColumnDefinition::new(1, 7, ExpectedType::Integer32),
            ColumnDefinition::new(13, -1, ExpectedType::String),
        ]),
        // This row contains the country
        RowDefinition::new(ROW_E, Box::new(FastRowMatcher::new(9, 1, "L", true)), vec![
            ColumnDefinition::new(1, 7, ExpectedType::Integer32),
            ColumnDefinition::new(11, 12, ExpectedType::String),
        ]),
        // This row contains the KT (kanton) information
        RowDefinition::new(ROW_F, Box::new(FastRowMatcher::new(9, 1, "I", true)), vec![
            ColumnDefinition::new(1, 7, ExpectedType::Integer32),
            ColumnDefinition::new(11, 12, ExpectedType::String),
            ColumnDefinition::new(14, 22, ExpectedType::Integer32),
        ]),
    ]);

    let bhfart = match version {
        Version::V_5_40_41_2_0_4 | Version::V_5_40_41_2_0_5 | Version::V_5_40_41_2_0_6 => {
            "BHFART_60"
        }
        Version::V_5_40_41_2_0_7 => "BHFART",
    };
    let parser = FileParser::new(&format!("{path}/{bhfart}"), row_parser)?;

    parser.parse().try_for_each(|x| {
        let (id, _, values) = x?;
        match id {
            ROW_A => {}
            ROW_B => set_restrictions(values, data)?,
            ROW_C => set_sloid(values, data)?,
            ROW_D => add_boarding_area(values, data)?,
            ROW_E => {
                // TODO: add possibility to use Land data
            }
            ROW_F => {
                // TODO: add possibility to use KT information and the associated number
            }
            _ => unreachable!(),
        }
        Ok(())
    })
}

// ------------------------------------------------------------------------------------------------
// --- Data Processing Functions
// ------------------------------------------------------------------------------------------------

fn create_instance(mut values: Vec<ParsedValue>) -> Result<Stop> {
    let id: i32 = values.remove(0).into();
    let designations: String = values.remove(0).into();

    let (name, long_name, abbreviation, synonyms) = parse_designations(designations)?;

    Ok(Stop::new(id, name, long_name, abbreviation, synonyms))
}

fn set_coordinates(
    mut values: Vec<ParsedValue>,
    coordinate_system: CoordinateSystem,
    data: &mut FxHashMap<i32, Stop>,
) -> Result<()> {
    let stop_id: i32 = values.remove(0).into();
    let mut xy1: f64 = values.remove(0).into();
    let mut xy2: f64 = values.remove(0).into();
    // Altitude is not stored, as it is not provided for 95% of stops.
    let _altitude: i16 = values.remove(0).into();

    if coordinate_system == CoordinateSystem::WGS84 {
        // WGS84 coordinates are stored in reverse order for some unknown reason.
        (xy1, xy2) = (xy2, xy1);
    }

    let stop = data.get_mut(&stop_id).ok_or(ErrorKind::UnknownId)?;
    let coordinate = Coordinates::new(coordinate_system, xy1, xy2);

    match coordinate_system {
        CoordinateSystem::LV95 => stop.set_lv95_coordinates(coordinate),
        CoordinateSystem::WGS84 => stop.set_wgs84_coordinates(coordinate),
    }

    Ok(())
}

fn set_exchange_priority(
    mut values: Vec<ParsedValue>,
    data: &mut FxHashMap<i32, Stop>,
) -> Result<()> {
    let stop_id: i32 = values.remove(0).into();
    let exchange_priority: i16 = values.remove(0).into();

    let stop = data.get_mut(&stop_id).ok_or(ErrorKind::UnknownId)?;
    stop.set_exchange_priority(exchange_priority);

    Ok(())
}

fn set_exchange_flag(mut values: Vec<ParsedValue>, data: &mut FxHashMap<i32, Stop>) -> Result<()> {
    let stop_id: i32 = values.remove(0).into();
    let exchange_flag: i16 = values.remove(0).into();

    let stop = data.get_mut(&stop_id).ok_or(ErrorKind::UnknownId)?;
    stop.set_exchange_flag(exchange_flag);

    Ok(())
}

fn set_exchange_time(
    mut values: Vec<ParsedValue>,
    data: &mut FxHashMap<i32, Stop>,
) -> Result<Option<(i16, i16)>> {
    let stop_id: i32 = values.remove(0).into();
    let exchange_time_inter_city: i16 = values.remove(0).into();
    let exchange_time_other: i16 = values.remove(0).into();

    let exchange_time = Some((exchange_time_inter_city, exchange_time_other));

    if stop_id == 9999999 {
        // The first row of the file has the stop ID number 9999999.
        // It contains default exchange times to be used when a stop has no specific exchange time.
        Ok(exchange_time)
    } else {
        let stop = data.get_mut(&stop_id).ok_or(ErrorKind::UnknownId)?;
        stop.set_exchange_time(exchange_time);
        Ok(None)
    }
}

fn set_restrictions(mut values: Vec<ParsedValue>, data: &mut FxHashMap<i32, Stop>) -> Result<()> {
    let stop_id: i32 = values.remove(0).into();
    let restrictions: i16 = values.remove(0).into();

    if let Some(stop) = data.get_mut(&stop_id) {
        stop.set_restrictions(restrictions);
    } else {
        log::info!("Unknown ID: {stop_id} for restrictions");
    }

    Ok(())
}

fn set_sloid(mut values: Vec<ParsedValue>, data: &mut FxHashMap<i32, Stop>) -> Result<()> {
    let stop_id: i32 = values.remove(0).into();
    let sloid: String = values.remove(0).into();

    if let Some(stop) = data.get_mut(&stop_id) {
        stop.set_sloid(sloid);
    } else {
        log::info!("Unknown ID: {stop_id} for sloid");
    }

    Ok(())
}

fn add_boarding_area(mut values: Vec<ParsedValue>, data: &mut FxHashMap<i32, Stop>) -> Result<()> {
    let stop_id: i32 = values.remove(0).into();
    let sloid: String = values.remove(0).into();

    if let Some(stop) = data.get_mut(&stop_id) {
        stop.add_boarding_area(sloid);
    } else {
        log::info!("Unknown ID: {stop_id} for boarding area");
    }

    Ok(())
}

// ------------------------------------------------------------------------------------------------
// --- Helper Functions
// ------------------------------------------------------------------------------------------------

type NameAndAlternatives = (String, Option<String>, Option<String>, Option<Vec<String>>);

fn parse_designations(designations: String) -> Result<NameAndAlternatives> {
    let designations = designations
        .split('>')
        .filter(|&s| !s.is_empty())
        .map(|s| -> Result<(i32, String)> {
            let s = s.replace('$', "");
            let mut parts = s.split('<');

            let v = parts.next().ok_or(ErrorKind::MissingValuePart)?.to_string();
            let k = parts
                .next()
                .ok_or(ErrorKind::MissingValuePart)?
                .parse::<i32>()?;

            Ok((k, v))
        })
        .try_fold(
            FxHashMap::default(),
            |mut acc: std::collections::HashMap<i32, Vec<String>, _>, item| {
                let (k, v) = item?;
                acc.entry(k).or_default().push(v);
                Ok::<_, Error>(acc)
            },
        )?;

    let name = designations.get(&1).ok_or(ErrorKind::MissingStopName)?[0].clone();
    let long_name = designations.get(&2).map(|x| x[0].clone());
    let abbreviation = designations.get(&3).map(|x| x[0].clone());
    let synonyms = designations.get(&4).cloned();

    Ok((name, long_name, abbreviation, synonyms))
}
