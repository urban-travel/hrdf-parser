// 4 file(s).
// File(s) read by the parser:
// BETRIEB_DE, BETRIEB_EN, BETRIEB_FR, BETRIEB_IT

use regex::Regex;
use rustc_hash::FxHashMap;

use crate::{
    Result,
    error::ErrorKind,
    models::{Language, Model, TransportCompany},
    parsing::{
        ColumnDefinition, ExpectedType, FastRowMatcher, FileParser, ParsedValue, RowDefinition,
        RowParser,
    },
    storage::ResourceStorage,
};

pub fn parse(path: &str) -> Result<ResourceStorage<TransportCompany>> {
    log::info!("Parsing BETRIEB_DE...");
    log::info!("Parsing BETRIEB_EN...");
    log::info!("Parsing BETRIEB_FR...");
    log::info!("Parsing BETRIEB_IT...");
    const ROW_A: i32 = 1;
    const ROW_B: i32 = 2;
    const ROW_C: i32 = 3;

    #[rustfmt::skip]
    let row_parser = RowParser::new(vec![
        // This row is ignored.
        RowDefinition::new(ROW_A, Box::new(FastRowMatcher::new(7, 1, "K", true)), Vec::new()),
        // This row is used to create a TransportCompany instance.
        RowDefinition::new(ROW_B, Box::new(FastRowMatcher::new(7, 1, ":", true)), vec![
            ColumnDefinition::new(1, 5, ExpectedType::Integer32),
            ColumnDefinition::new(9, -1, ExpectedType::String),
        ]),
        // This row is used to create a TransportCompany instance from the SBOID identifier.
        RowDefinition::new(ROW_C, Box::new(FastRowMatcher::new(7, 1, "N", true)), vec![
            ColumnDefinition::new(9, -1, ExpectedType::String),
        ]),
    ]);
    let parser = FileParser::new(&format!("{path}/BETRIEB_DE"), row_parser)?;

    let data = parser
        .parse()
        .map(|x| {
            x.map(|(id, _, values)| {
                match id {
                    ROW_A => {}
                    ROW_B => return Some(create_instance(values)),
                    ROW_C => { // TODO we should probably add an explicit treatment for the sboid
                    }
                    _ => unreachable!(),
                };
                None
            })
        })
        .collect::<Result<Vec<_>>>()?;
    // If there are no errors, "None" values are removed.
    let data = data.into_iter().flatten().collect();
    let mut data = TransportCompany::vec_to_map(data);

    load_designations(path, &mut data, Language::German)?;
    load_designations(path, &mut data, Language::English)?;
    load_designations(path, &mut data, Language::French)?;
    load_designations(path, &mut data, Language::Italian)?;

    Ok(ResourceStorage::new(data))
}

fn load_designations(
    path: &str,
    data: &mut FxHashMap<i32, TransportCompany>,
    language: Language,
) -> Result<()> {
    const ROW_A: i32 = 1;
    const ROW_B: i32 = 2;
    const ROW_C: i32 = 3;

    #[rustfmt::skip]
    let row_parser = RowParser::new(vec![
        // This row contains the short name, long name and full name in a specific language.
        RowDefinition::new(ROW_A, Box::new(FastRowMatcher::new(7, 1, "K", true)), vec![
            ColumnDefinition::new(1, 5, ExpectedType::Integer32),
            ColumnDefinition::new(9, -1, ExpectedType::String),
        ]),
        // This row is ignored.
        RowDefinition::new(ROW_B, Box::new(FastRowMatcher::new(7, 1, ":", true)), Vec::new()),
        // This row is used to create a TransportCompany instance from the SBOID identifier.
        RowDefinition::new(ROW_C, Box::new(FastRowMatcher::new(7, 1, "N", true)), vec![
            ColumnDefinition::new(9, -1, ExpectedType::String),
        ]),
    ]);
    let filename = match language {
        Language::German => "BETRIEB_DE",
        Language::English => "BETRIEB_EN",
        Language::French => "BETRIEB_FR",
        Language::Italian => "BETRIEB_IT",
    };
    let parser = FileParser::new(&format!("{path}/{filename}"), row_parser)?;

    parser.parse().try_for_each(|x| {
        let (id, _, values) = x?;
        if id == ROW_A {
            set_designations(values, data, language)?
        }
        Ok(())
    })
}

// ------------------------------------------------------------------------------------------------
// --- Data Processing Functions
// ------------------------------------------------------------------------------------------------

fn create_instance(mut values: Vec<ParsedValue>) -> TransportCompany {
    let id: i32 = values.remove(0).into();
    let administrations = values.remove(0).into();

    let administrations = parse_administrations(administrations);

    TransportCompany::new(id, administrations)
}

fn set_designations(
    mut values: Vec<ParsedValue>,
    data: &mut FxHashMap<i32, TransportCompany>,
    language: Language,
) -> Result<()> {
    let id: i32 = values.remove(0).into();
    let designations = values.remove(0).into();

    let (short_name, long_name, full_name) = parse_designations(designations);

    let transport_company = data.get_mut(&id).ok_or(ErrorKind::UnknownId(id))?;
    transport_company.set_short_name(language, &short_name);
    transport_company.set_long_name(language, &long_name);
    transport_company.set_full_name(language, &full_name);

    Ok(())
}

// ------------------------------------------------------------------------------------------------
// --- Helper Functions
// ------------------------------------------------------------------------------------------------

fn parse_administrations(administrations: String) -> Vec<String> {
    administrations
        .split_whitespace()
        .map(|s| s.to_owned())
        .collect()
}

fn parse_designations(designations: String) -> (String, String, String) {
    // unwrap: The creation of this regular expression will never fail.
    let re = Regex::new(r" ?(K|L|V) ").unwrap();
    let designations: Vec<String> = re
        .split(&designations)
        .map(|s| s.chars().filter(|&c| c != '"').collect())
        .collect();

    let short_name = designations[0].to_owned();
    let long_name = designations[1].to_owned();
    let full_name = designations[2].to_owned();

    (short_name, long_name, full_name)
}
