/// # Holiday parsing
///
/// For more informations see
/// [https://opentransportdata.swiss/en/cookbook/hafas-rohdaten-format-hrdf/#Technical_description_What_is_in_the_HRDF_files_contents](the HRDF documentation).
///
/// List of public holidays that apply in Switzerland.
///
/// In addition to the date of the holiday, the description of the holiday is listed in four languages: DE, FR, IT, EN
///
/// Can be read in decoupled from other data.
///
/// 1 file(s).
/// File(s) read by the parser:
/// FEIERTAG
use std::str::FromStr;

use chrono::NaiveDate;
use rustc_hash::FxHashMap;

use crate::{
    Result,
    error::ErrorKind,
    models::{Holiday, Language, Model},
    parsing::{ColumnDefinition, ExpectedType, FileParser, ParsedValue, RowDefinition, RowParser},
    storage::ResourceStorage,
    utils::AutoIncrement,
};

fn holiday_row_parser() -> RowParser {
    RowParser::new(vec![
        // This row is used to create a Holiday instance.
        RowDefinition::from(vec![
            ColumnDefinition::new(1, 10, ExpectedType::String),
            ColumnDefinition::new(12, -1, ExpectedType::String),
        ]),
    ])
}

fn holiday_row_converter(parser: FileParser) -> Result<FxHashMap<i32, Holiday>> {
    let auto_increment = AutoIncrement::new();

    let data = parser
        .parse()
        .map(|x| x.and_then(|(_, _, values)| create_instance(values, &auto_increment)))
        .collect::<Result<Vec<_>>>()?;
    let data = Holiday::vec_to_map(data);
    Ok(data)
}

pub fn parse(path: &str) -> Result<ResourceStorage<Holiday>> {
    log::info!("Parsing FEIERTAG...");
    let row_parser = holiday_row_parser();
    let parser = FileParser::new(&format!("{path}/FEIERTAG"), row_parser)?;
    let data = holiday_row_converter(parser)?;

    Ok(ResourceStorage::new(data))
}

// ------------------------------------------------------------------------------------------------
// --- Data Processing Functions
// ------------------------------------------------------------------------------------------------

fn create_instance(
    mut values: Vec<ParsedValue>,
    auto_increment: &AutoIncrement,
) -> Result<Holiday> {
    let date: String = values.remove(0).into();
    let name_translations: String = values.remove(0).into();

    let date = NaiveDate::parse_from_str(&date, "%d.%m.%Y")?;
    let name = parse_name_translations(name_translations)?;

    Ok(Holiday::new(auto_increment.next(), date, name))
}

// ------------------------------------------------------------------------------------------------
// --- Helper Functions
// ------------------------------------------------------------------------------------------------

fn parse_name_translations(name_translations: String) -> Result<FxHashMap<Language, String>> {
    name_translations
        .split('>')
        .filter(|&s| !s.is_empty())
        .map(|s| -> Result<(Language, String)> {
            let mut parts = s.split('<');

            let v = parts.next().ok_or(ErrorKind::MissingValuePart)?.to_string();
            let k = parts.next().ok_or(ErrorKind::MissingValuePart)?.to_string();
            let k = Language::from_str(&k)?;

            Ok((k, v))
        })
        .try_fold(FxHashMap::default(), |mut acc, item| {
            let (k, v) = item?;
            acc.insert(k, v);
            Ok(acc)
        })
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
            "25.12.2024 Weihnachtstag<deu>Noël<fra>Natale<ita>Christmas Day<eng>".to_string(),
            "26.12.2024 Stephanstag<deu>Saint Etienne<fra>Santo Stefano<ita>Boxing Day<eng>"
                .to_string(),
        ];
        let parser = FileParser {
            row_parser: holiday_row_parser(),
            rows,
        };
        let mut parser_iterator = parser.parse();
        let (_, _, mut parsed_values) = parser_iterator.next().unwrap().unwrap();
        let date: String = parsed_values.remove(0).into();
        assert_eq!("25.12.2024", &date);
        let name_translations: String = parsed_values.remove(0).into();
        assert_eq!(
            "Weihnachtstag<deu>Noël<fra>Natale<ita>Christmas Day<eng>",
            &name_translations
        );
    }

    #[test]
    fn type_converter_v207() {
        let rows = vec![
            "25.12.2024 Weihnachtstag<deu>Noël<fra>Natale<ita>Christmas Day<eng>".to_string(),
            "26.12.2024 Stephanstag<deu>Saint Etienne<fra>Santo Stefano<ita>Boxing Day<eng>"
                .to_string(),
        ];
        let parser = FileParser {
            row_parser: holiday_row_parser(),
            rows,
        };
        let data = holiday_row_converter(parser).unwrap();
        // First row (id: 1)
        let attribute = data.get(&1).unwrap();
        let reference = r#"
            {
                "id": 1,
                "date": "2024-12-25",
                "name": {
                    "German": "Weihnachtstag",
                    "English": "Christmas Day",
                    "French": "Noël",
                    "Italian": "Natale"
                }
            }"#;
        let (attribute, reference) = get_json_values(attribute, reference).unwrap();
        assert_eq!(attribute, reference);
        // Second row (id: 2)
        let attribute = data.get(&2).unwrap();
        let reference = r#"
            {
                "id": 2,
                "date": "2024-12-26",
                "name": {
                    "German": "Stephanstag",
                    "English": "Boxing Day",
                    "French": "Saint Etienne",
                    "Italian": "Santo Stefano"
                }
            }"#;
        let (attribute, reference) = get_json_values(attribute, reference).unwrap();
        assert_eq!(attribute, reference);
    }
}
