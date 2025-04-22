// 5 file(s).
// File(s) read by the parser:
// ATTRIBUT
// ---
// Files not used by the parser:
// ATTRIBUT_DE, ATTRIBUT_EN, ATTRIBUT_FR, ATTRIBUT_IT
use std::{error::Error, str::FromStr};

use rustc_hash::FxHashMap;
use serde::Serialize;

use crate::{
    Version,
    models::{Attribute, Language, Model},
    parsing::{
        AdvancedRowMatcher, ColumnDefinition, ExpectedType, FastRowMatcher, FileParser,
        ParsedValue, RowDefinition, RowParser,
    },
    storage::ResourceStorage,
    utils::AutoIncrement,
};

type AttributeAndTypeConverter = (ResourceStorage<Attribute>, FxHashMap<String, i32>);
type FxHashMapsAndTypeConverter = (FxHashMap<i32, Attribute>, FxHashMap<String, i32>);

enum RowType {
    RowA = 1,
    RowB = 2,
    RowC = 3,
    RowD = 4,
}

fn attribute_row_parser(version: Version) -> Result<RowParser, Box<dyn Error>> {
    let row_parser = RowParser::new(vec![
        // This row is used to create an Attribute instance.
        RowDefinition::new(
            RowType::RowA as i32,
            Box::new(AdvancedRowMatcher::new(
                r"^.{2} [0-9] [0-9 ]{3} [0-9 ]{2}$",
            )?),
            vec![
                ColumnDefinition::new(1, 2, ExpectedType::String),
                ColumnDefinition::new(4, 4, ExpectedType::Integer16),
                ColumnDefinition::new(6, 8, ExpectedType::Integer16),
                ColumnDefinition::new(10, 11, ExpectedType::Integer16),
            ],
        ),
        // This row is ignored.
        RowDefinition::new(
            RowType::RowB as i32,
            Box::new(FastRowMatcher::new(1, 1, "#", true)),
            vec![ColumnDefinition::new(1, -1, ExpectedType::String)],
        ),
        // This row indicates the language for translations in the section that follows it.
        RowDefinition::new(
            RowType::RowC as i32,
            Box::new(FastRowMatcher::new(1, 1, "<", true)),
            vec![ColumnDefinition::new(1, -1, ExpectedType::String)],
        ),
        // This row contains the description in a specific language.
        // The format changed in V 2.0.7 and now the description starts at column 5 instead of 4
        RowDefinition::new(
            RowType::RowD as i32,
            Box::new(AdvancedRowMatcher::new(r"^.{2} .+$")?),
            vec![
                ColumnDefinition::new(1, 2, ExpectedType::String),
                match version {
                    Version::V_5_40_41_2_0_4
                    | Version::V_5_40_41_2_0_5
                    | Version::V_5_40_41_2_0_6 => {
                        ColumnDefinition::new(4, -1, ExpectedType::String)
                    }
                    Version::V_5_40_41_2_0_7 => ColumnDefinition::new(5, -1, ExpectedType::String),
                },
            ],
        ),
    ]);
    Ok(row_parser)
}

fn convert_data_strcutures(
    parser: FileParser,
) -> Result<FxHashMapsAndTypeConverter, Box<dyn Error>> {
    let auto_increment = AutoIncrement::new();
    let mut data = FxHashMap::default();
    let mut pk_type_converter = FxHashMap::default();

    let mut current_language = Language::default();

    for x in parser.parse() {
        let (id, _, values) = x?;
        if id == RowType::RowA as i32 {
            let attribute = create_instance(values, &auto_increment, &mut pk_type_converter);
            data.insert(attribute.id(), attribute);
        } else if id == RowType::RowB as i32 {
            // We discard lines starting with #
        } else if id == RowType::RowC as i32 {
            update_current_language(values, &mut current_language)?;
        } else if id == RowType::RowD as i32 {
            set_description(values, &pk_type_converter, &mut data, current_language)?;
        } else {
            unreachable!()
        }
    }
    Ok((data, pk_type_converter))
}

pub fn parse(version: Version, path: &str) -> Result<AttributeAndTypeConverter, Box<dyn Error>> {
    log::info!("Parsing ATTRIBUT...");
    let row_parser = attribute_row_parser(version)?;
    // The ATTRIBUT file is used instead of ATTRIBUT_* for simplicity's sake.
    let parser = FileParser::new(&format!("{path}/ATTRIBUT"), row_parser)?;
    let (data, pk_type_converter) = convert_data_strcutures(parser)?;
    Ok((ResourceStorage::new(data), pk_type_converter))
}

// ------------------------------------------------------------------------------------------------
// --- Data Processing Functions
// ------------------------------------------------------------------------------------------------

fn create_instance(
    mut values: Vec<ParsedValue>,
    auto_increment: &AutoIncrement,
    pk_type_converter: &mut FxHashMap<String, i32>,
) -> Attribute {
    let designation: String = values.remove(0).into();
    let stop_scope: i16 = values.remove(0).into();
    let main_sorting_priority: i16 = values.remove(0).into();
    let secondary_sorting_priority: i16 = values.remove(0).into();

    let id = auto_increment.next();

    pk_type_converter.insert(designation.to_owned(), id);
    Attribute::new(
        id,
        designation.to_owned(),
        stop_scope,
        main_sorting_priority,
        secondary_sorting_priority,
    )
}

fn set_description(
    mut values: Vec<ParsedValue>,
    pk_type_converter: &FxHashMap<String, i32>,
    data: &mut FxHashMap<i32, Attribute>,
    language: Language,
) -> Result<(), Box<dyn Error>> {
    let legacy_id: String = values.remove(0).into();
    let description: String = values.remove(0).into();

    let id = pk_type_converter
        .get(&legacy_id)
        .ok_or("Unknown legacy ID")?;
    data.get_mut(id)
        .ok_or("Unknown ID")?
        .set_description(language, &description);

    Ok(())
}

// ------------------------------------------------------------------------------------------------
// --- Helper Functions
// ------------------------------------------------------------------------------------------------

fn update_current_language(
    mut values: Vec<ParsedValue>,
    current_language: &mut Language,
) -> Result<(), Box<dyn Error>> {
    let language: String = values.remove(0).into();
    let language = language.replace(['<', '>'], "");

    if language != "text" {
        *current_language = Language::from_str(&language)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;
    use pretty_assertions::assert_eq;
    use serde::Deserialize;

    fn get_json_values<F>(
        lhs: &F,
        rhs: &str,
    ) -> Result<(serde_json::Value, serde_json::Value), Box<dyn Error>>
    where
        for<'a> F: Serialize + Deserialize<'a>,
    {
        let serialized = serde_json::to_string(&lhs)?;
        let reference = serde_json::to_string(&serde_json::from_str::<F>(rhs)?)?;
        Ok((
            serialized.parse::<serde_json::Value>()?,
            reference.parse::<serde_json::Value>()?,
        ))
    }

    #[test]
    fn description_row_d_v206() {
        let rows = vec![
            "VR VELOS: Reservation obligatory".to_string(),
            "2  2nd class only".to_string(),
        ];
        let parser = FileParser {
            row_parser: attribute_row_parser(Version::V_5_40_41_2_0_6).unwrap(),
            rows,
        };
        let mut parser_iterator = parser.parse();
        let (id, _, mut parsed_values) = parser_iterator.next().unwrap().unwrap();
        assert_eq!(id, RowType::RowD as i32);
        let legacy_id: String = parsed_values.remove(0).into();
        assert_eq!("VR", &legacy_id);
        let description: String = parsed_values.remove(0).into();
        assert_eq!("VELOS: Reservation obligatory", &description);
        let (id, _, mut parsed_values) = parser_iterator.next().unwrap().unwrap();
        assert_eq!(id, RowType::RowD as i32);
        let legacy_id: String = parsed_values.remove(0).into();
        assert_eq!("2", &legacy_id);
        let description: String = parsed_values.remove(0).into();
        assert_eq!("2nd class only", &description);
    }

    #[test]
    fn parser_row_d_v207() {
        let rows = vec![
            "VR  VELOS: Reservation obligatory".to_string(),
            "2   2nd class only".to_string(),
        ];
        let parser = FileParser {
            row_parser: attribute_row_parser(Version::V_5_40_41_2_0_7).unwrap(),
            rows,
        };
        let mut parser_iterator = parser.parse();
        let (id, _, mut parsed_values) = parser_iterator.next().unwrap().unwrap();
        assert_eq!(id, RowType::RowD as i32);
        let legacy_id: String = parsed_values.remove(0).into();
        assert_eq!("VR", &legacy_id);
        let description: String = parsed_values.remove(0).into();
        assert_eq!("VELOS: Reservation obligatory", &description);
        let (id, _, mut parsed_values) = parser_iterator.next().unwrap().unwrap();
        assert_eq!(id, RowType::RowD as i32);
        let legacy_id: String = parsed_values.remove(0).into();
        assert_eq!("2", &legacy_id);
        let description: String = parsed_values.remove(0).into();
        assert_eq!("2nd class only", &description);
    }

    #[test]
    fn parser_row_a_v207() {
        let rows = vec!["1  0   1  5".to_string(), "GR 0   6  3".to_string()];
        let parser = FileParser {
            row_parser: attribute_row_parser(Version::V_5_40_41_2_0_7).unwrap(),
            rows,
        };
        let mut parser_iterator = parser.parse();
        let (id, _, mut parsed_values) = parser_iterator.next().unwrap().unwrap();
        assert_eq!(id, RowType::RowA as i32);
        let legacy_id: String = parsed_values.remove(0).into();
        assert_eq!("1", &legacy_id);
        let num: i16 = parsed_values.remove(0).into();
        assert_eq!(0, num);
        let num: i16 = parsed_values.remove(0).into();
        assert_eq!(1, num);
        let num: i16 = parsed_values.remove(0).into();
        assert_eq!(5, num);
        let (id, _, mut parsed_values) = parser_iterator.next().unwrap().unwrap();
        assert_eq!(id, RowType::RowA as i32);
        let legacy_id: String = parsed_values.remove(0).into();
        assert_eq!("GR", &legacy_id);
        let num: i16 = parsed_values.remove(0).into();
        assert_eq!(0, num);
        let num: i16 = parsed_values.remove(0).into();
        assert_eq!(6, num);
        let num: i16 = parsed_values.remove(0).into();
        assert_eq!(3, num);
    }

    #[test]
    fn type_converter_row_a_v207() {
        let rows = vec![
            "GK 0   4  5".to_string(),
            "<deu>".to_string(),
            "GK  Zollkontrolle möglich, mehr Zeit einrechnen".to_string(),
            "<fra>".to_string(),
            "GK  Contrôle douanier possible, prévoir davantage de temps".to_string(),
            "<ita>".to_string(),
            "GK  Possibile controllo doganale, prevedere più tempo".to_string(),
            "<eng>".to_string(),
            "GK  Possible customs check, please allow extra time".to_string(),
        ];
        let parser = FileParser {
            row_parser: attribute_row_parser(Version::V_5_40_41_2_0_7).unwrap(),
            rows,
        };
        let (data, pk_type_converter) = convert_data_strcutures(parser).unwrap();
        assert_eq!(*pk_type_converter.get("GK").unwrap(), 1);
        let attribute = data.get(&1).unwrap();
        let reference = r#"
            {
                "id":1,
                "designation":"GK",
                "stop_scope":0,
                "main_sorting_priority":4,
                "secondary_sorting_priority":5,
                "description":{
                    "German":"Zollkontrolle möglich, mehr Zeit einrechnen",
                    "English":"Possible customs check, please allow extra time",
                    "French":"Contrôle douanier possible, prévoir davantage de temps",
                    "Italian":"Possibile controllo doganale, prevedere più tempo"
                }
            }"#;
        let (attribute, reference) = get_json_values(attribute, reference).unwrap();
        assert_eq!(attribute, reference);
    }

    #[test]
    fn parser_row_b_v207() {
        let rows = vec!["# PG PG PG".to_string()];
        let parser = FileParser {
            row_parser: attribute_row_parser(Version::V_5_40_41_2_0_7).unwrap(),
            rows,
        };
        let mut parser_iterator = parser.parse();
        let (id, _, mut parsed_values) = parser_iterator.next().unwrap().unwrap();
        assert_eq!(id, RowType::RowB as i32);
        let description: String = parsed_values.remove(0).into();
        assert_eq!(&description, "# PG PG PG");
    }

    #[test]
    fn parser_row_c_v207() {
        let rows = vec![
            "<ita>".to_string(),
            "<fra>".to_string(),
            "<deu>".to_string(),
            "<eng>".to_string(),
            "<text>".to_string(),
        ];
        let parser = FileParser {
            row_parser: attribute_row_parser(Version::V_5_40_41_2_0_7).unwrap(),
            rows,
        };
        let mut parser_iterator = parser.parse();
        let (id, _, mut parsed_values) = parser_iterator.next().unwrap().unwrap();
        assert_eq!(id, RowType::RowC as i32);
        let lang: String = parsed_values.remove(0).into();
        assert_eq!(&lang, "<ita>");

        let (id, _, parsed_values) = parser_iterator.next().unwrap().unwrap();
        assert_eq!(id, RowType::RowC as i32);
        let mut current_language = Language::default();
        update_current_language(parsed_values, &mut current_language).unwrap();
        assert_eq!(current_language, Language::French);
        let (id, _, parsed_values) = parser_iterator.next().unwrap().unwrap();
        assert_eq!(id, RowType::RowC as i32);
        update_current_language(parsed_values, &mut current_language).unwrap();
        assert_eq!(current_language, Language::German);
        let (id, _, parsed_values) = parser_iterator.next().unwrap().unwrap();
        assert_eq!(id, RowType::RowC as i32);
        update_current_language(parsed_values, &mut current_language).unwrap();
        assert_eq!(current_language, Language::English);
        let (id, _, parsed_values) = parser_iterator.next().unwrap().unwrap();
        assert_eq!(id, RowType::RowC as i32);
        update_current_language(parsed_values, &mut current_language).unwrap();
        assert_eq!(current_language, Language::English);
    }
}
