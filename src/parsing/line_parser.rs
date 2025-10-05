/// # Line parsing
///
/// ## List of lines. The file contains:
///
/// * Line ID (not unique per line!)
/// * Line property code
/// * Property
///
/// ## The following property codes are supported:
///
/// * Line type K: Line key
/// * Line type W: Internal line designation
/// * Line type N T: Line short name
/// * Line type L T: Long line name
/// * Line type R T: Line region name (reserved for BAV ID)
/// * Line type D T: Line description
/// * Line type F: Line color
/// * Line type B: Line background color
/// * Line type H: Main line
/// * Line type I: Line info texts
///
/// ## Examples
///
/// `
/// ...
/// 0000001 K ch:1:SLNID:33:1     % Linie 1, Linienschlüssel ch:1:SLNID:33:1
/// 0000001 W interne Bezeichnung % Linie 1, interne Linienbezeichnung "interne Bezeichnung"
/// 0000001 N T Kurzname          % Linie 1, Linienkurzname "Kurzname"
/// 0000001 L T Langname          % Linie 1, Linienlangname "Langname"
/// 0000001 D T Description       % Linie 1, Linienbeschreibung "Description"
/// 0000001 F 001 002 003         % Linie 1, Linienfarbe RGB 1, 2, 3
/// 0000001 B 001 002 003         % Linie 1, Linienhintergrundfarbe RGB 1, 2, 3
/// 0000001 H 0000002             % Linie 1, Hauptlinie 2
/// 0000001 I TU 000000001        % Linie 1, Infotexttyp TU, Infotextnummer (s. INFOTEXT-Datei)
/// ...
/// 0000010 K 68                  % Linie 10, Linienschlüsse 68
/// 0000010 N T 68                % Linie 10, Linienkurzname 68
/// 0000010 F 255 255 255         % Linie 10, Linienfarbe RGB 255, 255, 255
/// 0000010 B 236 097 159         % Linie 10, Linienhintergrundfarbe RGB 236, 097, 159
/// ...
/// `
///
/// 1 file(s).
/// File(s) read by the parser:
/// LINIE
use nom::{
    IResult, Parser, branch::alt, bytes::tag, character::char, combinator::map, sequence::preceded,
};
use rustc_hash::FxHashMap;

use crate::{
    models::{Color, Line, Model},
    parsing::{
        error::{PResult, ParsingError},
        helpers::{
            i16_from_n_digits_parser, i32_from_n_digits_parser, read_lines, string_till_eol_parser,
        },
    },
    storage::ResourceStorage,
};

#[derive(Debug)]
enum LineType {
    // * Line type K: Line key
    Kline {
        id: i32,
        name: String,
    },
    // * Line type W: Internal line designation
    Wline {
        id: i32,
        internal_designation: String,
    },
    // * Line type N T: Line short name (not present)
    NTline {
        id: i32,
        short_name: String,
    },
    // * Line type L T: Long line name
    LTline {
        id: i32,
        long_name: String,
    },
    // * Line type R T: Line region name (reserved for BAV ID)
    #[allow(unused)]
    RTline,
    // * Line type D T: Line description (not present)
    #[allow(unused)]
    DTline,
    // * Line type F: Line color
    Fline {
        id: i32,
        r: i16,
        g: i16,
        b: i16,
    },
    // * Line type B: Line background color
    Bline {
        id: i32,
        r: i16,
        g: i16,
        b: i16,
    },
    // * Line type H: Main line (not present)
    #[allow(unused)]
    Hline,
    // * Line type I: Line info texts (not present)
    #[allow(unused)]
    Iline,
}

fn row_k_nt_lt_w_combinator(input: &str) -> IResult<&str, Option<LineType>> {
    map(
        (
            i32_from_n_digits_parser(7),
            preceded(
                char(' '),
                alt((tag("K "), tag("N T "), tag("L T "), tag("W "))),
            ),
            string_till_eol_parser,
        ),
        |(id, line_type, name)| match line_type {
            "K " => Some(LineType::Kline { id, name }),
            "N T " => Some(LineType::NTline {
                id,
                short_name: name,
            }),
            "L T " => Some(LineType::LTline {
                id,
                long_name: name,
            }),
            "W " => Some(LineType::Wline {
                id,
                internal_designation: name,
            }),
            _ => None,
        },
    )
    .parse(input)
}

fn row_f_b_combinator(input: &str) -> IResult<&str, Option<LineType>> {
    map(
        (
            i32_from_n_digits_parser(7),
            preceded(char(' '), alt((tag("F "), tag("B ")))),
            (
                i16_from_n_digits_parser(3),
                preceded(char(' '), i16_from_n_digits_parser(3)),
                preceded(char(' '), i16_from_n_digits_parser(3)),
            ),
        ),
        |(id, line_type, (r, g, b))| match line_type {
            "F " => Some(LineType::Fline { id, r, g, b }),
            "B " => Some(LineType::Bline { id, r, g, b }),
            _ => None,
        },
    )
    .parse(input)
}

fn parse_line(line: &str, data: &mut FxHashMap<i32, Line>) -> PResult<()> {
    let (_, line_row) = alt((row_k_nt_lt_w_combinator, row_f_b_combinator)).parse(line)?;

    match line_row.ok_or(ParsingError::MissingLineType)? {
        LineType::Kline { id, name } => {
            data.insert(id, Line::new(id, name));
        }
        LineType::NTline { id, short_name } => {
            let line = data.get_mut(&id).ok_or_else(|| {
                ParsingError::UnknownId(format!("For id: {id}, type K row missing."))
            })?;
            if id != line.id() {
                return Err(ParsingError::UnknownId(format!(
                    "Line id not corresponding, {id}, {}",
                    line.id()
                )));
            }
            line.set_short_name(short_name);
        }
        LineType::LTline { id, long_name } => {
            let line = data.get_mut(&id).ok_or_else(|| {
                ParsingError::UnknownId(format!("For id: {id}, type K row missing."))
            })?;
            if id != line.id() {
                return Err(ParsingError::UnknownId(format!(
                    "Line id not corresponding, {id}, {}",
                    line.id()
                )));
            }
            line.set_long_name(long_name);
        }
        LineType::Wline {
            id,
            internal_designation,
        } => {
            let line = data.get_mut(&id).ok_or_else(|| {
                ParsingError::UnknownId(format!("For id: {id}, type K row missing."))
            })?;
            if id != line.id() {
                return Err(ParsingError::UnknownId(format!(
                    "Line id not corresponding, {id}, {}",
                    line.id()
                )));
            }
            line.set_internal_designation(internal_designation);
        }

        LineType::Fline { id, r, g, b } => {
            let line = data.get_mut(&id).ok_or_else(|| {
                ParsingError::UnknownId(format!("For id: {id}, type K row missing."))
            })?;
            if id != line.id() {
                return Err(ParsingError::UnknownId(format!(
                    "Line id not corresponding, {id}, {}",
                    line.id()
                )));
            }
            line.set_text_color(Color::new(r, g, b));
        }
        LineType::Bline { id, r, g, b } => {
            let line = data.get_mut(&id).ok_or_else(|| {
                ParsingError::UnknownId(format!("For id: {id}, type K row missing."))
            })?;
            if id != line.id() {
                return Err(ParsingError::UnknownId(format!(
                    "Line id not corresponding, {id}, {}",
                    line.id()
                )));
            }
            line.set_background_color(Color::new(r, g, b));
        }
        l => {
            return Err(ParsingError::Unknown(format!("Line not parsed {l:?}")));
        }
    }

    Ok(())
}

pub fn parse(path: &str) -> PResult<ResourceStorage<Line>> {
    log::info!("Parsing LINIE...");

    let lines = read_lines(&format!("{path}/LINIE"), 0)?;

    let mut data = FxHashMap::default();

    lines
        .into_iter()
        .filter(|line| !line.trim().is_empty())
        .try_for_each(|line| parse_line(&line, &mut data))?;

    Ok(ResourceStorage::new(data))
}

#[cfg(test)]
mod tests {
    use crate::parsing::tests::get_json_values;

    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_row_k_combinator_valid() {
        let input = "0000001 K ch:1:SLNID:33:1";
        let result = row_k_nt_lt_w_combinator(input);
        assert!(result.is_ok());
        let (_, line_type) = result.unwrap();
        match line_type {
            Some(LineType::Kline { id, name }) => {
                assert_eq!(id, 1);
                assert_eq!(name, "ch:1:SLNID:33:1");
            }
            _ => panic!("Expected Kline variant"),
        }
    }

    #[test]
    fn test_row_k_combinator_with_spaces() {
        let input = "0000010 K 68";
        let result = row_k_nt_lt_w_combinator(input);
        assert!(result.is_ok());
        let (_, line_type) = result.unwrap();
        match line_type {
            Some(LineType::Kline { id, name }) => {
                assert_eq!(id, 10);
                assert_eq!(name, "68");
            }
            _ => panic!("Expected Kline variant"),
        }
    }

    #[test]
    fn test_row_nt_combinator_valid() {
        let input = "0000001 N T Kurzname";
        let result = row_k_nt_lt_w_combinator(input);
        assert!(result.is_ok());
        let (_, line_type) = result.unwrap();
        match line_type {
            Some(LineType::NTline { id, short_name }) => {
                assert_eq!(id, 1);
                assert_eq!(short_name, "Kurzname");
            }
            _ => panic!("Expected NTline variant"),
        }
    }

    #[test]
    fn test_row_lt_combinator_valid() {
        let input = "0000001 L T Langname";
        let result = row_k_nt_lt_w_combinator(input);
        assert!(result.is_ok());
        let (_, line_type) = result.unwrap();
        match line_type {
            Some(LineType::LTline { id, long_name }) => {
                assert_eq!(id, 1);
                assert_eq!(long_name, "Langname");
            }
            _ => panic!("Expected LTline variant"),
        }
    }

    #[test]
    fn test_row_w_combinator_valid() {
        let input = "0000001 W interne Bezeichnung";
        let result = row_k_nt_lt_w_combinator(input);
        assert!(result.is_ok());
        let (_, line_type) = result.unwrap();
        match line_type {
            Some(LineType::Wline {
                id,
                internal_designation,
            }) => {
                assert_eq!(id, 1);
                assert_eq!(internal_designation, "interne Bezeichnung");
            }
            _ => panic!("Expected Wline variant"),
        }
    }

    #[test]
    fn test_row_f_combinator_valid() {
        let input = "0000001 F 001 002 003";
        let result = row_f_b_combinator(input);
        assert!(result.is_ok());
        let (_, line_type) = result.unwrap();
        match line_type {
            Some(LineType::Fline { id, r, g, b }) => {
                assert_eq!(id, 1);
                assert_eq!(r, 1);
                assert_eq!(g, 2);
                assert_eq!(b, 3);
            }
            _ => panic!("Expected Fline variant"),
        }
    }

    #[test]
    fn test_row_f_combinator_max_rgb() {
        let input = "0000010 F 255 255 255";
        let result = row_f_b_combinator(input);
        assert!(result.is_ok());
        let (_, line_type) = result.unwrap();
        match line_type {
            Some(LineType::Fline { id, r, g, b }) => {
                assert_eq!(id, 10);
                assert_eq!(r, 255);
                assert_eq!(g, 255);
                assert_eq!(b, 255);
            }
            _ => panic!("Expected Fline variant"),
        }
    }

    #[test]
    fn test_row_b_combinator_valid() {
        let input = "0000001 B 001 002 003";
        let result = row_f_b_combinator(input);
        assert!(result.is_ok());
        let (_, line_type) = result.unwrap();
        match line_type {
            Some(LineType::Bline { id, r, g, b }) => {
                assert_eq!(id, 1);
                assert_eq!(r, 1);
                assert_eq!(g, 2);
                assert_eq!(b, 3);
            }
            _ => panic!("Expected Bline variant"),
        }
    }

    #[test]
    fn test_row_b_combinator_complex_rgb() {
        let input = "0000010 B 236 097 159";
        let result = row_f_b_combinator(input);
        assert!(result.is_ok());
        let (_, line_type) = result.unwrap();
        match line_type {
            Some(LineType::Bline { id, r, g, b }) => {
                assert_eq!(id, 10);
                assert_eq!(r, 236);
                assert_eq!(g, 97);
                assert_eq!(b, 159);
            }
            _ => panic!("Expected Bline variant"),
        }
    }

    #[test]
    fn test_parse_line_k_creates_new_line() {
        let mut data = FxHashMap::default();
        parse_line("0000001 K TestLine", &mut data).unwrap();
        assert_eq!(data.len(), 1);
        let line = data.get(&1).unwrap();
        let reference = r#"
            {
                "id":1,
                "name": "TestLine",
                "short_name": "",
                "long_name": "",
                "internal_designation": "",
                "text_color": {"r":0,"g":0,"b":0},
                "background_color": {"r":0,"g":0,"b":0}
            }"#;
        let (line, reference) = get_json_values(line, reference).unwrap();
        assert_eq!(line, reference);
    }

    #[test]
    #[should_panic]
    fn test_parse_line_nt_requires_existing_k() {
        let mut data = FxHashMap::default();
        parse_line("0000001 N T ShortName", &mut data).unwrap();
    }

    #[test]
    fn test_parse_line_complete_sequence() {
        let mut data = FxHashMap::default();

        parse_line("0000001 K ch:1:SLNID:33:1", &mut data).unwrap();
        parse_line("0000001 W internal", &mut data).unwrap();
        parse_line("0000001 N T Short", &mut data).unwrap();
        parse_line("0000001 L T Long Name", &mut data).unwrap();
        parse_line("0000001 F 255 128 064", &mut data).unwrap();
        parse_line("0000001 B 010 020 030", &mut data).unwrap();

        assert_eq!(data.len(), 1);
        let line = data.get(&1).unwrap();
        let reference = r#"
            {
                "id":1,
                "name": "ch:1:SLNID:33:1",
                "short_name": "Short",
                "long_name": "Long Name",
                "internal_designation": "internal",
                "text_color": {"r":255,"g":128,"b":64},
                "background_color": {"r":10,"g":20,"b":30}
            }"#;
        let (line, reference) = get_json_values(line, reference).unwrap();
        assert_eq!(line, reference);
    }

    #[test]
    fn test_parse_line_multiple_lines() {
        let mut data = FxHashMap::default();

        parse_line("0000001 K Line1", &mut data).unwrap();
        parse_line("0000002 K Line2", &mut data).unwrap();
        parse_line("0000001 N T L1", &mut data).unwrap();
        parse_line("0000002 N T L2", &mut data).unwrap();

        assert_eq!(data.len(), 2);
        let line = data.get(&1).unwrap();
        let reference = r#"
            {
                "id":1,
                "name": "Line1",
                "short_name": "L1",
                "long_name": "",
                "internal_designation": "",
                "text_color": {"r":0,"g":0,"b":0},
                "background_color": {"r":0,"g":0,"b":0}
            }"#;
        let (line, reference) = get_json_values(line, reference).unwrap();
        assert_eq!(line, reference);
        let line = data.get(&2).unwrap();
        let reference = r#"
            {
                "id":2,
                "name": "Line2",
                "short_name": "L2",
                "long_name": "",
                "internal_designation": "",
                "text_color": {"r":0,"g":0,"b":0},
                "background_color": {"r":0,"g":0,"b":0}
            }"#;
        let (line, reference) = get_json_values(line, reference).unwrap();
        assert_eq!(line, reference);
    }

    #[test]
    #[should_panic]
    fn test_parse_line_id_mismatch_error() {
        let mut data = FxHashMap::default();
        data.insert(1, Line::new(999, "Wrong".to_string()));

        parse_line("0000001 N T Test", &mut data).unwrap();
    }

    #[test]
    #[should_panic]
    fn test_empty_lines_are_filtered() {
        let mut data = FxHashMap::default();

        // Empty line should not cause error
        parse_line("", &mut data).unwrap();
    }

    #[test]
    fn test_color_parsing() {
        let mut data = FxHashMap::default();
        parse_line("0000123 K ColorTest", &mut data).unwrap();
        parse_line("0000123 F 255 000 128", &mut data).unwrap();
        parse_line("0000123 B 064 128 255", &mut data).unwrap();

        let line = data.get(&123).unwrap();
        let reference = r#"
            {
                "id":123,
                "name": "ColorTest",
                "short_name": "",
                "long_name": "",
                "internal_designation": "",
                "text_color": {"r":255,"g":0,"b":128},
                "background_color": {"r":64,"g":128,"b":255}
            }"#;
        let (line, reference) = get_json_values(line, reference).unwrap();
        assert_eq!(line, reference);
    }
}
