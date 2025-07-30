/// # Administration Exchange Time parsing
///
/// Tranfer time betweem two transport companies (see
/// [https://opentransportdata.swiss/en/cookbook/hafas-rohdaten-format-hrdf/](the documentation)
///
/// The columns contain the following:
///
/// - The stop number or @
/// - The adminitrative designation 1
/// - The administrative designation 2
/// - The minimum trasnfer time between dadministrations
/// - The stop designations
///
/// Typical rows look like:
///
/// `
/// 8101236 007000 085000 02 Feldkirch      % HS-Nr 8101236, TU-Code 7000,    TU-Code 85000,  Mindestumsteigzeit 2, HS-Name Feldkirch
/// 8101236 81____ 007000 02 Feldkirch      % ...
/// 8500065 000037 000037 00 Ettingen, Dorf % HS-Nr*  8500065, TU-Code 000037,  TU-Code 000037, Mindestumsteigzeit 0, HS-Name Ettingen, Dorf
/// `
///
/// 1 file(s).
/// File(s) read by the parser:
/// UMSTEIGV
use rustc_hash::FxHashMap;

use crate::{
    Result,
    models::{ExchangeTimeAdministration, Model},
    parsing::{ColumnDefinition, ExpectedType, FileParser, ParsedValue, RowDefinition, RowParser},
    storage::ResourceStorage,
    utils::AutoIncrement,
};

fn exchange_administration_row_parser() -> RowParser {
    RowParser::new(vec![
        // This row is used to create a AdministrationExchangeTime instance.
        RowDefinition::from(vec![
            ColumnDefinition::new(1, 7, ExpectedType::OptionInteger32),
            ColumnDefinition::new(9, 14, ExpectedType::String),
            ColumnDefinition::new(16, 21, ExpectedType::String),
            ColumnDefinition::new(23, 24, ExpectedType::Integer16),
        ]),
    ])
}
fn exchange_administration_row_converter(
    parser: FileParser,
) -> Result<FxHashMap<i32, ExchangeTimeAdministration>> {
    let auto_increment = AutoIncrement::new();

    let data = parser
        .parse()
        .map(|x| x.map(|(_, _, values)| create_instance(values, &auto_increment)))
        .collect::<Result<Vec<_>>>()?;
    let data = ExchangeTimeAdministration::vec_to_map(data);
    Ok(data)
}

pub fn parse(path: &str) -> Result<ResourceStorage<ExchangeTimeAdministration>> {
    log::info!("Parsing UMSTEIGV...");
    let row_parser = exchange_administration_row_parser();
    let parser = FileParser::new(&format!("{path}/UMSTEIGV"), row_parser)?;
    let data = exchange_administration_row_converter(parser)?;

    Ok(ResourceStorage::new(data))
}

// ------------------------------------------------------------------------------------------------
// --- Data Processing Functions
// ------------------------------------------------------------------------------------------------

fn row_from_parsed_values(mut values: Vec<ParsedValue>) -> (Option<i32>, String, String, i16) {
    let stop_id: Option<i32> = values.remove(0).into();
    let administration_1: String = values.remove(0).into();
    let administration_2: String = values.remove(0).into();
    let duration: i16 = values.remove(0).into();
    (stop_id, administration_1, administration_2, duration)
}

fn create_instance(
    values: Vec<ParsedValue>,
    auto_increment: &AutoIncrement,
) -> ExchangeTimeAdministration {
    let (stop_id, administration_1, administration_2, duration) = row_from_parsed_values(values);

    ExchangeTimeAdministration::new(
        auto_increment.next(),
        stop_id,
        administration_1,
        administration_2,
        duration,
    )
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
            "1111135 sbg034 sbg034 01 Waldshut, Busbahnhof".to_string(),
            "8501008 085000 000011 10 Genève".to_string(),
            "@@@@@@@ 000793 000873 02".to_string(),
            "8101236 81____ 007000 02 Feldkirch".to_string(),
        ];
        let parser = FileParser {
            row_parser: exchange_administration_row_parser(),
            rows,
        };
        let mut parser_iterator = parser.parse();
        // First row
        let (_, _, parsed_values) = parser_iterator.next().unwrap().unwrap();
        let (stop_id, administration_1, administration_2, duration) =
            row_from_parsed_values(parsed_values);
        assert_eq!(Some(1111135), stop_id);
        assert_eq!("sbg034", &administration_1);
        assert_eq!("sbg034", &administration_2);
        assert_eq!(1, duration);
        // second row
        let (_, _, parsed_values) = parser_iterator.next().unwrap().unwrap();
        let (stop_id, administration_1, administration_2, duration) =
            row_from_parsed_values(parsed_values);
        assert_eq!(Some(8501008), stop_id);
        assert_eq!("085000", &administration_1);
        assert_eq!("000011", &administration_2);
        assert_eq!(10, duration);
        // third row
        let (_, _, parsed_values) = parser_iterator.next().unwrap().unwrap();
        let (stop_id, administration_1, administration_2, duration) =
            row_from_parsed_values(parsed_values);
        assert_eq!(None, stop_id);
        assert_eq!("000793", &administration_1);
        assert_eq!("000873", &administration_2);
        assert_eq!(2, duration);
        // Third row
        let (_, _, parsed_values) = parser_iterator.next().unwrap().unwrap();
        let (stop_id, administration_1, administration_2, duration) =
            row_from_parsed_values(parsed_values);
        assert_eq!(Some(8101236), stop_id);
        assert_eq!("81____", &administration_1);
        assert_eq!("007000", &administration_2);
        assert_eq!(2, duration);
    }

    #[test]
    fn type_converter_v207() {
        let rows = vec![
            "1111135 sbg034 sbg034 01 Waldshut, Busbahnhof".to_string(),
            "8501008 085000 000011 10 Genève".to_string(),
            "@@@@@@@ 000793 000873 02".to_string(),
            "8101236 81____ 007000 02 Feldkirch".to_string(),
        ];
        let parser = FileParser {
            row_parser: exchange_administration_row_parser(),
            rows,
        };
        let data = exchange_administration_row_converter(parser).unwrap();
        // First row
        let attribute = data.get(&1).unwrap();
        let reference = r#"
            {
                "id":1,
                "stop_id": 1111135,
                "administration_1": "sbg034",
                "administration_2": "sbg034",
                "duration": 1
            }"#;
        let (attribute, reference) = get_json_values(attribute, reference).unwrap();
        assert_eq!(attribute, reference);
        // Second row
        let attribute = data.get(&2).unwrap();
        let reference = r#"
            {
                "id":2,
                "stop_id": 8501008,
                "administration_1": "085000",
                "administration_2": "000011",
                "duration": 10
            }"#;
        let (attribute, reference) = get_json_values(attribute, reference).unwrap();
        assert_eq!(attribute, reference);
        // Third row
        let attribute = data.get(&3).unwrap();
        let reference = r#"
            {
                "id":3,
                "stop_id": null,
                "administration_1": "000793",
                "administration_2": "000873",
                "duration": 2
            }"#;
        let (attribute, reference) = get_json_values(attribute, reference).unwrap();
        assert_eq!(attribute, reference);
        // Fourth row
        let attribute = data.get(&4).unwrap();
        let reference = r#"
            {
                "id":4,
                "stop_id": 8101236,
                "administration_1": "81____",
                "administration_2": "007000",
                "duration": 2
            }"#;
        let (attribute, reference) = get_json_values(attribute, reference).unwrap();
        assert_eq!(attribute, reference);
    }
}
