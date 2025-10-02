/// # ZUGART file
///
/// List of service categories. Per language the (Class:) grouping of offer
/// categories with identical characteristics. (Option:) Search criteria
/// for the application for connection search. (Categorie:) Designation
/// of the offer category.
///
/// Note again: The term “Angebotskategorie* (offer category)
/// may have a different meaning here than in colloquial language!
/// A colloquial term (also according to the HRDF doc.)
/// would be “means of transport” (type).
///
/// This file is modified in Switzerland:
///
/// - Offer category definition (or generic definition):
///     - Offer category code/class code
///     - Category Product class
///     - Tariff group (always A)
///     - Output control (always 0)
///     - Generic name
///     - Surcharge (always 0)
///     - Flag (N: local transport, B: ship)
///     - Reference to category, see below.
///
/// ## Example (excerpt):
///
/// `
/// ...
/// IC  1 A 0 IC  0   #014 % Code "IC",  Kategorie 1, Tarifgruppe A, Ausgabesteuerung 0, Gattungsbezeichnung IC,  Zuschlag 0
/// ICE 0 A 0 ICE 0   #015 % Code "ICE", Kategorie 0, Tarifgruppe A, Ausgabesteuerung 0, Gattungsbezeichnung ICE, Zuschlag 0
/// ...
/// RUB 6 A 0 RUB 0 B #026 % Code "RUB", Kategorie 6, Tarifgruppe A, Ausgabesteuerung 0, Gattungsbezeichnung RUB, Zuschlag 0, Flag B (Schiff)
/// ...
/// `
///
/// - Introduction Text definition with <text>
/// - Specify language with e.g. <German>
/// - Product classes:
///     - Product class Number between 0-13
///     - Product text
///
/// ## Example (excerpt):
///
/// `
/// ...
/// <text>                                                  % Keyword für Textdefinition
/// <Deutsch>                                               % Sprache ist Deutsch
/// class00 ICE/EN/CNL/ES/NZ/TGV/THA/X2                     % Produktklasse 00 steht für ICE, EN, usw.
/// class01 EuroCity/InterCity/ICN/InterCityNight/SuperCity % Produktklasse 01 steht für EuroCity, InterCity, usw.
/// class02 InterRegio/PanoramaExpress                      % Produktklasse 02 steht für InterRegio, PanoramaExpress
/// ...
/// `
///
/// - Options:
///     - Option definition Number between 10-14 (Further details on this topic and the implementation in Switzerland can be found in the RV)
///
/// ## Example (excerpt):
///
/// `
/// ...
/// option10 nur Direktverbindungen  % Option 10 steht für nur Direktverbindungen
/// option11 Direkt mit Schlafwagen* % Option 10 steht für Direkt mit Schlafwagen
/// option12 Direkt mit Liegewagen*  % Option 10 steht für Liegewagen
/// ...
/// `
///
/// - Categories:
///     - Generic long name number Number between 0-999 (see above)
///
/// ## Example (excerpt):
///
/// `
/// ...
/// category014 InterCity        % Kategorie 14 steht für InterCity
/// category015 InterCityExpress % Kategorie 15 steht für InterCityExpress
/// ...
/// category026 Rufbus           % Kategorie 26 steht für Rufbus
/// ...
/// `
///
/// 1 file(s).
/// File(s) read by the parser:
/// ZUGART
use std::error::Error;

use nom::{
    IResult, Parser,
    branch::alt,
    bytes::complete::{tag, take_until1},
    character::complete::{char, i16, space1},
    combinator::map,
    sequence::{preceded, terminated},
};
use rustc_hash::FxHashMap;

use crate::{
    models::{Language, Model, TransportType},
    parsing::helpers::{
        optional_i32_from_n_digits_parser, read_lines, string_from_n_chars_parser,
        string_till_eol_parser,
    },
    storage::ResourceStorage,
    utils::AutoIncrement,
};

type TransportTypeAndTypeConverter = (ResourceStorage<TransportType>, FxHashMap<String, i32>);

enum TransportTypeAndTypeLine {
    OfferDefinition {
        designation: String,
        product_class_id: i16,
        tariff_group: String,
        output_control: i16,
        short_name: String,
        surcharge: i16,
        flag: String,
    },
    LanguageDefinition(String),
    Class {
        product_class_id: i16,
        product_class_name: String,
    },
    Category {
        #[allow(unused)]
        category_id: i16,
        category_name: String,
    },
    Option {
        #[allow(unused)]
        option_id: i16,
        #[allow(unused)]
        option_name: String,
    },
    Information {
        #[allow(unused)]
        code_name: String,
        #[allow(unused)]
        id: Option<i32>,
    },
}

fn offer_definition_combinator(input: &str) -> IResult<&str, TransportTypeAndTypeLine> {
    map(
        (
            string_from_n_chars_parser(3),
            preceded(space1, i16),
            preceded(space1, string_from_n_chars_parser(1)),
            preceded(space1, i16),
            preceded(space1, string_from_n_chars_parser(8)),
            preceded(space1, i16),
            preceded(space1, string_from_n_chars_parser(1)),
        ),
        |(
            designation,
            product_class_id,
            tariff_group,
            output_control,
            short_name,
            surcharge,
            flag,
        )| {
            TransportTypeAndTypeLine::OfferDefinition {
                designation,
                product_class_id,
                tariff_group,
                output_control,
                short_name,
                surcharge,
                flag,
            }
        },
    )
    .parse(input)
}

fn language_combinator(input: &str) -> IResult<&str, TransportTypeAndTypeLine> {
    map(
        terminated(preceded(tag("<"), take_until1(">")), tag(">")),
        |s: &str| TransportTypeAndTypeLine::LanguageDefinition(s.to_string()),
    )
    .parse(input)
}

fn class_combinator(input: &str) -> IResult<&str, TransportTypeAndTypeLine> {
    map(
        (
            preceded(tag("class"), i16),
            preceded(space1, string_till_eol_parser()),
        ),
        |(product_class_id, product_class_name)| TransportTypeAndTypeLine::Class {
            product_class_id,
            product_class_name,
        },
    )
    .parse(input)
}

fn category_combinator(input: &str) -> IResult<&str, TransportTypeAndTypeLine> {
    map(
        (
            preceded(tag("category"), i16),
            preceded(space1, string_till_eol_parser()),
        ),
        |(category_id, category_name)| TransportTypeAndTypeLine::Category {
            category_id,
            category_name,
        },
    )
    .parse(input)
}

fn option_combinator(input: &str) -> IResult<&str, TransportTypeAndTypeLine> {
    map(
        (
            preceded(tag("option"), i16),
            preceded(space1, string_till_eol_parser()),
        ),
        |(option_id, option_name)| TransportTypeAndTypeLine::Option {
            option_id,
            option_name,
        },
    )
    .parse(input)
}

fn iline_combinator(input: &str) -> IResult<&str, TransportTypeAndTypeLine> {
    map(
        (
            preceded(preceded(tag("*I"), space1), string_from_n_chars_parser(2)),
            preceded(char(' '), optional_i32_from_n_digits_parser(7)),
        ),
        |(code_name, id)| TransportTypeAndTypeLine::Information { code_name, id },
    )
    .parse(input)
}

fn parse_line(
    line: &str,
    data: &mut FxHashMap<i32, TransportType>,
    pk_type_converter: &mut FxHashMap<String, i32>,
    auto_increment: &AutoIncrement,
    current_language: &mut Language,
) -> Result<(), Box<dyn Error>> {
    let (_, transport_row) = alt((
        offer_definition_combinator,
        language_combinator,
        category_combinator,
        class_combinator,
        option_combinator,
        iline_combinator,
    ))
    .parse(line)
    .map_err(|e| format!("Error {e} while parsing {line}"))?;

    match transport_row {
        TransportTypeAndTypeLine::OfferDefinition {
            designation,
            product_class_id,
            tariff_group,
            output_control,
            short_name,
            surcharge,
            flag,
        } => {
            let id = auto_increment.next();

            if let Some(previous) = pk_type_converter.insert(designation.to_owned(), id) {
                log::error!(
                    "Warning: previous id {previous} for {designation}. The designation, {designation}, is not unique."
                );
            };
            let tt = TransportType::new(
                id,
                designation.to_owned(),
                product_class_id,
                tariff_group,
                output_control,
                short_name,
                surcharge,
                flag,
            );
            data.insert(tt.id(), tt);
        }
        TransportTypeAndTypeLine::LanguageDefinition(language) => {
            match language.as_str() {
                "Deutsch" => {
                    *current_language = Language::German;
                }
                "Franzoesisch" => {
                    *current_language = Language::French;
                }
                "Englisch" => {
                    *current_language = Language::English;
                }
                "Italienisch" => {
                    *current_language = Language::Italian;
                }
                "text" => {
                    // Do nothing
                }
                _ => unreachable!(),
            };
        }
        TransportTypeAndTypeLine::Class {
            product_class_id,
            product_class_name,
        } => {
            for transport_type in data.values_mut() {
                if transport_type.product_class_id() == product_class_id {
                    transport_type.set_product_class_name(*current_language, &product_class_name)
                }
            }
        }
        TransportTypeAndTypeLine::Category {
            category_id: _,
            category_name,
        } => {
            let id = auto_increment.get();
            if let Some(transport_type) = data.get_mut(&id) {
                transport_type.set_category_name(*current_language, &category_name);
            } else {
                return Err(format!("Error: TransportType not found for id: {id}").into());
            }
        }
        TransportTypeAndTypeLine::Option {
            option_id: _,
            option_name: _,
        } => {}
        TransportTypeAndTypeLine::Information {
            code_name: _,
            id: _,
        } => {}
    }

    Ok(())
}

pub fn parse(path: &str) -> Result<TransportTypeAndTypeConverter, Box<dyn Error>> {
    log::info!("Parsing ZUGART...");

    let transport_types = read_lines(&format!("{path}/ZUGART"), 0)?;

    let auto_increment = AutoIncrement::new();
    let mut data = FxHashMap::default();
    let mut pk_type_converter = FxHashMap::default();
    let mut current_language = Language::default();

    transport_types
        .into_iter()
        .filter(|line| !line.trim().is_empty())
        .try_for_each(|line| {
            parse_line(
                &line,
                &mut data,
                &mut pk_type_converter,
                &auto_increment,
                &mut current_language,
            )
        })?;

    Ok((ResourceStorage::new(data), pk_type_converter))
}
