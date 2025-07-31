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
use rustc_hash::FxHashMap;

use crate::{
    Result,
    error::ErrorKind,
    models::{InformationText, Language, Model},
    parsing::{ColumnDefinition, ExpectedType, FileParser, ParsedValue, RowDefinition, RowParser},
    storage::ResourceStorage,
};

fn id_row_parser() -> RowParser {
    RowParser::new(vec![
        // This row is used to create a InformationText instance.
        RowDefinition::from(vec![ColumnDefinition::new(1, 9, ExpectedType::Integer32)]),
    ])
}

fn id_row_converter(parser: FileParser) -> Result<FxHashMap<i32, InformationText>> {
    let data = parser
        .parse()
        .map(|x| x.map(|(_, _, values)| create_instance(values)))
        .collect::<Result<Vec<_>>>()?;
    let data = InformationText::vec_to_map(data);
    Ok(data)
}

fn infotext_row_parser() -> RowParser {
    RowParser::new(vec![
        // This row contains the content in a specific language.
        RowDefinition::from(vec![
            ColumnDefinition::new(1, 9, ExpectedType::Integer32),
            ColumnDefinition::new(11, -1, ExpectedType::String),
        ]),
    ])
}

fn infotext_row_converter(
    parser: FileParser,
    data: &mut FxHashMap<i32, InformationText>,
    language: Language,
) -> Result<()> {
    parser.parse().try_for_each(|x| {
        let (_, _, values) = x?;
        set_content(values, data, language)?;
        Ok(())
    })
}

pub fn parse(path: &str) -> Result<ResourceStorage<InformationText>> {
    log::info!("Parsing INFOTEXT_DE...");
    log::info!("Parsing INFOTEXT_EN...");
    log::info!("Parsing INFOTEXT_FR...");
    log::info!("Parsing INFOTEXT_IT...");

    let row_parser = id_row_parser();
    let parser = FileParser::new(&format!("{path}/INFOTEXT_DE"), row_parser)?;
    let mut data = id_row_converter(parser)?;

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
) -> Result<()> {
    let row_parser = infotext_row_parser();
    let filename = match language {
        Language::German => "INFOTEXT_DE",
        Language::English => "INFOTEXT_EN",
        Language::French => "INFOTEXT_FR",
        Language::Italian => "INFOTEXT_IT",
    };
    let parser = FileParser::new(&format!("{path}/{filename}"), row_parser)?;
    infotext_row_converter(parser, data, language)
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
) -> Result<()> {
    let id: i32 = values.remove(0).into();
    let description: String = values.remove(0).into();

    data.get_mut(&id)
        .ok_or(ErrorKind::UnknownId(id))?
        .set_content(language, &description);

    Ok(())
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;
    use crate::parsing::tests::get_json_values;
    use pretty_assertions::assert_eq;

    #[test]
    fn id_row_parser_v207() {
        let rows = vec![
            "000001921 ch:1:sjyid:100001:3995-001".to_string(),
            "000003459 2518".to_string(),
        ];
        let parser = FileParser {
            row_parser: id_row_parser(),
            rows,
        };
        let mut parser_iterator = parser.parse();
        let (_, _, mut parsed_values) = parser_iterator.next().unwrap().unwrap();
        let id: i32 = parsed_values.remove(0).into();
        assert_eq!(1921, id);
        let (_, _, mut parsed_values) = parser_iterator.next().unwrap().unwrap();
        let id: i32 = parsed_values.remove(0).into();
        assert_eq!(3459, id);
    }

    #[test]
    fn id_type_converter_v207() {
        let rows = vec![
            "000001921 ch:1:sjyid:100001:3995-001".to_string(),
            "000003459 2518".to_string(),
        ];
        let parser = FileParser {
            row_parser: id_row_parser(),
            rows,
        };
        let data = id_row_converter(parser).unwrap();
        // First row (id: 1)
        let attribute = data.get(&1921).unwrap();
        let reference = r#"
            {
                "id": 1921,
                "content": {}
            }"#;
        let (attribute, reference) = get_json_values(attribute, reference).unwrap();
        assert_eq!(attribute, reference);
        // Second row (id: 2)
        let attribute = data.get(&3459).unwrap();
        let reference = r#"
            {
                "id": 3459,
                "content": {}
            }"#;
        let (attribute, reference) = get_json_values(attribute, reference).unwrap();
        assert_eq!(attribute, reference);
    }

    #[test]
    fn infotext_row_parser_v207() {
        let rows = vec![
            "000001921 ch:1:sjyid:100001:3995-001".to_string(),
            "000003459 2518".to_string(),
        ];
        let parser = FileParser {
            row_parser: infotext_row_parser(),
            rows,
        };
        let mut parser_iterator = parser.parse();
        let (_, _, mut parsed_values) = parser_iterator.next().unwrap().unwrap();
        let id: i32 = parsed_values.remove(0).into();
        assert_eq!(1921, id);
        let content: String = parsed_values.remove(0).into();
        assert_eq!("ch:1:sjyid:100001:3995-001", &content);
        let (_, _, mut parsed_values) = parser_iterator.next().unwrap().unwrap();
        let id: i32 = parsed_values.remove(0).into();
        assert_eq!(3459, id);
        let content: String = parsed_values.remove(0).into();
        assert_eq!("2518", &content);
    }

    #[test]
    fn infotext_type_converter_v207() {
        let rows = vec![
            "000001921 ch:1:sjyid:100001:3995-001".to_string(),
            "000003459 2518".to_string(),
        ];
        let parser_fr = FileParser {
            row_parser: infotext_row_parser(),
            rows: rows.clone(),
        };
        let parser_en = FileParser {
            row_parser: infotext_row_parser(),
            rows: rows.clone(),
        };
        let parser_de = FileParser {
            row_parser: infotext_row_parser(),
            rows: rows.clone(),
        };
        let parser_it = FileParser {
            row_parser: infotext_row_parser(),
            rows: rows.clone(),
        };
        let parser = FileParser {
            row_parser: infotext_row_parser(),
            rows,
        };
        let mut data = id_row_converter(parser).unwrap();
        infotext_row_converter(parser_fr, &mut data, Language::French).unwrap();
        infotext_row_converter(parser_en, &mut data, Language::English).unwrap();
        infotext_row_converter(parser_de, &mut data, Language::German).unwrap();
        infotext_row_converter(parser_it, &mut data, Language::Italian).unwrap();
        // First row (id: 1)
        let attribute = data.get(&1921).unwrap();
        let reference = r#"
            {
                "id": 1921,
                "content": {
                    "French": "ch:1:sjyid:100001:3995-001",
                    "Italian": "ch:1:sjyid:100001:3995-001",
                    "German": "ch:1:sjyid:100001:3995-001",
                    "English": "ch:1:sjyid:100001:3995-001"
                }
            }"#;
        let (attribute, reference) = get_json_values(attribute, reference).unwrap();
        assert_eq!(attribute, reference);
        // Second row (id: 2)
        let attribute = data.get(&3459).unwrap();
        let reference = r#"
            {
                "id": 3459,
                "content": {
                    "French": "2518",
                    "Italian": "2518",
                    "German": "2518",
                    "English": "2518"
                }
            }"#;
        let (attribute, reference) = get_json_values(attribute, reference).unwrap();
        assert_eq!(attribute, reference);
    }
}
