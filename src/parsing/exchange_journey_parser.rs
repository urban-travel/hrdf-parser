/// # Journey exchange time parsing
///
/// List of journey pairs that have a special transfer relationship. File contains:
///
/// - HS no. (see BAHNHOF / STATION)
/// - Journey no. 1 (see FPLAN)
/// - TU code 1 (see BETRIEB / OPERATION_*)
/// - Journey no. 2
/// - TU code 2
/// - Transfer time in min.
/// - Traffic day bitfield (see BITFELD file)
/// - HS name
///
/// ## Example (excerpt):
///
/// `
/// 8500218 002351 000011 030351 000011 001 053724 Olten    % HS-Nr 8500218, Fahrt-Nr 002351, TU-Code 000011, Fahrt-Nr 030351, TU-Code 000011, Umsteigezeit 1, Verkehrstage 053724, HS-Name "Olten"
/// `
/// ### Remarks:
///
/// - The HS-Name is ignored here (Olten in this example).
/// - The trafic day bitfield is optional
///
/// 1 file(s).
/// File(s) read by the parser:
/// UMSTEIGZ
use rustc_hash::{FxHashMap, FxHashSet};

use crate::{
    JourneyId, Result,
    error::ErrorKind,
    models::{ExchangeTimeJourney, Model},
    parsing::{ColumnDefinition, ExpectedType, FileParser, ParsedValue, RowDefinition, RowParser},
    storage::ResourceStorage,
    utils::AutoIncrement,
};

fn exchange_journey_row_parser() -> RowParser {
    RowParser::new(vec![
        // This row is used to create a JourneyExchangeTime instance.
        RowDefinition::from(vec![
            ColumnDefinition::new(1, 7, ExpectedType::Integer32),
            ColumnDefinition::new(9, 14, ExpectedType::Integer32),
            ColumnDefinition::new(16, 21, ExpectedType::String),
            ColumnDefinition::new(23, 28, ExpectedType::Integer32),
            ColumnDefinition::new(30, 35, ExpectedType::String),
            ColumnDefinition::new(37, 39, ExpectedType::Integer16),
            ColumnDefinition::new(40, 40, ExpectedType::String),
            ColumnDefinition::new(42, 47, ExpectedType::OptionInteger32),
        ]),
    ])
}
fn exchange_journey_row_converter(
    parser: FileParser,
    journeys_pk_type_converter: &FxHashSet<JourneyId>,
) -> Result<FxHashMap<i32, ExchangeTimeJourney>> {
    let auto_increment = AutoIncrement::new();

    let data = parser
        .parse()
        .map(|x| {
            x.and_then(|(_, _, values)| {
                create_instance(values, &auto_increment, journeys_pk_type_converter)
            })
        })
        .collect::<Result<Vec<_>>>()?;
    let data = ExchangeTimeJourney::vec_to_map(data);
    Ok(data)
}

pub fn parse(
    path: &str,
    journeys_pk_type_converter: &FxHashSet<JourneyId>,
) -> Result<ResourceStorage<ExchangeTimeJourney>> {
    log::info!("Parsing UMSTEIGZ...");
    let row_parser = exchange_journey_row_parser();
    let parser = FileParser::new(&format!("{path}/UMSTEIGZ"), row_parser)?;
    let data = exchange_journey_row_converter(parser, journeys_pk_type_converter)?;

    Ok(ResourceStorage::new(data))
}

// ------------------------------------------------------------------------------------------------
// --- Data Processing Functions
// ------------------------------------------------------------------------------------------------

fn create_instance(
    mut values: Vec<ParsedValue>,
    auto_increment: &AutoIncrement,
    journeys_pk_type_converter: &FxHashSet<JourneyId>,
) -> Result<ExchangeTimeJourney> {
    let stop_id: i32 = values.remove(0).into();
    let journey_id_1: i32 = values.remove(0).into();
    let administration_1: String = values.remove(0).into();
    let journey_id_2: i32 = values.remove(0).into();
    let administration_2: String = values.remove(0).into();
    let duration: i16 = values.remove(0).into();
    let is_guaranteed: String = values.remove(0).into();
    let bit_field_id: Option<i32> = values.remove(0).into();

    let _journey_id_1 = journeys_pk_type_converter
        .get(&(journey_id_1, administration_1.clone()))
        .ok_or(ErrorKind::UnknownLegacyIdAdmin {
            name: "journey",
            id: journey_id_1,
            admin: administration_1.clone(),
        })?;

    let _journey_id_2 = journeys_pk_type_converter
        .get(&(journey_id_2, administration_2.clone()))
        .ok_or(ErrorKind::UnknownLegacyIdAdmin {
            name: "journey",
            id: journey_id_2,
            admin: administration_2.clone(),
        })?;

    // TODO: I haven't seen an is_guaranteed field in the doc. Check if this makes sense.
    // It is present in UMSTEIGL. Mabe a copy/paste leftover
    let is_guaranteed = is_guaranteed == "!";

    Ok(ExchangeTimeJourney::new(
        auto_increment.next(),
        stop_id,
        (journey_id_1, administration_1),
        (journey_id_2, administration_2),
        duration,
        is_guaranteed,
        bit_field_id,
    ))
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;
    use crate::parsing::tests::get_json_values;
    use pretty_assertions::assert_eq;

    #[test]
    fn row_parser_v207() {
        let rows = vec![
            "8501008 023057 000011 001671 000011 002  000010 Genève".to_string(),
            "8501120 001929 000011 024256 000011 999         Lausanne".to_string(),
        ];
        let parser = FileParser {
            row_parser: exchange_journey_row_parser(),
            rows,
        };
        let mut parser_iterator = parser.parse();
        // First row
        let (_, _, mut parsed_values) = parser_iterator.next().unwrap().unwrap();
        let stop_id: i32 = parsed_values.remove(0).into();
        assert_eq!(8501008, stop_id);
        let journey_id_1: i32 = parsed_values.remove(0).into();
        assert_eq!(23057, journey_id_1);
        let administration_1: String = parsed_values.remove(0).into();
        assert_eq!("000011", &administration_1);
        let journey_id_2: i32 = parsed_values.remove(0).into();
        assert_eq!(1671, journey_id_2);
        let administration_2: String = parsed_values.remove(0).into();
        assert_eq!("000011", &administration_2);
        let duration: i16 = parsed_values.remove(0).into();
        assert_eq!(2, duration);
        let is_guaranteed: String = parsed_values.remove(0).into();
        assert_eq!("", &is_guaranteed);
        let bit_field_id: Option<i32> = parsed_values.remove(0).into();
        assert_eq!(Some(10), bit_field_id);
        // Second row
        let (_, _, mut parsed_values) = parser_iterator.next().unwrap().unwrap();
        let stop_id: i32 = parsed_values.remove(0).into();
        assert_eq!(8501120, stop_id);
        let journey_id_1: i32 = parsed_values.remove(0).into();
        assert_eq!(1929, journey_id_1);
        let administration_1: String = parsed_values.remove(0).into();
        assert_eq!("000011", &administration_1);
        let journey_id_2: i32 = parsed_values.remove(0).into();
        assert_eq!(24256, journey_id_2);
        let administration_2: String = parsed_values.remove(0).into();
        assert_eq!("000011", &administration_2);
        let duration: i16 = parsed_values.remove(0).into();
        assert_eq!(999, duration);
        let is_guaranteed: String = parsed_values.remove(0).into();
        assert_eq!("", &is_guaranteed);
        let bit_field_id: Option<i32> = parsed_values.remove(0).into();
        assert_eq!(None, bit_field_id);
    }

    #[test]
    fn type_converter_v207() {
        let rows = vec![
            "8501008 023057 000011 001671 000011 002  000010 Genève".to_string(),
            "8501120 001929 000011 024256 000011 999         Lausanne".to_string(),
        ];
        let parser = FileParser {
            row_parser: exchange_journey_row_parser(),
            rows,
        };

        // The journeys_pk_type_converter is dummy and created just for testing purposes
        let mut journeys_pk_type_converter: FxHashSet<JourneyId> = FxHashSet::default();
        journeys_pk_type_converter.insert((23057, "000011".to_string()));
        journeys_pk_type_converter.insert((1929, "000011".to_string()));
        journeys_pk_type_converter.insert((1671, "000011".to_string()));
        journeys_pk_type_converter.insert((24256, "000011".to_string()));

        let data = exchange_journey_row_converter(parser, &journeys_pk_type_converter).unwrap();
        // First row
        let attribute = data.get(&1).unwrap();
        let reference = r#"
            {
                "id":1,
                "stop_id": 8501008,
                "journey_legacy_id_1": 23057,
                "administration_1": "000011",
                "journey_legacy_id_2": 1671,
                "administration_2": "000011",
                "duration": 2,
                "is_guaranteed": false,
                "bit_field_id": 10
            }"#;
        let (attribute, reference) = get_json_values(attribute, reference).unwrap();
        assert_eq!(attribute, reference);
        let attribute = data.get(&2).unwrap();
        let reference = r#"
            {
                "id":2,
                "stop_id": 8501120,
                "journey_legacy_id_1": 1929,
                "administration_1": "000011",
                "journey_legacy_id_2": 24256,
                "administration_2": "000011",
                "duration": 999,
                "is_guaranteed": false,
                "bit_field_id": null
            }"#;
        let (attribute, reference) = get_json_values(attribute, reference).unwrap();
        assert_eq!(attribute, reference);
    }
}
