// 1 file(s).
// File(s) read by the parser:
// METABHF

use rustc_hash::FxHashMap;

use crate::{
    Result,
    error::ErrorKind,
    models::{Model, StopConnection},
    parsing::{
        AdvancedRowMatcher, ColumnDefinition, ExpectedType, FastRowMatcher, FileParser,
        ParsedValue, RowDefinition, RowParser,
    },
    storage::ResourceStorage,
    utils::AutoIncrement,
};

pub fn parse(
    path: &str,
    attributes_pk_type_converter: &FxHashMap<String, i32>,
) -> Result<ResourceStorage<StopConnection>> {
    log::info!("Parsing METABHF...");
    const ROW_A: i32 = 1;
    const ROW_B: i32 = 2;
    const ROW_C: i32 = 3;

    #[rustfmt::skip]
    let row_parser = RowParser::new(vec![
        // This row is used to create a StopConnection instance.
        RowDefinition::new(ROW_A, Box::new(AdvancedRowMatcher::new(r"[0-9]{7} [0-9]{7} [0-9]{3}")?), vec![
            ColumnDefinition::new(1, 7, ExpectedType::Integer32),
            ColumnDefinition::new(9, 15, ExpectedType::Integer32),
            ColumnDefinition::new(17, 19, ExpectedType::Integer16),
        ]),
        // This row contains the attributes.
        RowDefinition::new(ROW_B, Box::new(FastRowMatcher::new(1, 2, "*A", true)), vec![
            ColumnDefinition::new(4, 5, ExpectedType::String),
        ]),
        // This row is ignored.
        RowDefinition::new(ROW_C, Box::new(FastRowMatcher::new(8, 1, ":", true)), Vec::new()),
    ]);
    let parser = FileParser::new(&format!("{path}/METABHF"), row_parser)?;

    let auto_increment = AutoIncrement::new();
    let mut data = Vec::new();

    for x in parser.parse() {
        let (id, _, values) = x?;
        match id {
            ROW_A => data.push(create_instance(values, &auto_increment)),
            _ => {
                let stop_connection = data.last_mut().ok_or(ErrorKind::RowMissing { typ: "A" })?;

                match id {
                    ROW_B => set_attribute(values, stop_connection, attributes_pk_type_converter)?,
                    ROW_C => {}
                    _ => unreachable!(),
                }
            }
        }
    }

    let data = StopConnection::vec_to_map(data);

    Ok(ResourceStorage::new(data))
}

// ------------------------------------------------------------------------------------------------
// --- Data Processing Functions
// ------------------------------------------------------------------------------------------------

fn create_instance(mut values: Vec<ParsedValue>, auto_increment: &AutoIncrement) -> StopConnection {
    let stop_id_1: i32 = values.remove(0).into();
    let stop_id_2: i32 = values.remove(0).into();
    let duration: i16 = values.remove(0).into();

    StopConnection::new(auto_increment.next(), stop_id_1, stop_id_2, duration)
}

fn set_attribute(
    mut values: Vec<ParsedValue>,
    current_instance: &mut StopConnection,
    attributes_pk_type_converter: &FxHashMap<String, i32>,
) -> Result<()> {
    let attribute_designation: String = values.remove(0).into();
    let attribute_id = *attributes_pk_type_converter
        .get(&attribute_designation)
        .ok_or(ErrorKind::UnknownLegacyId)?;
    current_instance.set_attribute(attribute_id);
    Ok(())
}
