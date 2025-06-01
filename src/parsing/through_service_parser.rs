// 1 file(s).
// File(s) read by the parser:
// DURCHBI
use std::error::Error;

use rustc_hash::FxHashMap;

use crate::{
    models::{Model, ThroughService},
    parsing::{ColumnDefinition, ExpectedType, FileParser, ParsedValue, RowDefinition, RowParser},
    storage::ResourceStorage,
    utils::AutoIncrement,
};

pub fn parse(
    path: &str,
    trips_pk_type_converter: &FxHashMap<(i32, String), i32>,
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
            ColumnDefinition::new(44, 50, ExpectedType::OptionInteger32),
        ]),
    ]);
    let parser = FileParser::new(&format!("{path}/DURCHBI"), row_parser)?;

    let auto_increment = AutoIncrement::new();

    let data = parser
        .parse()
        .map(|x| {
            x.and_then(|(_, _, values)| {
                create_instance(values, &auto_increment, trips_pk_type_converter)
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
    trips_pk_type_converter: &FxHashMap<(i32, String), i32>,
) -> Result<ThroughService, Box<dyn Error>> {
    let trip_1_id: i32 = values.remove(0).into();
    let trip_1_administration: String = values.remove(0).into();
    let trip_1_stop_id: i32 = values.remove(0).into();
    let trip_2_id: i32 = values.remove(0).into();
    let trip_2_administration: String = values.remove(0).into();
    let bit_field_id: i32 = values.remove(0).into();
    let trip_2_stop_id: Option<i32> = values.remove(0).into();

    let trip_1_id = *trips_pk_type_converter
        .get(&(trip_1_id, trip_1_administration))
        .ok_or("Unknown legacy ID")?;

    let trip_2_id = *trips_pk_type_converter
        .get(&(trip_2_id, trip_2_administration))
        .ok_or("Unknown legacy ID")?;

    Ok(ThroughService::new(
        auto_increment.next(),
        trip_1_id,
        trip_1_stop_id,
        trip_2_id,
        trip_2_stop_id,
        bit_field_id,
    ))
}
