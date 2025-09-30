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
    Parser,
    bytes::complete::{tag, take_until1},
    character::complete::{i16, space1},
    combinator::map,
    sequence::{preceded, terminated},
};
use rustc_hash::FxHashMap;

use crate::{
    Version,
    models::{Language, Model, TransportType},
    parsing::{
        AdvancedRowMatcher, ColumnDefinition, ExpectedType, FastRowMatcher, FileParser,
        ParsedValue, RowDefinition, RowParser,
        helpers::{string_from_n_chars_parser, string_till_eol_parser},
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
        category_id: i16,
        category_name: String,
    },
    Option {},
}

fn offer_definition_combinator<'a>()
-> impl Parser<&'a str, Output = TransportTypeAndTypeLine, Error = nom::error::Error<&'a str>> {
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
}

fn language_combinator<'a>()
-> impl Parser<&'a str, Output = TransportTypeAndTypeLine, Error = nom::error::Error<&'a str>> {
    map(
        terminated(preceded(tag("<"), take_until1(">")), tag(">")),
        |s: &str| TransportTypeAndTypeLine::LanguageDefinition(s.to_string()),
    )
}

fn class_combinator<'a>()
-> impl Parser<&'a str, Output = TransportTypeAndTypeLine, Error = nom::error::Error<&'a str>> {
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
}

fn category_combinator<'a>()
-> impl Parser<&'a str, Output = TransportTypeAndTypeLine, Error = nom::error::Error<&'a str>> {
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
}

fn option_combinator<'a>()
-> impl Parser<&'a str, Output = TransportTypeAndTypeLine, Error = nom::error::Error<&'a str>> {
    map(
        (
            preceded(tag("option"), i16),
            preceded(space1, string_till_eol_parser()),
        ),
        |(category_id, category_name)| TransportTypeAndTypeLine::Category {
            category_id,
            category_name,
        },
    )
}

pub fn parse(
    version: Version,
    path: &str,
) -> Result<TransportTypeAndTypeConverter, Box<dyn Error>> {
    log::info!("Parsing ZUGART...");
    const ROW_A: i32 = 1;
    const ROW_B: i32 = 2;
    const ROW_C: i32 = 3;
    const ROW_D: i32 = 4;
    const ROW_E: i32 = 5;
    const ROW_F: i32 = 6;

    #[rustfmt::skip]
    let row_parser = RowParser::new(vec![
        // This row is used to create a TransportType instance.
        RowDefinition::new(ROW_A, Box::new(
            AdvancedRowMatcher::new(r"^.{3} [ 0-9]{2}")?
        ),

        match version {
             Version::V_5_40_41_2_0_4 | Version::V_5_40_41_2_0_5 | Version::V_5_40_41_2_0_6 => vec![
                ColumnDefinition::new(1, 3, ExpectedType::String),
                ColumnDefinition::new(5, 6, ExpectedType::Integer16),
                ColumnDefinition::new(8, 8, ExpectedType::String),
                ColumnDefinition::new(10, 10, ExpectedType::Integer16),
                ColumnDefinition::new(12, 19, ExpectedType::String),
                ColumnDefinition::new(21, 21, ExpectedType::Integer16),
                ColumnDefinition::new(23, 23, ExpectedType::String),
            ],
            Version::V_5_40_41_2_0_7 => vec![
                ColumnDefinition::new(1, 3, ExpectedType::String),
                ColumnDefinition::new(5, 6, ExpectedType::Integer16),
                ColumnDefinition::new(8, 8, ExpectedType::String),
                ColumnDefinition::new(11, 11, ExpectedType::Integer16),
                ColumnDefinition::new(13, 20, ExpectedType::String),
                ColumnDefinition::new(22, 22, ExpectedType::Integer16),
                ColumnDefinition::new(24, 24, ExpectedType::String),
            ],

        }),
        // This row indicates the language for translations in the section that follows it.
        RowDefinition::new(ROW_B, Box::new(FastRowMatcher::new(1, 1, "<", true)), vec![
            ColumnDefinition::new(1, -1, ExpectedType::String),
        ]),
        // This row contains the product class name in a specific language.
        RowDefinition::new(ROW_C, Box::new(
            AdvancedRowMatcher::new(r"^class.+$")?
        ), vec![
            ColumnDefinition::new(6, 7, ExpectedType::Integer16),
            ColumnDefinition::new(9, -1, ExpectedType::String),
        ]),
        // This row is ignored.
        RowDefinition::new(ROW_D, Box::new(AdvancedRowMatcher::new(r"^option.+$")?), Vec::new()),
        // This row contains the category name in a specific language.
        RowDefinition::new(ROW_E, Box::new(
            AdvancedRowMatcher::new(r"^category.+$")?
        ), vec![
            ColumnDefinition::new(10, 12, ExpectedType::Integer32),
            ColumnDefinition::new(14, -1, ExpectedType::String),
        ]),
        // This row contains specific information
        RowDefinition::new(ROW_F, Box::new(FastRowMatcher::new(1, 2, "*I", true)), vec![
            ColumnDefinition::new(4, 5, ExpectedType::String),
            ColumnDefinition::new(7, 15, ExpectedType::OptionInteger32),
        ]),
    ]);

    let parser = FileParser::new(&format!("{path}/ZUGART"), row_parser)?;

    let auto_increment = AutoIncrement::new();
    let mut data = Vec::new();
    let mut pk_type_converter = FxHashMap::default();

    let mut current_language = Language::default();

    for x in parser.parse() {
        let (id, _, values) = x?;

        match id {
            ROW_A => {
                let transport_type =
                    create_instance(values, &auto_increment, &mut pk_type_converter);
                data.push(transport_type);
            }
            _ => {
                let transport_type = data.last_mut().ok_or("Type A row missing.")?;

                match id {
                    ROW_B => update_current_language(values, &mut current_language),
                    ROW_C => {
                        set_product_class_name(values, &mut data, current_language);
                    }
                    ROW_D => {}
                    ROW_E => set_category_name(values, transport_type, current_language),
                    ROW_F => {
                        // TODO: Use information, currently not used
                    }
                    _ => unreachable!(),
                }
            }
        }
    }

    let data = TransportType::vec_to_map(data);

    Ok((ResourceStorage::new(data), pk_type_converter))
}

pub fn old_parse(
    version: Version,
    path: &str,
) -> Result<TransportTypeAndTypeConverter, Box<dyn Error>> {
    log::info!("Parsing ZUGART...");
    const ROW_A: i32 = 1;
    const ROW_B: i32 = 2;
    const ROW_C: i32 = 3;
    const ROW_D: i32 = 4;
    const ROW_E: i32 = 5;
    const ROW_F: i32 = 6;

    #[rustfmt::skip]
    let row_parser = RowParser::new(vec![
        // This row is used to create a TransportType instance.
        RowDefinition::new(ROW_A, Box::new(
            AdvancedRowMatcher::new(r"^.{3} [ 0-9]{2}")?
        ),

        match version {
             Version::V_5_40_41_2_0_4 | Version::V_5_40_41_2_0_5 | Version::V_5_40_41_2_0_6 => vec![
                ColumnDefinition::new(1, 3, ExpectedType::String),
                ColumnDefinition::new(5, 6, ExpectedType::Integer16),
                ColumnDefinition::new(8, 8, ExpectedType::String),
                ColumnDefinition::new(10, 10, ExpectedType::Integer16),
                ColumnDefinition::new(12, 19, ExpectedType::String),
                ColumnDefinition::new(21, 21, ExpectedType::Integer16),
                ColumnDefinition::new(23, 23, ExpectedType::String),
            ],
            Version::V_5_40_41_2_0_7 => vec![
                ColumnDefinition::new(1, 3, ExpectedType::String),
                ColumnDefinition::new(5, 6, ExpectedType::Integer16),
                ColumnDefinition::new(8, 8, ExpectedType::String),
                ColumnDefinition::new(11, 11, ExpectedType::Integer16),
                ColumnDefinition::new(13, 20, ExpectedType::String),
                ColumnDefinition::new(22, 22, ExpectedType::Integer16),
                ColumnDefinition::new(24, 24, ExpectedType::String),
            ],

        }),
        // This row indicates the language for translations in the section that follows it.
        RowDefinition::new(ROW_B, Box::new(FastRowMatcher::new(1, 1, "<", true)), vec![
            ColumnDefinition::new(1, -1, ExpectedType::String),
        ]),
        // This row contains the product class name in a specific language.
        RowDefinition::new(ROW_C, Box::new(
            AdvancedRowMatcher::new(r"^class.+$")?
        ), vec![
            ColumnDefinition::new(6, 7, ExpectedType::Integer16),
            ColumnDefinition::new(9, -1, ExpectedType::String),
        ]),
        // This row is ignored.
        RowDefinition::new(ROW_D, Box::new(AdvancedRowMatcher::new(r"^option.+$")?), Vec::new()),
        // This row contains the category name in a specific language.
        RowDefinition::new(ROW_E, Box::new(
            AdvancedRowMatcher::new(r"^category.+$")?
        ), vec![
            ColumnDefinition::new(10, 12, ExpectedType::Integer32),
            ColumnDefinition::new(14, -1, ExpectedType::String),
        ]),
        // This row contains specific information
        RowDefinition::new(ROW_F, Box::new(FastRowMatcher::new(1, 2, "*I", true)), vec![
            ColumnDefinition::new(4, 5, ExpectedType::String),
            ColumnDefinition::new(7, 15, ExpectedType::OptionInteger32),
        ]),
    ]);

    let parser = FileParser::new(&format!("{path}/ZUGART"), row_parser)?;

    let auto_increment = AutoIncrement::new();
    let mut data = Vec::new();
    let mut pk_type_converter = FxHashMap::default();

    let mut current_language = Language::default();

    for x in parser.parse() {
        let (id, _, values) = x?;

        match id {
            ROW_A => {
                let transport_type =
                    create_instance(values, &auto_increment, &mut pk_type_converter);
                data.push(transport_type);
            }
            _ => {
                let transport_type = data.last_mut().ok_or("Type A row missing.")?;

                match id {
                    ROW_B => update_current_language(values, &mut current_language),
                    ROW_C => {
                        set_product_class_name(values, &mut data, current_language);
                    }
                    ROW_D => {}
                    ROW_E => set_category_name(values, transport_type, current_language),
                    ROW_F => {
                        // TODO: Use information, currently not used
                    }
                    _ => unreachable!(),
                }
            }
        }
    }

    let data = TransportType::vec_to_map(data);

    Ok((ResourceStorage::new(data), pk_type_converter))
}

// ------------------------------------------------------------------------------------------------
// --- Data Processing Functions
// ------------------------------------------------------------------------------------------------

fn create_instance(
    mut values: Vec<ParsedValue>,
    auto_increment: &AutoIncrement,
    pk_type_converter: &mut FxHashMap<String, i32>,
) -> TransportType {
    let designation: String = values.remove(0).into();
    let product_class_id: i16 = values.remove(0).into();
    let tarrif_group: String = values.remove(0).into();
    let output_control: i16 = values.remove(0).into();
    let short_name: String = values.remove(0).into();
    let surchage: i16 = values.remove(0).into();
    let flag: String = values.remove(0).into();

    let id = auto_increment.next();

    if let Some(previous) = pk_type_converter.insert(designation.to_owned(), id) {
        log::error!(
            "Warning: previous id {previous} for {designation}. The designation, {designation}, is not unique."
        );
    };
    TransportType::new(
        id,
        designation.to_owned(),
        product_class_id,
        tarrif_group,
        output_control,
        short_name,
        surchage,
        flag,
    )
}

fn set_product_class_name(
    mut values: Vec<ParsedValue>,
    data: &mut Vec<TransportType>,
    language: Language,
) {
    let product_class_id: i16 = values.remove(0).into();
    let product_class_name: String = values.remove(0).into();

    for transport_type in data {
        if transport_type.product_class_id() == product_class_id {
            transport_type.set_product_class_name(language, &product_class_name)
        }
    }
}

fn set_category_name(
    mut values: Vec<ParsedValue>,
    transport_type: &mut TransportType,
    language: Language,
) {
    let _: i32 = values.remove(0).into();
    let category_name: String = values.remove(0).into();

    transport_type.set_category_name(language, &category_name);
}

// ------------------------------------------------------------------------------------------------
// --- Helper Functions
// ------------------------------------------------------------------------------------------------

fn update_current_language(mut values: Vec<ParsedValue>, current_language: &mut Language) {
    let language: String = values.remove(0).into();
    let language = &language[1..&language.len() - 1];

    if language != "text" {
        *current_language = match language {
            "Deutsch" => Language::German,
            "Franzoesisch" => Language::French,
            "Englisch" => Language::English,
            "Italienisch" => Language::Italian,
            _ => unreachable!(),
        };
    }
}
