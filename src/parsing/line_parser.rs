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
use std::error::Error;

use nom::{branch::alt, bytes::tag, character::char, combinator::map, Parser};

use crate::{
    models::{Color, Line, Model},
    parsing::helpers::{
        i16_from_n_digits_parser, i32_from_n_digits_parser, read_lines, string_till_eol_parser,
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

fn row_k_nt_lt_w_combinator<'a>(
) -> impl Parser<&'a str, Output = Option<LineType>, Error = nom::error::Error<&'a str>> {
    map(
        (
            i32_from_n_digits_parser(7),
            char(' '),
            alt((tag("K "), tag("N T "), tag("L T "), tag("W "))),
            string_till_eol_parser(),
        ),
        |(id, _, line_type, name)| match line_type {
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
}

fn row_f_b_combinator<'a>(
) -> impl Parser<&'a str, Output = Option<LineType>, Error = nom::error::Error<&'a str>> {
    map(
        (
            i32_from_n_digits_parser(7),
            char(' '),
            alt((tag("F "), tag("B "))),
            (
                i16_from_n_digits_parser(3),
                char(' '),
                i16_from_n_digits_parser(3),
                char(' '),
                i16_from_n_digits_parser(3),
            ),
        ),
        |(id, _, line_type, (r, _, g, _, b))| match line_type {
            "F " => Some(LineType::Fline { id, r, g, b }),
            "B " => Some(LineType::Bline { id, r, g, b }),
            _ => None,
        },
    )
}

fn parse_line(line: &str, data: &mut Vec<Line>) -> Result<(), Box<dyn Error>> {
    let (_, line_row) = alt((row_k_nt_lt_w_combinator(), row_f_b_combinator()))
        .parse(line)
        .map_err(|e| format!("Error {e} while parsing {line}"))?;

    match line_row.ok_or("Error missing line type")? {
        LineType::Kline { id, name } => data.push(Line::new(id, name)),
        LineType::NTline { id, short_name } => {
            let line = data.last_mut().ok_or("Type K row missing.")?;
            if id != line.id() {
                return Err(
                    format!("Error: Line id not corresponding, {id}, {}", line.id()).into(),
                );
            }
            line.set_short_name(short_name);
        }
        LineType::LTline { id, long_name } => {
            let line = data.last_mut().ok_or("Type K row missing.")?;
            if id != line.id() {
                return Err(
                    format!("Error: Line id not corresponding, {id}, {}", line.id()).into(),
                );
            }
            line.set_long_name(long_name);
        }
        LineType::Wline {
            id,
            internal_designation,
        } => {
            let line = data.last_mut().ok_or("Type K row missing.")?;
            if id != line.id() {
                return Err(
                    format!("Error: Line id not corresponding, {id}, {}", line.id()).into(),
                );
            }
            line.set_internal_designation(internal_designation);
        }

        LineType::Fline { id, r, g, b } => {
            let line = data.last_mut().ok_or("Type K row missing.")?;
            if id != line.id() {
                return Err(
                    format!("Error: Line id not corresponding, {id}, {}", line.id()).into(),
                );
            }
            line.set_text_color(Color::new(r, g, b));
        }
        LineType::Bline { id, r, g, b } => {
            let line = data.last_mut().ok_or("Type K row missing.")?;
            if id != line.id() {
                return Err(
                    format!("Error: Line id not corresponding, {id}, {}", line.id()).into(),
                );
            }
            line.set_background_color(Color::new(r, g, b));
        }
        l => return Err(format!("Line not parsed {l:?}").into()),
    }

    Ok(())
}

pub fn parse(path: &str) -> Result<ResourceStorage<Line>, Box<dyn Error>> {
    log::info!("Parsing LINIE...");

    let lines = read_lines(&format!("{path}/LINIE"), 0)?;

    let mut data = Vec::new();

    lines
        .into_iter()
        .filter(|line| !line.trim().is_empty())
        .try_for_each(|line| parse_line(&line, &mut data))?;
    let data = Line::vec_to_map(data);

    Ok(ResourceStorage::new(data))
}

// TODO: Add tests
