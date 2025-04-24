/// # Infotext parsing
///
/// Additional information on objects (journeys, lines, etc.). This information can either be
///
/// - Be simple texts, e.g.: 000018154 Rollstühle können mit Unterstützung des Fahrpersonals befördert werden,
/// - Values with semantic meaning. This means values that cannot be represented in any other way and have therefore been “outsourced” to INFOTEXT, e.g.  000000000 ch:1:sjyid:100001:3-002
///
/// The INFOTEXTCODE attribute defines whether these are simple texts or texts with a semantic meaning.
/// The INFOTEXTCODE is not in the INFOTEXT file, but only in the INFOTEXT referencing files, e.g. FPLAN.
///
/// ## Remark
///
/// We start by parsing the INFOTEXT_DE file to get the ids of each ilne and then complement them
/// with the rest of the infotext from INFOTEXT_* for the semantic meaning part, since all
/// files have the same content from this point of view. The rest is parsed by language
///
/// 4 file(s).
/// File(s) read by the parser:
/// INFOTEXT_DE, INFOTEXT_EN, INFOTEXT_FR, INFOTEXT_IT
use std::error::Error;

use rustc_hash::FxHashMap;

use crate::{
    models::{InformationText, Language, Model},
    parsing::{ColumnDefinition, ExpectedType, FileParser, ParsedValue, RowDefinition, RowParser},
    storage::ResourceStorage,
};

pub fn parse(path: &str) -> Result<ResourceStorage<InformationText>, Box<dyn Error>> {
    log::info!("Parsing INFOTEXT_DE...");
    log::info!("Parsing INFOTEXT_EN...");
    log::info!("Parsing INFOTEXT_FR...");
    log::info!("Parsing INFOTEXT_IT...");

    #[rustfmt::skip]
    let row_parser = RowParser::new(vec![
        // This row is used to create a InformationText instance.
        RowDefinition::from(vec![
            ColumnDefinition::new(1, 9, ExpectedType::Integer32),
        ]),
    ]);
    let parser = FileParser::new(&format!("{path}/INFOTEXT_DE"), row_parser)?;

    let data = parser
        .parse()
        .map(|x| x.map(|(_, _, values)| create_instance(values)))
        .collect::<Result<Vec<_>, _>>()?;
    let mut data = InformationText::vec_to_map(data);

    load_content(path, &mut data, Language::German)?;
    load_content(path, &mut data, Language::English)?;
    load_content(path, &mut data, Language::French)?;
    load_content(path, &mut data, Language::Italian)?;

    Ok(ResourceStorage::new(data))
}

fn load_content(
    path: &str,
    data: &mut FxHashMap<i32, InformationText>,
    language: Language,
) -> Result<(), Box<dyn Error>> {
    #[rustfmt::skip]
    let row_parser = RowParser::new(vec![
        // This row contains the content in a specific language.
        RowDefinition::from(vec![
            ColumnDefinition::new(1, 9, ExpectedType::Integer32),
            ColumnDefinition::new(11, -1, ExpectedType::String),
        ]),
    ]);
    let filename = match language {
        Language::German => "INFOTEXT_DE",
        Language::English => "INFOTEXT_EN",
        Language::French => "INFOTEXT_FR",
        Language::Italian => "INFOTEXT_IT",
    };
    let parser = FileParser::new(&format!("{path}/{filename}"), row_parser)?;

    parser.parse().try_for_each(|x| {
        let (_, _, values) = x?;
        set_content(values, data, language)?;
        Ok(())
    })
}

// ------------------------------------------------------------------------------------------------
// --- Data Processing Functions
// ------------------------------------------------------------------------------------------------

fn create_instance(mut values: Vec<ParsedValue>) -> InformationText {
    let id: i32 = values.remove(0).into();

    InformationText::new(id)
}

fn set_content(
    mut values: Vec<ParsedValue>,
    data: &mut FxHashMap<i32, InformationText>,
    language: Language,
) -> Result<(), Box<dyn Error>> {
    let id: i32 = values.remove(0).into();
    let description: String = values.remove(0).into();

    data.get_mut(&id)
        .ok_or("Unknown ID")?
        .set_content(language, &description);

    Ok(())
}
