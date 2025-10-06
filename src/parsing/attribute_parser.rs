/// # Attribute parsing
///
/// List of abbreviations describing additional offers (e.g.: dining car)
/// or restrictions (e.g.: seat reservation obligatory). See [https://opentransportdata.swiss/en/cookbook/hafas-rohdaten-format-hrdf/#Technical_description_What_is_in_the_HRDF_files_contents](the documentation) for more information.
///
/// This file contains:
///
/// ## The list of offers
///
///
///
/// ### Example (excerpt):
///
/// `
/// Y  0   5  5 % The code Y applies to the journey section (0) with priority 5 and sorting 5
/// `
///
/// ## Description of how the offers can be displayed
///
/// **Important:** Currently these lines are not used in the library
///
/// ### Example (excerpt):
///
/// `
/// # Y  Y  Y  % Attribute code Y should be output as Y for partial route and as Y for full route
/// `
///
/// ## Description in the following languages : German, English, French, Italian
///
/// ## Example (excerpts):
///
/// ...
/// <text>                % Keyword pour la définition du texte
/// <deu>                 % The language becomes german
/// ...
/// Y  Zu Fuss            % Code Y, with description "Zu Fuss"
/// ...
/// <fra>                 % The language becomes French
/// ...
/// Y  A pied             % Code Y, with description "A pied"
/// ...
///
/// File(s) read by the parser:
/// ATTRIBUT
/// ---
/// Files not used by the parser vor version < 2.0.7:
/// ATTRIBUT_DE, ATTRIBUT_EN, ATTRIBUT_FR, ATTRIBUT_IT
/// These files were suppressed in 2.0.7
use std::str::FromStr;

use nom::{
    IResult, Parser,
    branch::alt,
    bytes::{tag, take_until},
    character::{char, complete::multispace1},
    sequence::{preceded, terminated},
};
use rustc_hash::FxHashMap;

use crate::{
    models::{Attribute, Language, Model},
    parsing::{
        error::{HResult, HrdfError, PResult, ParsingError},
        helpers::{
            i16_from_n_digits_parser, read_lines, string_from_n_chars_parser,
            string_till_eol_parser,
        },
    },
    storage::ResourceStorage,
    utils::AutoIncrement,
};

type AttributeAndTypeConverter = (ResourceStorage<Attribute>, FxHashMap<String, i32>);

enum AttributeLine {
    Offer {
        designation_id: String,
        stop_scope: i16,
        priority: i16,
        secondary_sorting_priority: i16,
    },
    Language(String),
    LanguageDescription {
        legacy_id: String,
        description: String,
    },
    Description(String),
}

fn row_offer_combinator(input: &str) -> IResult<&str, AttributeLine> {
    (
        string_from_n_chars_parser(2),
        preceded(char(' '), i16_from_n_digits_parser(1)),
        preceded(char(' '), i16_from_n_digits_parser(3)),
        preceded(char(' '), i16_from_n_digits_parser(2)),
    )
        .map(
            |(designation_id, stop_scope, priority, secondary_sorting_priority)| {
                AttributeLine::Offer {
                    designation_id,
                    stop_scope,
                    priority,
                    secondary_sorting_priority,
                }
            },
        )
        .parse(input)
}

fn row_language_combinator(input: &str) -> IResult<&str, AttributeLine> {
    preceded(tag("<"), terminated(take_until(">"), tag(">")))
        .map(|s| AttributeLine::Language(String::from(s)))
        .parse(input)
}

fn row_description_combinator(input: &str) -> IResult<&str, AttributeLine> {
    preceded(tag("#"), string_till_eol_parser)
        .map(AttributeLine::Description)
        .parse(input)
}

fn row_language_description_combinator(input: &str) -> IResult<&str, AttributeLine> {
    (
        string_from_n_chars_parser(2),
        multispace1,
        string_till_eol_parser,
    )
        .map(
            |(legacy_id, _, description)| AttributeLine::LanguageDescription {
                legacy_id,
                description,
            },
        )
        .parse(input)
}

fn parse_line(
    line: &str,
    data: &mut FxHashMap<i32, Attribute>,
    pk_type_converter: &mut FxHashMap<String, i32>,
    auto_increment: &AutoIncrement,
    current_language: &mut Language,
) -> PResult<()> {
    let (_, attribute_row) = alt((
        row_offer_combinator,
        row_language_combinator,
        row_language_description_combinator,
        row_description_combinator,
    ))
    .parse(line)?;

    match attribute_row {
        AttributeLine::Offer {
            designation_id,
            stop_scope,
            priority,
            secondary_sorting_priority,
        } => {
            let id = auto_increment.next();

            if let Some(previous) = pk_type_converter.insert(designation_id.to_owned(), id) {
                log::error!(
                    "Error: previous id {previous} for {designation_id}. The designation, {designation_id}, is not unique."
                );
            }
            let attribute = Attribute::new(
                id,
                designation_id.to_owned(),
                stop_scope,
                priority,
                secondary_sorting_priority,
            );
            data.insert(attribute.id(), attribute);
        }
        AttributeLine::Language(s) => {
            if s != "text" {
                *current_language = Language::from_str(&s)?;
            }
        }
        AttributeLine::LanguageDescription {
            legacy_id,
            description,
        } => {
            let id = pk_type_converter
                .get(&legacy_id)
                .ok_or_else(|| ParsingError::UnknownId(format!("legacy_id : {legacy_id}")))?;

            data.get_mut(id)
                .ok_or_else(|| ParsingError::UnknownId(format!("id : {id}")))?
                .set_description(*current_language, &description);
        }
        AttributeLine::Description(_s) => {
            // We do nothing
        }
    }

    Ok(())
}

pub fn parse(path: &str) -> HResult<AttributeAndTypeConverter> {
    log::info!("Parsing ATTRIBUT...");

    let file = format!("{path}/ATTRIBUT");
    let lines = read_lines(&file, 0)?;

    let auto_increment = AutoIncrement::new();
    let mut data = FxHashMap::default();
    let mut pk_type_converter = FxHashMap::default();
    let mut current_language = Language::default();

    lines
        .into_iter()
        .enumerate()
        .filter(|(_, line)| !line.trim().is_empty())
        .try_for_each(|(line_number, line)| {
            parse_line(
                &line,
                &mut data,
                &mut pk_type_converter,
                &auto_increment,
                &mut current_language,
            )
            .map_err(|e| HrdfError::Parsing {
                error: e,
                file: String::from(&file),
                line,
                line_number,
            })
        })?;

    Ok((ResourceStorage::new(data), pk_type_converter))
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;
    use crate::parsing::tests::get_json_values;
    use pretty_assertions::assert_eq;

    fn row_language_description_parser(input: &str) -> PResult<(String, String)> {
        let (_, ld) = row_language_description_combinator(input)?;

        match ld {
            AttributeLine::LanguageDescription {
                legacy_id,
                description,
            } => Ok((legacy_id, description)),
            _ => Err("Not a LanguageDescription".into()),
        }
    }

    #[test]
    fn language_description_row() {
        let input = "VR VELOS: Reservation obligatory";
        let (id, description) = row_language_description_parser(input).unwrap();
        assert_eq!("VR", id);
        assert_eq!("VELOS: Reservation obligatory", description);

        let input = "2  2nd class only";
        let (id, description) = row_language_description_parser(input).unwrap();
        assert_eq!("2", id);
        assert_eq!("2nd class only", description);
    }

    #[test]
    fn language_description_long_row() {
        let input = "VR  VELOS: Reservation obligatory";
        let (id, description) = row_language_description_parser(input).unwrap();
        assert_eq!("VR", id);
        assert_eq!("VELOS: Reservation obligatory", description);

        let input = "2   2nd class only";
        let (id, description) = row_language_description_parser(input).unwrap();
        assert_eq!("2", id);
        assert_eq!("2nd class only", description);
    }

    fn row_description_parser(input: &str) -> PResult<String> {
        let (_, lang) = row_description_combinator(input)?;

        match lang {
            AttributeLine::Description(s) => Ok(s),
            _ => Err("Not a Description".into()),
        }
    }

    #[test]
    fn description_row() {
        let input = "# WR WR WR";
        let description = row_description_parser(input).unwrap();
        assert_eq!("WR WR WR", description);
    }

    fn row_offer_parser(input: &str) -> PResult<(String, i16, i16, i16)> {
        let (_, line) = row_offer_combinator(input)?;
        match line {
            AttributeLine::Offer {
                designation_id,
                stop_scope,
                priority,
                secondary_sorting_priority,
            } => Ok((
                designation_id,
                stop_scope,
                priority,
                secondary_sorting_priority,
            )),
            _ => Err("Not an Offer".into()),
        }
    }

    #[test]
    fn offer_row() {
        let input = "PR 0   4  5";
        let (id, journey_section, priority, sorting) = row_offer_parser(input).unwrap();
        assert_eq!("PR", id);
        assert_eq!(0, journey_section);
        assert_eq!(4, priority);
        assert_eq!(5, sorting);
    }

    fn row_language_parser(input: &str) -> PResult<String> {
        let (_, line) = row_language_combinator(input)?;

        match line {
            AttributeLine::Language(language) => Ok(language),
            _ => Err("Not a Language".into()),
        }
    }

    #[test]
    fn language_row() {
        let input = "<text>";
        let language = row_language_parser(input).unwrap();
        assert_eq!("text", language);

        let input = "<fre>";
        let language = row_language_parser(input).unwrap();
        assert_eq!("fre", language);
    }

    #[test]
    fn muti_line_parsing() {
        let rows = vec![
            "GK 0   4  5".to_string(),
            "# PG PG PG".to_string(),
            "<deu>".to_string(),
            "GK  Zollkontrolle möglich, mehr Zeit einrechnen".to_string(),
            "<fra>".to_string(),
            "GK  Contrôle douanier possible, prévoir davantage de temps".to_string(),
            "<ita>".to_string(),
            "GK  Possibile controllo doganale, prevedere più tempo".to_string(),
            "<eng>".to_string(),
            "GK  Possible customs check, please allow extra time".to_string(),
        ];

        let auto_increment = AutoIncrement::new();
        let mut data = FxHashMap::default();
        let mut pk_type_converter = FxHashMap::default();
        let mut current_language = Language::default();

        rows.into_iter()
            .filter(|line| !line.trim().is_empty())
            .try_for_each(|line| {
                parse_line(
                    &line,
                    &mut data,
                    &mut pk_type_converter,
                    &auto_increment,
                    &mut current_language,
                )
            })
            .unwrap();

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
}
