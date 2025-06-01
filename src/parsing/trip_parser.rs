// 1 file(s).
// File(s) read by the parser:
// FPLAN
use std::error::Error;

use chrono::NaiveTime;
use rustc_hash::FxHashMap;

use crate::{
    models::{Trip, TripMetadataEntry, TripMetadataType, TripRouteEntry, Model},
    parsing::{
        ColumnDefinition, ExpectedType, FastRowMatcher, FileParser, ParsedValue, RowDefinition,
        RowParser,
    },
    storage::ResourceStorage,
    utils::{create_time_from_value, AutoIncrement},
};

pub fn parse(
    path: &str,
    transport_types_pk_type_converter: &FxHashMap<String, i32>,
    attributes_pk_type_converter: &FxHashMap<String, i32>,
    directions_pk_type_converter: &FxHashMap<String, i32>,
) -> Result<(ResourceStorage<Trip>, FxHashMap<(i32, String), i32>), Box<dyn Error>> {
    log::info!("Parsing FPLAN...");
    const ROW_A: i32 = 1;
    const ROW_B: i32 = 2;
    const ROW_C: i32 = 3;
    const ROW_D: i32 = 4;
    const ROW_E: i32 = 5;
    const ROW_F: i32 = 6;
    const ROW_G: i32 = 7;
    const ROW_H: i32 = 8;
    const ROW_I: i32 = 9;

    #[rustfmt::skip]
    let row_parser = RowParser::new(vec![
        // This row is used to create a Trip instance.
        RowDefinition::new(ROW_A, Box::new(FastRowMatcher::new(1, 2, "*Z", true)), vec![
            ColumnDefinition::new(4, 9, ExpectedType::Integer32),
            ColumnDefinition::new(11, 16, ExpectedType::String),
        ]),
        RowDefinition::new(ROW_B, Box::new(FastRowMatcher::new(1, 2, "*G", true)), vec![
            ColumnDefinition::new(4, 6, ExpectedType::String),
            ColumnDefinition::new(8, 14, ExpectedType::OptionInteger32),
            ColumnDefinition::new(16, 22, ExpectedType::OptionInteger32),
        ]),
        RowDefinition::new(ROW_C, Box::new(FastRowMatcher::new(1, 5, "*A VE", true)), vec![
            ColumnDefinition::new(7, 13, ExpectedType::OptionInteger32),
            ColumnDefinition::new(15, 21, ExpectedType::OptionInteger32),
            ColumnDefinition::new(23, 28, ExpectedType::OptionInteger32),
        ]),
        RowDefinition::new(ROW_D, Box::new(FastRowMatcher::new(1, 2, "*A", true)), vec![
            ColumnDefinition::new(4, 5, ExpectedType::String),
            ColumnDefinition::new(7, 13, ExpectedType::OptionInteger32),
            ColumnDefinition::new(15, 21, ExpectedType::OptionInteger32),
        ]),
        RowDefinition::new(ROW_E, Box::new(FastRowMatcher::new(1, 2, "*I", true)), vec![
            ColumnDefinition::new(4, 5, ExpectedType::String),
            ColumnDefinition::new(7, 13, ExpectedType::OptionInteger32),
            ColumnDefinition::new(15, 21, ExpectedType::OptionInteger32),
            ColumnDefinition::new(23, 28, ExpectedType::OptionInteger32),
            ColumnDefinition::new(30, 38, ExpectedType::Integer32),
            ColumnDefinition::new(40, 45, ExpectedType::OptionInteger32),
            ColumnDefinition::new(47, 52, ExpectedType::OptionInteger32),
        ]),
        RowDefinition::new(ROW_F, Box::new(FastRowMatcher::new(1, 2, "*L", true)), vec![
            ColumnDefinition::new(4, 11, ExpectedType::String),
            ColumnDefinition::new(13, 19, ExpectedType::OptionInteger32),
            ColumnDefinition::new(21, 27, ExpectedType::OptionInteger32),
            ColumnDefinition::new(29, 34, ExpectedType::OptionInteger32),
            ColumnDefinition::new(36, 41, ExpectedType::OptionInteger32),
        ]),
        RowDefinition::new(ROW_G, Box::new(FastRowMatcher::new(1, 2, "*R", true)), vec![
            ColumnDefinition::new(4, 4, ExpectedType::String),
            ColumnDefinition::new(6, 12, ExpectedType::String),
            ColumnDefinition::new(14, 20, ExpectedType::OptionInteger32),
            ColumnDefinition::new(22, 28, ExpectedType::OptionInteger32),
            ColumnDefinition::new(30, 35, ExpectedType::OptionInteger32),
            ColumnDefinition::new(37, 42, ExpectedType::OptionInteger32),
        ]),
        // *CI
        RowDefinition::new(ROW_H, Box::new(FastRowMatcher::new(1, 3, "*CI", true)), vec![
            ColumnDefinition::new(1, 3, ExpectedType::String),
            ColumnDefinition::new(5, 8, ExpectedType::Integer32),
            ColumnDefinition::new(10, 16, ExpectedType::OptionInteger32),
            ColumnDefinition::new(18, 24, ExpectedType::OptionInteger32),
        ]),
        // *CO
        RowDefinition::new(ROW_H, Box::new(FastRowMatcher::new(1, 3, "*CO", true)), vec![
            ColumnDefinition::new(1, 3, ExpectedType::String),
            ColumnDefinition::new(5, 8, ExpectedType::Integer32),
            ColumnDefinition::new(10, 16, ExpectedType::OptionInteger32),
            ColumnDefinition::new(18, 24, ExpectedType::OptionInteger32),
        ]),
        RowDefinition::new(ROW_I, Box::new(FastRowMatcher::new(1, 0, "", true)), vec![
            ColumnDefinition::new(1, 7, ExpectedType::Integer32),
            ColumnDefinition::new(30, 35, ExpectedType::OptionInteger32),
            ColumnDefinition::new(37, 42, ExpectedType::OptionInteger32),
        ]),
    ]);
    let parser = FileParser::new(&format!("{path}/FPLAN"), row_parser)?;

    let auto_increment = AutoIncrement::new();
    let mut data = Vec::new();
    let mut pk_type_converter = FxHashMap::default();

    for x in parser.parse() {
        let (id, _, values) = x?;
        match id {
            ROW_A => data.push(create_instance(
                values,
                &auto_increment,
                &mut pk_type_converter,
            )),
            _ => {
                let trip = data.last_mut().ok_or("Type A row missing.")?;

                match id {
                    ROW_B => {
                        set_transport_type(values, trip, &transport_types_pk_type_converter)?
                    }
                    ROW_C => set_bit_field(values, trip),
                    ROW_D => add_attribute(values, trip, &attributes_pk_type_converter)?,
                    ROW_E => add_information_text(values, trip),
                    ROW_F => set_line(values, trip)?,
                    ROW_G => set_direction(values, trip, directions_pk_type_converter)?,
                    ROW_H => set_boarding_or_disembarking_exchange_time(values, trip),
                    ROW_I => add_route_entry(values, trip),
                    _ => unreachable!(),
                }
            }
        }
    }

    let data = Trip::vec_to_map(data);

    Ok((ResourceStorage::new(data), pk_type_converter))
}

// ------------------------------------------------------------------------------------------------
// --- Data Processing Functions
// ------------------------------------------------------------------------------------------------

fn create_instance(
    mut values: Vec<ParsedValue>,
    auto_increment: &AutoIncrement,
    pk_type_converter: &mut FxHashMap<(i32, String), i32>,
) -> Trip {
    let legacy_id: i32 = values.remove(0).into();
    let administration: String = values.remove(0).into();

    let id = auto_increment.next();

    pk_type_converter.insert((legacy_id, administration.to_owned()), id);
    Trip::new(id, administration)
}

fn set_transport_type(
    mut values: Vec<ParsedValue>,
    trip: &mut Trip,
    transport_types_pk_type_converter: &FxHashMap<String, i32>,
) -> Result<(), Box<dyn Error>> {
    let designation: String = values.remove(0).into();
    let from_stop_id: Option<i32> = values.remove(0).into();
    let until_stop_id: Option<i32> = values.remove(0).into();

    let transport_type_id = *transport_types_pk_type_converter
        .get(&designation)
        .ok_or("Unknown legacy ID")?;

    trip.add_metadata_entry(
        TripMetadataType::TransportType,
        TripMetadataEntry::new(
            from_stop_id,
            until_stop_id,
            Some(transport_type_id),
            None,
            None,
            None,
            None,
            None,
        ),
    );

    Ok(())
}

fn set_bit_field(mut values: Vec<ParsedValue>, trip: &mut Trip) {
    let from_stop_id: Option<i32> = values.remove(0).into();
    let until_stop_id: Option<i32> = values.remove(0).into();
    let bit_field_id: Option<i32> = values.remove(0).into();

    trip.add_metadata_entry(
        TripMetadataType::BitField,
        TripMetadataEntry::new(
            from_stop_id,
            until_stop_id,
            None,
            bit_field_id,
            None,
            None,
            None,
            None,
        ),
    );
}

fn add_attribute(
    mut values: Vec<ParsedValue>,
    trip: &mut Trip,
    attributes_pk_type_converter: &FxHashMap<String, i32>,
) -> Result<(), Box<dyn Error>> {
    let designation: String = values.remove(0).into();
    let from_stop_id: Option<i32> = values.remove(0).into();
    let until_stop_id: Option<i32> = values.remove(0).into();

    let attribute_id = *attributes_pk_type_converter
        .get(&designation)
        .ok_or("Unknown legacy ID")?;

    trip.add_metadata_entry(
        TripMetadataType::Attribute,
        TripMetadataEntry::new(
            from_stop_id,
            until_stop_id,
            Some(attribute_id),
            None,
            None,
            None,
            None,
            None,
        ),
    );

    Ok(())
}

fn add_information_text(mut values: Vec<ParsedValue>, trip: &mut Trip) {
    let code: String = values.remove(0).into();
    let from_stop_id: Option<i32> = values.remove(0).into();
    let until_stop_id: Option<i32> = values.remove(0).into();
    let bit_field_id: Option<i32> = values.remove(0).into();
    let information_text_id: i32 = values.remove(0).into();
    let departure_time: Option<i32> = values.remove(0).into();
    let arrival_time: Option<i32> = values.remove(0).into();

    let arrival_time = create_time(arrival_time);
    let departure_time = create_time(departure_time);

    trip.add_metadata_entry(
        TripMetadataType::InformationText,
        TripMetadataEntry::new(
            from_stop_id,
            until_stop_id,
            Some(information_text_id),
            bit_field_id,
            departure_time,
            arrival_time,
            Some(code),
            None,
        ),
    );
}

fn set_line(mut values: Vec<ParsedValue>, trip: &mut Trip) -> Result<(), Box<dyn Error>> {
    let line_designation: String = values.remove(0).into();
    let from_stop_id: Option<i32> = values.remove(0).into();
    let until_stop_id: Option<i32> = values.remove(0).into();
    let departure_time: Option<i32> = values.remove(0).into();
    let arrival_time: Option<i32> = values.remove(0).into();

    let arrival_time = create_time(arrival_time);
    let departure_time = create_time(departure_time);

    let line_designation_first_char = line_designation
        .chars()
        .next()
        .ok_or("Missing designation")?;
    let (resource_id, extra_field_1) = if line_designation_first_char == '#' {
        (Some(line_designation[1..].parse::<i32>()?), None)
    } else {
        (None, Some(line_designation))
    };

    trip.add_metadata_entry(
        TripMetadataType::Line,
        TripMetadataEntry::new(
            from_stop_id,
            until_stop_id,
            resource_id,
            None,
            departure_time,
            arrival_time,
            extra_field_1,
            None,
        ),
    );

    Ok(())
}

fn set_direction(
    mut values: Vec<ParsedValue>,
    trip: &mut Trip,
    directions_pk_type_converter: &FxHashMap<String, i32>,
) -> Result<(), Box<dyn Error>> {
    let direction_type: String = values.remove(0).into();
    let direction_id: String = values.remove(0).into();
    let from_stop_id: Option<i32> = values.remove(0).into();
    let until_stop_id: Option<i32> = values.remove(0).into();
    let departure_time: Option<i32> = values.remove(0).into();
    let arrival_time: Option<i32> = values.remove(0).into();

    let arrival_time = create_time(arrival_time);
    let departure_time = create_time(departure_time);

    let direction_id = if direction_id.is_empty() {
        None
    } else {
        let id = *directions_pk_type_converter
            .get(&direction_id)
            .ok_or("Unknown legacy ID")?;
        Some(id)
    };

    trip.add_metadata_entry(
        TripMetadataType::Direction,
        TripMetadataEntry::new(
            from_stop_id,
            until_stop_id,
            direction_id,
            None,
            departure_time,
            arrival_time,
            Some(direction_type),
            None,
        ),
    );

    Ok(())
}

fn set_boarding_or_disembarking_exchange_time(mut values: Vec<ParsedValue>, trip: &mut Trip) {
    let ci_co: String = values.remove(0).into();
    let exchange_time: i32 = values.remove(0).into();
    let from_stop_id: Option<i32> = values.remove(0).into();
    let until_stop_id: Option<i32> = values.remove(0).into();

    let metadata_type = if ci_co == "*CI" {
        TripMetadataType::ExchangeTimeBoarding
    } else {
        TripMetadataType::ExchangeTimeDisembarking
    };

    trip.add_metadata_entry(
        metadata_type,
        TripMetadataEntry::new(
            from_stop_id,
            until_stop_id,
            None,
            None,
            None,
            None,
            None,
            Some(exchange_time),
        ),
    );
}

fn add_route_entry(mut values: Vec<ParsedValue>, trip: &mut Trip) {
    let stop_id: i32 = values.remove(0).into();
    let arrival_time: Option<i32> = values.remove(0).into();
    let departure_time: Option<i32> = values.remove(0).into();

    let arrival_time = create_time(arrival_time);
    let departure_time = create_time(departure_time);

    trip.add_route_entry(TripRouteEntry::new(
        stop_id,
        arrival_time,
        departure_time,
    ));
}

// ------------------------------------------------------------------------------------------------
// --- Helper Functions
// ------------------------------------------------------------------------------------------------

fn create_time(time: Option<i32>) -> Option<NaiveTime> {
    time.map(|value| {
        create_time_from_value(match value.abs() {
            val if val >= 2400 => val % 2400,
            val => val,
        } as u32)
    })
}
