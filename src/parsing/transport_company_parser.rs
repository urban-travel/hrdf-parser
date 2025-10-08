use std::path::Path;

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
use nom::{
    IResult, Parser,
    branch::alt,
    bytes::{complete::take_until, tag},
    character::complete::{i32, space1},
    combinator::map,
    sequence::{preceded, terminated},
};
use rustc_hash::FxHashMap;

use crate::error::{HResult, HrdfError};
use crate::{
    models::{Language, TransportCompany},
    parsing::{
        error::PResult,
        helpers::{read_lines, string_till_eol_parser},
    },
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
        #[allow(unused)]
        id: i32,
        #[allow(unused)]
        sboid: String,
    },
    ColonLine {
        id: i32,
        administrations: Vec<String>,
    },
}

fn kline_combinator(input: &str) -> IResult<&str, TransportCompanyLine> {
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
    .parse(input)
}

fn nline_combinator(input: &str) -> IResult<&str, TransportCompanyLine> {
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
    .parse(input)
}

fn colon_combinator(input: &str) -> IResult<&str, TransportCompanyLine> {
    map(
        (
            i32,
            preceded(
                space1,
                preceded(tag(":"), preceded(space1, string_till_eol_parser)),
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
    .parse(input)
}

fn parse_transport_company_line(
    line: &str,
    transport_company: &mut FxHashMap<i32, TransportCompany>,
    language: Language,
) -> PResult<()> {
    let (_, tcl) = alt((kline_combinator, nline_combinator, colon_combinator)).parse(line)?;

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

pub fn parse(path: &Path) -> HResult<ResourceStorage<TransportCompany>> {
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
        let file = path.join(format!("BETRIEB_{postfix}"));
        read_lines(&file, 0)?
            .into_iter()
            .enumerate()
            .filter(|(_, line)| !line.trim().is_empty())
            .try_for_each(|(line_number, line)| {
                parse_transport_company_line(&line, &mut transport_company, language).map_err(|e| {
                    HrdfError::Parsing {
                        error: e,
                        file: String::from(file.to_string_lossy()),
                        line,
                        line_number,
                    }
                })
            })?;
    }

    Ok(ResourceStorage::new(transport_company))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parsing::tests::get_json_values;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_kline_combinator_basic() {
        let input = r#"00379 K "SBB" L "SBB" V "Schweizerische Bundesbahnen SBB""#;
        let result = kline_combinator(input);
        assert!(result.is_ok());
        let (_, tc_line) = result.unwrap();
        match tc_line {
            TransportCompanyLine::Kline {
                id,
                short_name,
                long_name,
                full_name,
            } => {
                assert_eq!(id, 379);
                assert_eq!(short_name, "SBB");
                assert_eq!(long_name, "SBB");
                assert_eq!(full_name, "Schweizerische Bundesbahnen SBB");
            }
            _ => panic!("Expected Kline variant"),
        }
    }

    #[test]
    fn test_kline_combinator_sob() {
        let input = r#"00380 K "SOB" L "SOB-bt" V "Schweizerische Südostbahn (bt)""#;
        let result = kline_combinator(input);
        assert!(result.is_ok());
        let (_, tc_line) = result.unwrap();
        match tc_line {
            TransportCompanyLine::Kline {
                id,
                short_name,
                long_name,
                full_name,
            } => {
                assert_eq!(id, 380);
                assert_eq!(short_name, "SOB");
                assert_eq!(long_name, "SOB-bt");
                assert_eq!(full_name, "Schweizerische Südostbahn (bt)");
            }
            _ => panic!("Expected Kline variant"),
        }
    }

    #[test]
    fn test_kline_combinator_with_spaces_in_names() {
        let input = r#"00381 K "SOB" L "SOB-sob" V "Schweizerische Südostbahn (sob)""#;
        let result = kline_combinator(input);
        assert!(result.is_ok());
        let (_, tc_line) = result.unwrap();
        match tc_line {
            TransportCompanyLine::Kline {
                id,
                short_name,
                long_name,
                full_name,
            } => {
                assert_eq!(id, 381);
                assert_eq!(short_name, "SOB");
                assert_eq!(long_name, "SOB-sob");
                assert_eq!(full_name, "Schweizerische Südostbahn (sob)");
            }
            _ => panic!("Expected Kline variant"),
        }
    }

    #[test]
    fn test_nline_combinator() {
        let input = r#"00379 N "ch:1:sboid:379""#;
        let result = nline_combinator(input);
        assert!(result.is_ok());
        let (_, tc_line) = result.unwrap();
        match tc_line {
            TransportCompanyLine::Nline { id, sboid } => {
                assert_eq!(id, 379);
                assert_eq!(sboid, "ch:1:sboid:379");
            }
            _ => panic!("Expected Nline variant"),
        }
    }

    #[test]
    fn test_colon_combinator_single_admin() {
        let input = "00379 : 000011";
        let result = colon_combinator(input);
        assert!(result.is_ok());
        let (_, tc_line) = result.unwrap();
        match tc_line {
            TransportCompanyLine::ColonLine {
                id,
                administrations,
            } => {
                assert_eq!(id, 379);
                assert_eq!(administrations.len(), 1);
                assert_eq!(administrations[0], "000011");
            }
            _ => panic!("Expected ColonLine variant"),
        }
    }

    #[test]
    fn test_colon_combinator_multiple_admins() {
        let input = "00380 : 000036 000082";
        let result = colon_combinator(input);
        assert!(result.is_ok());
        let (_, tc_line) = result.unwrap();
        match tc_line {
            TransportCompanyLine::ColonLine {
                id,
                administrations,
            } => {
                assert_eq!(id, 380);
                assert_eq!(administrations.len(), 2);
                assert_eq!(administrations[0], "000036");
                assert_eq!(administrations[1], "000082");
            }
            _ => panic!("Expected ColonLine variant"),
        }
    }

    #[test]
    fn test_parse_transport_company_line_creates_new_company() {
        let mut companies = FxHashMap::default();
        parse_transport_company_line(
            r#"00379 K "SBB" L "SBB" V "Schweizerische Bundesbahnen SBB""#,
            &mut companies,
            Language::German,
        )
        .unwrap();
        assert_eq!(companies.len(), 1);
        let company = companies.get(&379).unwrap();
        let reference = r#"
            {
                "id":379,
                "short_name":{"German":"SBB"},
                "long_name":{"German":"SBB"},
                "full_name":{"German":"Schweizerische Bundesbahnen SBB"},
                "administrations":[]
            }"#;

        let (company, reference) = get_json_values(company, reference).unwrap();
        assert_eq!(company, reference);
    }

    #[test]
    fn test_parse_transport_company_line_updates_existing() {
        let mut companies = FxHashMap::default();

        // Create company
        parse_transport_company_line(
            r#"00379 K "SBB" L "SBB" V "Schweizerische Bundesbahnen SBB""#,
            &mut companies,
            Language::German,
        )
        .unwrap();

        // Update with colon line
        parse_transport_company_line("00379 : 000011", &mut companies, Language::German).unwrap();

        assert_eq!(companies.len(), 1);
        let company = companies.get(&379).unwrap();
        let reference = r#"
            {
                "id":379,
                "short_name":{"German":"SBB"},
                "long_name":{"German":"SBB"},
                "full_name":{"German":"Schweizerische Bundesbahnen SBB"},
                "administrations":["000011"]
            }"#;

        let (company, reference) = get_json_values(company, reference).unwrap();
        assert_eq!(company, reference);
    }

    #[test]
    fn test_parse_transport_company_line_multiple_languages() {
        let mut companies = FxHashMap::default();

        parse_transport_company_line(
            r#"00379 K "SBB" L "SBB" V "Schweizerische Bundesbahnen SBB""#,
            &mut companies,
            Language::German,
        )
        .unwrap();

        parse_transport_company_line(
            r#"00379 K "CFF" L "CFF" V "Chemins de fer fédéraux CFF""#,
            &mut companies,
            Language::French,
        )
        .unwrap();

        assert_eq!(companies.len(), 1);
        let company = companies.get(&379).unwrap();
        let reference = r#"
            {
                "id":379,
                "short_name":{"German":"SBB", "French":"CFF"},
                "long_name":{"German":"SBB", "French":"CFF"},
                "full_name":{"German":"Schweizerische Bundesbahnen SBB", "French":"Chemins de fer fédéraux CFF"},
                "administrations":[]
            }"#;

        let (company, reference) = get_json_values(company, reference).unwrap();
        assert_eq!(company, reference);
    }

    #[test]
    fn test_colon_line_creates_company_if_not_exists() {
        let mut companies = FxHashMap::default();

        parse_transport_company_line("00379 : 000011", &mut companies, Language::German).unwrap();

        assert_eq!(companies.len(), 1);
        let company = companies.get(&379).unwrap();
        let reference = r#"
            {
                "id":379,
                "short_name":{},
                "long_name":{},
                "full_name":{},
                "administrations":["000011"]
            }"#;

        let (company, reference) = get_json_values(company, reference).unwrap();
        assert_eq!(company, reference);
    }

    #[test]
    fn test_nline_parsing_ignores_sboid() {
        let mut companies = FxHashMap::default();
        companies.insert(379, TransportCompany::new(379));

        let result = parse_transport_company_line(
            r#"00379 N "ch:1:sboid:379""#,
            &mut companies,
            Language::German,
        );

        assert!(result.is_ok());
        // SBOID is currently not used (TODO in code)
        let company = companies.get(&379).unwrap();
        let reference = r#"
            {
                "id":379,
                "short_name":{},
                "long_name":{},
                "full_name":{},
                "administrations":[]
            }"#;

        let (company, reference) = get_json_values(company, reference).unwrap();
        assert_eq!(company, reference);
    }
}
