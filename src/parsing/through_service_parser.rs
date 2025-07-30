// 1 file(s).
// File(s) read by the parser:
// DURCHBI

use rustc_hash::FxHashSet;

use crate::{
    JourneyId, Result,
    models::{Model, ThroughService},
    parsing::{ColumnDefinition, ExpectedType, FileParser, ParsedValue, RowDefinition, RowParser},
    storage::ResourceStorage,
    utils::AutoIncrement,
};

pub fn parse(
    path: &str,
    journeys_pk_type_converter: &FxHashSet<JourneyId>,
) -> Result<ResourceStorage<ThroughService>> {
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
        .collect::<Result<Vec<_>>>()?;
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
) -> Result<ThroughService> {
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
        log::warn!(
            "Unknown legacy ID for journey_1: {journey_1_id}, {}",
            journey_1_administration
        );
    }

    let journey_2 =
        journeys_pk_type_converter.get(&(journey_2_id, journey_2_administration.clone()));
    if journey_2.is_none() {
        log::warn!(
            "Unknown legacy ID for journey_2: {journey_2_id}, {}",
            journey_2_administration
        );
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
