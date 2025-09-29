/// # BETRIEB_* files
///
/// List of transport companies. The term “transport company” is understood in different ways.
/// In the context of opentransportdata.swiss, it is understood that it is an organisation
/// that is responsible for the runs described in the FPLAN. A detailed description of
/// the transport companies and business organisations can be found here.
///
/// Each TU is described in detail with 2 lines:
///
///
/// - The first line:
///     - Operator no. (for BETRIEB / OPERATION file)
///         - Short name (after the “K”)
///         - Long name (after the “L”)
///         - Full name (after the”V”)
///
/// - The second line:
///     - Operator no. (for BETRIEB / OPERATION file)
///     - “:”
///     - TU code (or administration number)
///         - Several TU codes can be listed. These share the information in the first line.
///
/// ## Example (excerpt):
///
/// `
/// ...
/// 00379 K "SBB" L "SBB" V "Schweizerische Bundesbahnen SBB"     % Betrieb-Nr 00379, kurz sbb, lang sbb, voll schweizerische bundesbahn sbb
/// 00379 : 000011                                                % Betrieb-Nr 00379, TU-Code 000011
/// 00380 K "SOB" L "SOB-bt" V "Schweizerische Südostbahn (bt)"   % Betrieb-Nr 00380, kurz sob, lang sob-bt,  voll schweizerische südostbahn (bt)
/// 00380 : 000036                                                % Betrieb-Nr 00380, TU-Code 000036
/// 00381 K "SOB" L "SOB-sob" V "Schweizerische Südostbahn (sob)" % Betrieb-Nr 00381, kurz sob, lang sob-sob, voll schweizerische südostbahn (sob)
/// 00381 : 000082                                                % Betrieb-Nr 00381, TU-Code 000082
/// ...
/// `
///
/// 4 file(s).
/// File(s) read by the parser:
/// BETRIEB_DE, BETRIEB_EN, BETRIEB_FR, BETRIEB_IT
use std::error::Error;

use nom::{
    Parser,
    branch::alt,
    bytes::{complete::take_until, tag},
    character::complete::{i32, space1},
    combinator::map,
    sequence::{preceded, terminated},
};
use rustc_hash::FxHashMap;

use crate::{
    models::{Language, TransportCompany},
    parsing::helpers::{read_lines, string_till_eol_parser},
    storage::ResourceStorage,
};

enum TransportCompanyLine {
    Kline {
        id: i32,
        short_name: String,
        long_name: String,
        full_name: String,
    },
    Nline {
        id: i32,
        sboid: String,
    },
    ColonLine {
        id: i32,
        administrations: Vec<String>,
    },
}

fn kline_combinator<'a>()
-> impl Parser<&'a str, Output = TransportCompanyLine, Error = nom::error::Error<&'a str>> {
    map(
        (
            i32,
            preceded(
                tag(" K"),
                preceded(
                    space1,
                    map(
                        terminated(preceded(tag("\""), take_until("\"")), tag("\"")),
                        String::from,
                    ),
                ),
            ),
            preceded(
                tag(" L"),
                preceded(
                    space1,
                    map(
                        terminated(preceded(tag("\""), take_until("\"")), tag("\"")),
                        String::from,
                    ),
                ),
            ),
            preceded(
                tag(" V"),
                preceded(
                    space1,
                    map(
                        terminated(preceded(tag("\""), take_until("\"")), tag("\"")),
                        String::from,
                    ),
                ),
            ),
        ),
        |(id, short_name, long_name, full_name)| TransportCompanyLine::Kline {
            id,
            short_name,
            long_name,
            full_name,
        },
    )
}

fn nline_combinator<'a>()
-> impl Parser<&'a str, Output = TransportCompanyLine, Error = nom::error::Error<&'a str>> {
    map(
        (
            i32,
            preceded(
                tag(" N"),
                preceded(
                    space1,
                    map(
                        terminated(preceded(tag("\""), take_until("\"")), tag("\"")),
                        String::from,
                    ),
                ),
            ),
        ),
        |(id, sboid)| TransportCompanyLine::Nline { id, sboid },
    )
}

fn colon_combinator<'a>()
-> impl Parser<&'a str, Output = TransportCompanyLine, Error = nom::error::Error<&'a str>> {
    map(
        (
            i32,
            preceded(
                space1,
                preceded(tag(":"), preceded(space1, string_till_eol_parser())),
            ),
        ),
        |(id, administrations)| {
            let administrations = administrations
                .split(" ")
                .map(String::from)
                .collect::<Vec<_>>();

            TransportCompanyLine::ColonLine {
                id,
                administrations,
            }
        },
    )
}

fn parse_transport_company_line(
    line: &str,
    transport_company: &mut FxHashMap<i32, TransportCompany>,
    language: Language,
) -> Result<(), Box<dyn Error>> {
    let (_, tcl) = alt((kline_combinator(), nline_combinator(), colon_combinator()))
        .parse(line)
        .map_err(|e| format!("Error {e} while parsing {line}"))?;

    match tcl {
        TransportCompanyLine::Kline {
            id,
            short_name,
            long_name,
            full_name,
        } => {
            if let Some(tc) = transport_company.get_mut(&id) {
                tc.set_short_name(language, &short_name);
                tc.set_full_name(language, &full_name);
                tc.set_long_name(language, &long_name);
            } else {
                let mut tc = TransportCompany::new(id);
                tc.set_short_name(language, &short_name);
                tc.set_full_name(language, &full_name);
                tc.set_long_name(language, &long_name);
                transport_company.insert(id, tc);
            }
        }
        TransportCompanyLine::Nline { id: _, sboid: _ } => {
            // TODO: Use sboid some day
        }
        TransportCompanyLine::ColonLine {
            id,
            administrations,
        } => {
            if let Some(tc) = transport_company.get_mut(&id) {
                tc.set_administrations(administrations);
            } else {
                let mut tc = TransportCompany::new(id);
                tc.set_administrations(administrations);
                transport_company.insert(id, tc);
            }
        }
    }

    Ok(())
}

pub fn parse(path: &str) -> Result<ResourceStorage<TransportCompany>, Box<dyn Error>> {
    let languages = [
        Language::German,
        Language::English,
        Language::French,
        Language::Italian,
    ];
    let mut transport_company = FxHashMap::default();

    for language in languages {
        let postfix = match language {
            Language::German => "DE",
            Language::French => "FR",
            Language::English => "EN",
            Language::Italian => "IT",
        };
        log::info!("Parsing BETRIEB_{postfix}...");
        read_lines(&format!("{path}/BETRIEB_{postfix}"), 0)?
            .into_iter()
            .filter(|line| !line.trim().is_empty())
            .try_for_each(|line| {
                parse_transport_company_line(&line, &mut transport_company, language)
                    .map_err(|e| format!("Error: {e}, for line: {line}"))
            })?;
    }

    Ok(ResourceStorage::new(transport_company))
}
