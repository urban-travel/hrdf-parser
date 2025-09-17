/// # Attribute parsing
///
/// List of abbreviations describing additional offers (e.g.: dining car)
/// or restrictions (e.g.: seat reservation obligatory). See [https://opentransportdata.swiss/en/cookbook/hafas-rohdaten-format-hrdf/#Technical_description_What_is_in_the_HRDF_files_contents](the documentaion) for more informations.
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
use std::{cell::RefCell, error::Error, rc::Rc, str::FromStr};

use nom::{
    Parser,
    branch::alt,
    bytes::{tag, take_until},
    character::{char, complete::multispace1},
    combinator::{map, map_res},
    sequence::{preceded, terminated},
};
use rustc_hash::FxHashMap;

use crate::{
    models::{Attribute, Language, Model},
    parsing::helpers::{
        i16_from_n_digits_parser, read_lines, string_from_n_chars_parser, string_till_eol_parser,
    },
    storage::ResourceStorage,
    utils::AutoIncrement,
};

type AttributeAndTypeConverter = (ResourceStorage<Attribute>, FxHashMap<String, i32>);

fn row_offer_combinator<'a>() -> impl Parser<
    &'a str,
    Output = (String, char, i16, char, i16, char, i16),
    Error = nom::error::Error<&'a str>,
> {
    (
        string_from_n_chars_parser(2),
        char(' '),
        i16_from_n_digits_parser(1),
        char(' '),
        i16_from_n_digits_parser(3),
        char(' '),
        i16_from_n_digits_parser(2),
    )
}

fn row_language_combinator<'a>()
-> impl Parser<&'a str, Output = String, Error = nom::error::Error<&'a str>> {
    map(
        preceded(tag("<"), terminated(take_until(">"), tag(">"))),
        String::from,
    )
}

fn row_description_combinator<'a>()
-> impl Parser<&'a str, Output = String, Error = nom::error::Error<&'a str>> {
    preceded(tag("#"), string_till_eol_parser())
}

fn row_language_description_combinator<'a>()
-> impl Parser<&'a str, Output = (String, &'a str, String), Error = nom::error::Error<&'a str>> {
    (
        string_from_n_chars_parser(2),
        multispace1,
        string_till_eol_parser(),
    )
}

fn parse_line(
    line: &str,
    data: Rc<RefCell<FxHashMap<i32, Attribute>>>,
    pk_type_converter: Rc<RefCell<FxHashMap<String, i32>>>,
    auto_increment: &AutoIncrement,
    current_language: Rc<RefCell<Language>>,
) -> Result<(), Box<dyn Error>> {
    let _ = alt((
        map_res(
            row_language_combinator(),
            |language| {
                if language != "text" {
                    log::info!("changing language to: {language}");
                    *current_language.borrow_mut() = Language::from_str(&language)?;
                    log::info!("current_language = {}", *current_language.borrow());
                }
                Ok::<(), Box<dyn Error>>(())
            },
        ),
        map_res(
            row_offer_combinator(),
            |(designation, _, stop_scope, _, priority, _, secondary_sorting_priority)| {
                let local_data = Rc::clone(&data);
                let id = auto_increment.next();

                if let Some(previous) = pk_type_converter.borrow_mut().insert(designation.to_owned(), id) {
                    log::error!(
                        "Error: previous id {previous} for {designation}. The designation, {designation}, is not unique."
                    );
                }

                let attribute = Attribute::new(
                    id,
                    designation.to_owned(),
                    stop_scope,
                    priority,
                    secondary_sorting_priority,
                );
                local_data
                    .borrow_mut()
                .insert(attribute.id(), attribute);

                Ok::<(), Box<dyn Error>>(())
            },
        ),
        map_res(
            row_description_combinator(),
            |_description| {
                // We do nothing the # starting row is ignord for now
                // TODO: Update maybe
                Ok::<(), Box<dyn Error>>(())
            },
        ),
        map_res(
            row_language_description_combinator(),
            |(legacy_id, _, description)| {
                let local_pk = pk_type_converter.borrow();
                let id = local_pk
                    .get(&legacy_id)
                    .ok_or("Unknown legacy ID")?;

                log::info!("{}", *current_language.borrow());

                data.borrow_mut().get_mut(id)
                    .ok_or("Unknown ID")?
                    .set_description(*current_language.borrow(), &description);

                Ok::<(), Box<dyn Error>>(())
            },
        ),
    ))
    .parse(line)
    .map_err(|e| format!("Failed to parse line '{}': {}", line, e))?;
    Ok::<(), Box<dyn Error>>(())
}

pub fn parse(path: &str) -> Result<AttributeAndTypeConverter, Box<dyn Error>> {
    log::info!("Parsing ATTRIBUT...");

    let lines = read_lines(&format!("{path}/ATTRIBUT"), 0)?;

    let auto_increment = AutoIncrement::new();
    let data = Rc::new(RefCell::new(FxHashMap::default()));
    let pk_type_converter = Rc::new(RefCell::new(FxHashMap::default()));
    let current_language = Rc::new(RefCell::new(Language::default()));

    lines
        .into_iter()
        .filter(|line| !line.trim().is_empty())
        .try_for_each(|line| {
            parse_line(
                &line,
                data.clone(),
                pk_type_converter.clone(),
                &auto_increment,
                Rc::clone(&current_language),
            )
        })?;

    let data = RefCell::<FxHashMap<i32, Attribute>>::into_inner(
        Rc::into_inner(data).ok_or("Unable to get data")?,
    );
    let pk_type_converter = RefCell::<FxHashMap<String, i32>>::into_inner(
        Rc::into_inner(pk_type_converter).ok_or("Unable to get pk_type_converter")?,
    );

    Ok((ResourceStorage::new(data), pk_type_converter))
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;
    use crate::parsing::tests::get_json_values;
    use nom::IResult;
    use pretty_assertions::assert_eq;

    fn row_language_description_parser(input: &str) -> IResult<&str, (String, String)> {
        let (res, (id, _, description)) = row_language_description_combinator().parse(input)?;
        Ok((res, (id, description)))
    }

    #[test]
    fn language_description_row() {
        let input = "VR VELOS: Reservation obligatory";
        let (_, (id, description)) = row_language_description_parser(input).unwrap();
        assert_eq!("VR", id);
        assert_eq!("VELOS: Reservation obligatory", description);

        let input = "2  2nd class only";
        let (_, (id, description)) = row_language_description_parser(input).unwrap();
        assert_eq!("2", id);
        assert_eq!("2nd class only", description);
    }

    #[test]
    fn language_description_long_row() {
        let input = "VR  VELOS: Reservation obligatory";
        let (_, (id, description)) = row_language_description_parser(input).unwrap();
        assert_eq!("VR", id);
        assert_eq!("VELOS: Reservation obligatory", description);

        let input = "2   2nd class only";
        let (_, (id, description)) = row_language_description_parser(input).unwrap();
        assert_eq!("2", id);
        assert_eq!("2nd class only", description);
    }

    fn row_description_parser(input: &str) -> IResult<&str, String> {
        let (res, description) = row_description_combinator().parse(input)?;
        Ok((res, description))
    }

    #[test]
    fn description_row() {
        let input = "# WR WR WR";
        let (_, description) = row_description_parser(input).unwrap();
        assert_eq!("WR WR WR", description);
    }

    fn row_offer_parser(input: &str) -> IResult<&str, (String, i16, i16, i16)> {
        let (res, (id, _, journey_section, _, priority, _, sorting)) =
            row_offer_combinator().parse(input).unwrap();
        Ok((res, (id, journey_section, priority, sorting)))
    }

    #[test]
    fn offer_row() {
        let input = "PR 0   4  5";
        let (_, (id, journey_section, priority, sorting)) = row_offer_parser(input).unwrap();
        assert_eq!("PR", id);
        assert_eq!(0, journey_section);
        assert_eq!(4, priority);
        assert_eq!(5, sorting);
    }

    fn row_language_parser(input: &str) -> IResult<&str, String> {
        let (res, language) = row_language_combinator().parse(input).unwrap();
        Ok((res, language))
    }

    #[test]
    fn language_row() {
        let input = "<text>";
        let (_, language) = row_language_parser(input).unwrap();
        assert_eq!("text", language);

        let input = "<fre>";
        let (_, language) = row_language_parser(input).unwrap();
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
        let data = Rc::new(RefCell::new(FxHashMap::default()));
        let pk_type_converter = Rc::new(RefCell::new(FxHashMap::default()));
        let current_language = Rc::new(RefCell::new(Language::default()));

        rows.into_iter()
            .filter(|line| !line.trim().is_empty())
            .try_for_each(|line| {
                parse_line(
                    &line,
                    data.clone(),
                    pk_type_converter.clone(),
                    &auto_increment,
                    Rc::clone(&current_language),
                )
            })
            .unwrap();

        let data = RefCell::<FxHashMap<i32, Attribute>>::into_inner(
            Rc::into_inner(data).ok_or("Unable to get data").unwrap(),
        );
        let pk_type_converter = RefCell::<FxHashMap<String, i32>>::into_inner(
            Rc::into_inner(pk_type_converter)
                .ok_or("Unable to get pk_type_converter")
                .unwrap(),
        );
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
