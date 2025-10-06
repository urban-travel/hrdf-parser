/// # Holiday parsing
///
/// For more information see
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
use nom::{IResult, Parser, character::char, sequence::separated_pair};
use rustc_hash::FxHashMap;

use crate::{
    error::{HResult, HrdfError},
    models::{Holiday, Language},
    parsing::{
        error::{PResult, ParsingError},
        helpers::{read_lines, string_from_n_chars_parser, string_till_eol_parser},
    },
    storage::ResourceStorage,
    utils::AutoIncrement,
};

fn parse_holiday_row(input: &str) -> IResult<&str, (String, String)> {
    separated_pair(
        string_from_n_chars_parser(10),
        char(' '),
        string_till_eol_parser,
    )
    .parse(input)
}

fn parse_line(line: &str, auto_increment: &AutoIncrement) -> PResult<(i32, Holiday)> {
    let (_, (date, translations)) = parse_holiday_row(line)?;

    let date = NaiveDate::parse_from_str(&date, "%d.%m.%Y")?;
    let name = parse_name_translations(translations)?;
    let id = auto_increment.next();

    Ok((id, Holiday::new(id, date, name)))
}

pub fn parse(path: &str) -> HResult<ResourceStorage<Holiday>> {
    log::info!("Parsing FEIERTAG...");
    let file = format!("{path}/FEIERTAG");
    let lines = read_lines(&file, 0)?;
    let auto_increment = AutoIncrement::new();
    let holidays = lines
        .into_iter()
        .enumerate()
        .filter(|(_, line)| !line.trim().is_empty())
        .map(|(line_number, line)| {
            parse_line(&line, &auto_increment).map_err(|e| HrdfError::Parsing {
                error: e,
                file: String::from(&file),
                line,
                line_number,
            })
        })
        .collect::<HResult<FxHashMap<_, _>>>()?;
    Ok(ResourceStorage::new(holidays))
}

// ------------------------------------------------------------------------------------------------
// --- Helper Functions
// ------------------------------------------------------------------------------------------------

fn parse_name_translations(name_translations: String) -> PResult<FxHashMap<Language, String>> {
    name_translations
        .split('>')
        .filter(|&s| !s.is_empty())
        .map(|s| -> PResult<(Language, String)> {
            let mut parts = s.split('<');

            let v = parts
                .next()
                .ok_or(ParsingError::Unknown("Missing value part".to_string()))?
                .to_string();
            let k = parts
                .next()
                .ok_or(ParsingError::Unknown("Missing value part".to_string()))?
                .to_string();
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
        let input = "25.12.2024 Weihnachtstag<deu>Noël<fra>Natale<ita>Christmas Day<eng>";
        let (_, (date, translations)) = parse_holiday_row(input).unwrap();
        assert_eq!("25.12.2024", &date);
        assert_eq!(
            "Weihnachtstag<deu>Noël<fra>Natale<ita>Christmas Day<eng>",
            &translations
        );
    }

    #[test]
    fn row_converter_v207() {
        let auto_increment = AutoIncrement::new();
        let input = "25.12.2024 Weihnachtstag<deu>Noël<fra>Natale<ita>Christmas Day<eng>";
        let (_, instance) = parse_line(input, &auto_increment).unwrap();

        // First row (id: 1)
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
        let (attribute, reference) = get_json_values(&instance, reference).unwrap();
        assert_eq!(attribute, reference);
    }
}
