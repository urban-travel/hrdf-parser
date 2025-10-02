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
use std::{error::Error, str::FromStr};

use nom::{IResult, Parser, character::char, sequence::separated_pair};
use rustc_hash::FxHashMap;

use crate::{
    models::{InformationText, Language},
    parsing::helpers::{i32_from_n_digits_parser, read_lines, string_till_eol_parser},
    storage::ResourceStorage,
};

fn parse_infotext_row(input: &str) -> IResult<&str, (i32, String)> {
    separated_pair(
        i32_from_n_digits_parser(9),
        char(' '),
        string_till_eol_parser,
    )
    .parse(input)
}

fn parse_line(
    line: &str,
    infotextmap: &mut FxHashMap<i32, InformationText>,
    current_language: Language,
) -> Result<(), Box<dyn Error>> {
    let (_, (id, infotext)) =
        parse_infotext_row(line).map_err(|e| format!("Failed to parse line '{}': {}", line, e))?;
    if let Some(mut info) = infotextmap.remove(&id) {
        info.set_content(current_language, &infotext);
        infotextmap.insert(id, info);
    } else {
        let mut info = InformationText::new(id);
        info.set_content(current_language, &infotext);
        infotextmap.insert(id, info);
    }
    Ok::<(), Box<dyn Error>>(())
}

pub fn parse(path: &str) -> Result<ResourceStorage<InformationText>, Box<dyn Error>> {
    let mut infotextmap: FxHashMap<i32, InformationText> = FxHashMap::default();
    let languages = ["DE", "EN", "FR", "IT"];
    for language in languages {
        log::info!("Parsing INFOTEXT_{language}...");

        let lines = read_lines(&format!("{path}/INFOTEXT_{language}"), 0)?;
        let current_language = Language::from_str(language)?;
        lines
            .into_iter()
            .filter(|line| !line.trim().is_empty())
            .try_for_each(|line| parse_line(&line, &mut infotextmap, current_language))?;
    }
    Ok(ResourceStorage::new(infotextmap))
}

// ------------------------------------------------------------------------------------------------
// --- Data Processing Functions
// ------------------------------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use crate::parsing::tests::get_json_values;

    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn infotext_row_parser1() {
        let input = "000001921 ch:1:sjyid:100001:3995-001";
        let (_, (id, infotext)) = parse_infotext_row(input).unwrap();
        assert_eq!(1921, id);
        assert_eq!("ch:1:sjyid:100001:3995-001", &infotext);
    }

    #[test]
    fn infotext_row_parser2() {
        let input = "000003459 2518 ";
        let (_, (id, infotext)) = parse_infotext_row(input).unwrap();
        assert_eq!(3459, id);
        assert_eq!("2518", &infotext);
    }

    #[test]
    fn parse_and_transform_infotext() {
        let input = "000001921 ch:1:sjyid:100001:3995-001";
        // First row (id: 1)
        let mut infotext_map = FxHashMap::default();
        parse_line(input, &mut infotext_map, Language::German).unwrap();
        parse_line(input, &mut infotext_map, Language::French).unwrap();
        parse_line(input, &mut infotext_map, Language::Italian).unwrap();
        parse_line(input, &mut infotext_map, Language::English).unwrap();
        println!("{infotext_map:?}");
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
        let (attribute, reference) =
            get_json_values(infotext_map.get(&1921).unwrap(), reference).unwrap();
        assert_eq!(attribute, reference);
    }
}
