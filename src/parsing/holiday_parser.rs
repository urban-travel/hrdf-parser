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
use std::{error::Error, str::FromStr};

use chrono::NaiveDate;
use rustc_hash::FxHashMap;

use crate::{
    models::{Holiday, Language, Model},
    parsing::{ColumnDefinition, ExpectedType, FileParser, ParsedValue, RowDefinition, RowParser},
    storage::ResourceStorage,
    utils::AutoIncrement,
};

pub fn parse(path: &str) -> Result<ResourceStorage<Holiday>, Box<dyn Error>> {
    log::info!("Parsing FEIERTAG...");
    #[rustfmt::skip]
    let row_parser = RowParser::new(vec![
        // This row is used to create a Holiday instance.
        RowDefinition::from(vec![
            ColumnDefinition::new(1, 10, ExpectedType::String),
            ColumnDefinition::new(12, -1, ExpectedType::String),
        ]),
    ]);
    let parser = FileParser::new(&format!("{path}/FEIERTAG"), row_parser)?;

    let auto_increment = AutoIncrement::new();

    let data = parser
        .parse()
        .map(|x| x.and_then(|(_, _, values)| create_instance(values, &auto_increment)))
        .collect::<Result<Vec<_>, _>>()?;
    let data = Holiday::vec_to_map(data);

    Ok(ResourceStorage::new(data))
}

// ------------------------------------------------------------------------------------------------
// --- Data Processing Functions
// ------------------------------------------------------------------------------------------------

fn create_instance(
    mut values: Vec<ParsedValue>,
    auto_increment: &AutoIncrement,
) -> Result<Holiday, Box<dyn Error>> {
    let date: String = values.remove(0).into();
    let name_translations: String = values.remove(0).into();

    let date = NaiveDate::parse_from_str(&date, "%d.%m.%Y")?;
    let name = parse_name_translations(name_translations)?;

    Ok(Holiday::new(auto_increment.next(), date, name))
}

// ------------------------------------------------------------------------------------------------
// --- Helper Functions
// ------------------------------------------------------------------------------------------------

fn parse_name_translations(
    name_translations: String,
) -> Result<FxHashMap<Language, String>, Box<dyn Error>> {
    name_translations
        .split('>')
        .filter(|&s| !s.is_empty())
        .map(|s| -> Result<(Language, String), Box<dyn Error>> {
            let mut parts = s.split('<');

            let v = parts.next().ok_or("Missing value part")?.to_string();
            let k = parts.next().ok_or("Missing value part")?.to_string();
            let k = Language::from_str(&k)?;

            Ok((k, v))
        })
        .try_fold(FxHashMap::default(), |mut acc, item| {
            let (k, v) = item?;
            acc.insert(k, v);
            Ok(acc)
        })
}
