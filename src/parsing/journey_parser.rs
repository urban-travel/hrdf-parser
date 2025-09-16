/// # Journey parser
///
/// List of journeys and by far the largest and most comprehensive file in the HRDF export.
///
/// This file contains:
///
/// 1 file(s).
/// File(s) read by the parser:
/// FPLAN
use std::{cell::RefCell, error::Error, rc::Rc};

use chrono::NaiveTime;
use nom::{
    Parser,
    branch::alt,
    bytes::tag,
    character::{char, complete::space1},
    combinator::{map, map_res},
    sequence::preceded,
};
use rustc_hash::{FxHashMap, FxHashSet};

use crate::{
    JourneyId,
    models::{Journey, JourneyMetadataEntry, JourneyMetadataType, JourneyRouteEntry, Model},
    parsing::helpers::{
        i32_from_n_digits_parser, optional_i32_from_n_digits_parser, read_lines,
        string_from_n_chars_parser,
    },
    storage::ResourceStorage,
    utils::{AutoIncrement, create_time_from_value},
};

type JourneyAndTypeConverter = (ResourceStorage<Journey>, FxHashSet<JourneyId>);

/// ## Z-lines
///
/// - *Z lines: as header information for the run. Further details on this topic and its implementation in Switzerland can be found in the RV. It includes:
///     - The journey number (primary key with the TU code)
///     - Transport company (TU) code (see File BETRIEB_*)
///         - For the TU code = 801, the region information must also be taken into account. This information is contained in line *I with the INFOTEXTCODE RN.
///     - Option
///         - NOT PART OF HRDF. 3-digit means of transport variant code without technical meaning
///     - (optional) Number of cycles
///     - (optional) Cycle time in minutes
///
/// ### Example (excerpt):
///
/// `
/// *Z 000003 000011   101         % Fahrtnummer 3, für TU 11 (SBB), mit Variante 101 (ignore)
/// ...
/// *Z 123456 000011   101 012 060 % Fahrtnummer 123456, für TU 11 (SBB), mit Variante 101 (ignore), 12 mal, alle 60 Minuten
/// ...
fn row_z_combinator<'a>() -> impl Parser<
    &'a str,
    Output = (
        i32,
        char,
        String,
        &'a str,
        i32,
        char,
        Option<i32>,
        char,
        Option<i32>,
    ),
    Error = nom::error::Error<&'a str>,
> {
    preceded(
        tag("*Z "),
        (
            i32_from_n_digits_parser(6),
            char(' '),
            string_from_n_chars_parser(6),
            space1,
            i32_from_n_digits_parser(3), // Maybe need to make optional
            char(' '),
            optional_i32_from_n_digits_parser(3),
            char(' '),
            optional_i32_from_n_digits_parser(3),
        ),
    )
}

/// ## G-lines
///
/// - *G-lines: Reference to the offer category (s. ZUGART file). It includes:
///     - Reference to the offer category
///         - The term “Angebotskategorie” (offer category) may have a different meaning here than in colloquial language! A colloquial term (also according to the HRDF doc.) would be “transport mode type”.
///     - Stop from which the offer category applies
///     - Stop up to which the offer category applies
/// ### Example (excerpt):
///
/// `
/// *Z ...
/// *G ICE 8500090 8503000 % Angebotskategorie ICE gilt ab HS-Nr. 8500090 bis HS-Nr. 8503000
/// ...
/// `
///
fn row_g_combinator<'a>() -> impl Parser<
    &'a str,
    Output = (String, char, Option<i32>, char, Option<i32>),
    Error = nom::error::Error<&'a str>,
> {
    preceded(
        tag("*G "),
        (
            string_from_n_chars_parser(3),
            char(' '),
            optional_i32_from_n_digits_parser(7),
            char(' '),
            optional_i32_from_n_digits_parser(7),
        ),
    )
}

/// ## A VE-lines
///
/// - *A VE lines: Reference to the validity information (see file BITFELD). Further details on this topic and its implementation in Switzerland can be found in the RV. It includes:
///     - Stop from which the offer category applies
///     - Stop up to which the offer category applies
///     - Reference to the validity information. In Switzerland: 000000 = always.
///
/// ### Example (excerpt):
///
/// `
/// *Z ...
/// *G ...
/// *A VE 8500090 8503000 001417 % Ab HS-Nr. 8500090 bis HS-Nr. 8503000, gelten die Gültigkeitstage 001417 (Bitfeld für bspw. alle Montage)
/// ...
/// `
fn row_a_ve_combinator<'a>() -> impl Parser<
    &'a str,
    Output = (Option<i32>, char, Option<i32>, char, Option<i32>),
    Error = nom::error::Error<&'a str>,
> {
    preceded(
        tag("*A VE "),
        (
            optional_i32_from_n_digits_parser(7),
            char(' '),
            optional_i32_from_n_digits_parser(7),
            char(' '),
            optional_i32_from_n_digits_parser(6),
        ),
    )
}

/// ## A *-lines
///
/// - *A *-lines: Reference to offers (s. file ATTRIBUT). It includes:
///     - The offer code
///         - The term “Angebot” (offer) may be imprecise here. The HRDF doc. uses the word “Attribut” (attribute), which is also somewhat imprecise. Basically, it is a collective term for extensions (e.g. dining car) or restrictions (e.g. no bicycles) that apply.
///     - Stop from which the offer category applies
///     - Stop up to which the offer category applies
///     - Reference to the validity information
///
/// ### Example (excerpt):
///
/// `
/// *Z ...
/// *G ...
/// *A VE ...
/// *A R  8500090 8503000        % Attribut R gilt ab HS-Nr. 8500090 bis HS-Nr. 8503000
/// *A WR 8500090 8503000 047873 % Attribut WR gilt ab HS-Nr. 8500090 bis HS-Nr. 8503000 mit den Gültigkeitstagen 047873
/// *A VR 8500090 8503000        % Attribut VR gilt ab HS-Nr. 8500090 bis HS-Nr. 8503000
/// ...
/// `
///
fn row_a_combinator<'a>() -> impl Parser<
    &'a str,
    Output = (
        String,
        char,
        Option<i32>,
        char,
        Option<i32>,
        char,
        Option<i32>,
    ),
    Error = nom::error::Error<&'a str>,
> {
    preceded(
        tag("*A "),
        (
            string_from_n_chars_parser(2),
            char(' '),
            optional_i32_from_n_digits_parser(7),
            char(' '),
            optional_i32_from_n_digits_parser(7),
            char(' '),
            optional_i32_from_n_digits_parser(6),
        ),
    )
}

/// ## I-lines
///
/// - *I-lines: Reference to notes (s. INFOTEXT file). Further details on this topic and its implementation in Switzerland can be found in the RV. It includes:
///     - Informational text code. In Switzerland: XI not supported. Permitted codes see list (DE / FR).
///     - Stop from which the info text applies
///     - Stop up to which the info text applies
///     - Reference to the validity information. In Switzerland: If not available = always.
///     - Reference to the info text
///     - Departure time
///     - Time of arrival
///     - Comments:
///         - The Swiss Journey ID (SJYID) is identified via the *I line with the code JY
///
/// ### Example (excerpt):
///
/// `
/// *Z ...
/// *G ...
/// *A VE ...
/// *A ...
/// *I hi 8573602 8587744        000018040             % Hinweis auf Infotext (hi) ab HS-Nr. 8573602 bis HS-Nr. 8587744  mit Infotext 18040
/// *I hi 8578157 8589334        000018037 01126 01159 % Hinweis auf Infotext (hi) ab HS-Nr. 8578157 bis HS-Nr. 8589334 mit Infotext 18037 Abfahrt 11:26 Ankunft 11:59
/// *I JY                        000000000                     %"
/// ...
/// `
///
fn row_i_combinator<'a>() -> impl Parser<
    &'a str,
    Output = (
        String,
        char,
        Option<i32>,
        char,
        Option<i32>,
        char,
        Option<i32>,
        char,
        i32,
        char,
        Option<i32>,
        char,
        Option<i32>,
    ),
    Error = nom::error::Error<&'a str>,
> {
    preceded(
        tag("*I "),
        (
            string_from_n_chars_parser(2),
            char(' '),
            optional_i32_from_n_digits_parser(7),
            char(' '),
            optional_i32_from_n_digits_parser(7),
            char(' '),
            optional_i32_from_n_digits_parser(6),
            char(' '),
            i32_from_n_digits_parser(9),
            char(' '),
            optional_i32_from_n_digits_parser(6),
            char(' '),
            optional_i32_from_n_digits_parser(6),
        ),
    )
}

/// ## L-lines
///
/// - *L lines: Line information or reference to the line information (see file LINIE). It includes:
///     - Line information, reference to external file if necessary.
///     - Stop from which the line is valid
///     - Stop to which the line is valid
///     - Departure time
///     - Time of arrival
///
/// ### Example (excerpt):
///
/// `
/// *Z ...
/// *G ...
/// *A VE ...
/// *A ...
/// *I ...
/// *L 8        8578157 8589334 01126 01159 % Linie 8 ab HS-Nr. 8578157 bis HS-Nr. 8589334 Abfahrt 11:26 Ankunft 11:59
/// *L #0000022 8589601 8589913             % Referenz auf Linie No. 22 ab HS-Nr. 8589601 bis HS-Nr. 8589913
/// ...
/// `
fn row_l_combinator<'a>() -> impl Parser<
    &'a str,
    Output = (
        String,
        char,
        Option<i32>,
        char,
        Option<i32>,
        char,
        Option<i32>,
        char,
        Option<i32>,
    ),
    Error = nom::error::Error<&'a str>,
> {
    preceded(
        tag("*L "),
        (
            string_from_n_chars_parser(8),
            char(' '),
            optional_i32_from_n_digits_parser(7),
            char(' '),
            optional_i32_from_n_digits_parser(7),
            char(' '),
            optional_i32_from_n_digits_parser(6),
            char(' '),
            optional_i32_from_n_digits_parser(6),
        ),
    )
}

/// ## R-lines
///
/// - *R lines: Reference to the direction text (see file RICHTUNG / DIRECTION). It includes:
///     - Direction (H=forward,R=backward)
///     - Reference to direction code
///     - Stop from which the direction applies
///     - Stop to which the direction applies
///     - Departure time
///     - Time of arrival
///     - Comments:
///         - R without information = no direction
///
/// ### Example (excerpt):
///
/// `
/// *Z ...
/// *G ...
/// *A VE ...
/// *A ...
/// *I ...
/// *L ...
/// *R H                                     % gilt für die gesamte Hin-Richtung
/// *R R R000063 1300146 8574808             % gilt für Rück-Richtung 63 ab HS-Nr. 1300146 bis HS-Nr. 8574808
/// ...
/// `
fn row_r_combinator<'a>() -> impl Parser<
    &'a str,
    Output = (
        String,
        char,
        String,
        char,
        Option<i32>,
        char,
        Option<i32>,
        char,
        Option<i32>,
        char,
        Option<i32>,
    ),
    Error = nom::error::Error<&'a str>,
> {
    preceded(
        tag("*R "),
        (
            string_from_n_chars_parser(1),
            char(' '),
            string_from_n_chars_parser(7),
            char(' '),
            optional_i32_from_n_digits_parser(7),
            char(' '),
            optional_i32_from_n_digits_parser(7),
            char(' '),
            optional_i32_from_n_digits_parser(6),
            char(' '),
            optional_i32_from_n_digits_parser(6),
        ),
    )
}

/// ## CI/CO lines
///
/// - *CI/CO lines: It includes:
///     - Number of minutes at check-in(CI)/out(CO)
///     - Stop from which the direction applies
///     - Stop to which the direction applies
///     - Departure time
///     - Time of arrival
///
/// ### Example (excerpt):
///
/// `
/// *Z ...
/// *G ...
/// *A VE ...
/// *A ...
/// *I ...
/// *L ...
/// *R ...
/// *CI 0002 8507000 8507000                                   % Check-in 2 Min. ab HS-Nr. 8507000 bis HS-Nr. 8507000
/// ...
/// *CO 0002 8507000 8507000                                   % Check-out 2 Min. ab HS-Nr. 8507000 bis HS-Nr. 8507000
/// ...
/// `
fn row_ci_co_combinator<'a>() -> impl Parser<
    &'a str,
    Output = (
        &'a str,
        char,
        i32,
        char,
        Option<i32>,
        char,
        Option<i32>,
        char,
        Option<i32>,
        char,
        Option<i32>,
    ),
    Error = nom::error::Error<&'a str>,
> {
    (
        alt((tag("*CI"), tag("*CO"))),
        char(' '),
        i32_from_n_digits_parser(4),
        char(' '),
        optional_i32_from_n_digits_parser(7),
        char(' '),
        optional_i32_from_n_digits_parser(7),
        char(' '),
        optional_i32_from_n_digits_parser(6),
        char(' '),
        optional_i32_from_n_digits_parser(6),
    )
}

/// ## Journey description
///
/// - Once all the lines described have been defined, the run is described with the journey times:
///     - Stop (s. BAHNHOF and others)
///     - Arrival time: Negative = No possibility to get out
///     - Departure time: Negative = No boarding option
///     - Journey number
///     - Administration
///
/// ### Example (excerpt):
///
/// `
/// *Z ...
/// *G ...
/// *A VE ...
/// *A ...
/// *I ...
/// *L ...
/// *R ...
/// *CI ...
/// *CO ...
/// 0053301 S Wannsee DB                 02014               % HS-Nr. 0053301 Ankunft N/A,   Abfahrt 20:14
/// 0053291 Wannseebrücke         02015  02015 052344 80____ % HS-Nr. 0053291 Ankunft 20:15, Abfahrt 20:15, Fahrtnummer 052344, Verwaltung 80____ (DB)
/// 0053202 Am Kl. Wannsee/Am Gr  02016  02016               %
/// `
///
fn row_journey_description_combinator<'a>() -> impl Parser<
    &'a str,
    Output = (
        i32,
        char,
        String,
        char,
        Option<i32>,
        char,
        Option<i32>,
        char,
        Option<i32>,
        char,
        String,
    ),
    Error = nom::error::Error<&'a str>,
> {
    (
        i32_from_n_digits_parser(7),
        char(' '),
        string_from_n_chars_parser(20),
        char(' '),
        optional_i32_from_n_digits_parser(6),
        char(' '),
        optional_i32_from_n_digits_parser(6),
        char(' '),
        optional_i32_from_n_digits_parser(6),
        char(' '),
        string_from_n_chars_parser(6),
    )
}

fn parse_line(
    line: &str,
    data: &Rc<RefCell<Vec<Journey>>>,
    pk_type_converter: &Rc<RefCell<FxHashSet<JourneyId>>>,
    auto_increment: &AutoIncrement,
    transport_types_pk_type_converter: &FxHashMap<String, i32>,
    attributes_pk_type_converter: &FxHashMap<String, i32>,
    directions_pk_type_converter: &FxHashMap<String, i32>,
) -> Result<(), Box<dyn Error>> {
    let _ = alt((
        map_res(
            row_z_combinator(),
            |(
                journey_id,
                _,
                transport_company_id,
                _,
                _transport_variant,
                _,
                _num_cycles,
                _,
                _cycle_dura_min,
            )| {
                let local_data = Rc::clone(data);
                let id = auto_increment.next();

                pk_type_converter
                    .borrow_mut()
                    .insert((journey_id, transport_company_id.to_owned()));

                local_data
                    .borrow_mut()
                    .push(Journey::new(id, journey_id, transport_company_id));
                Ok::<(), Box<dyn Error>>(())
            },
        ),
        map_res(
            row_g_combinator(),
            |(offer, _, stop_from_id, _, stop_to_id)| {
                let local_data = Rc::clone(data);
                let mut local_data = local_data.borrow_mut();
                let journey = local_data.last_mut().ok_or("Type A row missing.")?;
                let transport_type_id = *transport_types_pk_type_converter
                    .get(&offer)
                    .ok_or("Unknown legacy ID")?;

                journey.add_metadata_entry(
                    JourneyMetadataType::TransportType,
                    JourneyMetadataEntry::new(
                        stop_from_id,
                        stop_to_id,
                        Some(transport_type_id),
                        None,
                        None,
                        None,
                        None,
                        None,
                    ),
                );
                Ok::<(), Box<dyn Error>>(())
            },
        ),
        map_res(
            row_a_ve_combinator(),
            |(stop_from_id, _, stop_to_id, _, bit_field_id)| {
                let local_data = Rc::clone(data);
                let mut local_data = local_data.borrow_mut();
                let journey = local_data.last_mut().ok_or("Type A row missing.")?;
                journey.add_metadata_entry(
                    JourneyMetadataType::BitField,
                    JourneyMetadataEntry::new(
                        stop_from_id,
                        stop_to_id,
                        None,
                        bit_field_id,
                        None,
                        None,
                        None,
                        None,
                    ),
                );
                Ok::<(), Box<dyn Error>>(())
            },
        ),
        map_res(
            row_a_combinator(),
            |(offer, _, stop_from_id, _, stop_to_id, _, _reference)| {
                let local_data = Rc::clone(data);
                let mut local_data = local_data.borrow_mut();
                let journey = local_data.last_mut().ok_or("Type A row missing.")?;
                let attribute_id = *attributes_pk_type_converter
                    .get(&offer)
                    .ok_or("Unknown legacy ID")?;

                journey.add_metadata_entry(
                    JourneyMetadataType::Attribute,
                    JourneyMetadataEntry::new(
                        stop_from_id,
                        stop_to_id,
                        Some(attribute_id),
                        None,
                        None,
                        None,
                        None,
                        None,
                    ),
                );
                Ok::<(), Box<dyn Error>>(())
            },
        ),
        map_res(
            row_i_combinator(),
            |(
                info_code,
                _,
                stop_from_id,
                _,
                stop_to_id,
                _,
                validity_ref,
                _,
                info_ref,
                _,
                departure_time,
                _,
                arrival_time,
            )| {
                let local_data = Rc::clone(data);
                let mut local_data = local_data.borrow_mut();
                let journey = local_data.last_mut().ok_or("Type A row missing.")?;
                let arrival_time = create_time(arrival_time);
                let departure_time = create_time(departure_time);

                journey.add_metadata_entry(
                    JourneyMetadataType::InformationText,
                    JourneyMetadataEntry::new(
                        stop_from_id,
                        stop_to_id,
                        Some(info_ref),
                        validity_ref,
                        departure_time,
                        arrival_time,
                        Some(info_code),
                        None,
                    ),
                );
                Ok::<(), Box<dyn Error>>(())
            },
        ),
        map_res(
            row_l_combinator(),
            |(
                mut line_info,
                _,
                stop_from_id,
                _,
                stop_to_id,
                _,
                departure_time,
                _,
                arrival_time,
            )| {
                let local_data = Rc::clone(data);
                let mut local_data = local_data.borrow_mut();
                let journey = local_data.last_mut().ok_or("Type A row missing.")?;
                let arrival_time = create_time(arrival_time);
                let departure_time = create_time(departure_time);

                let line_info_first_char = line_info
                    .chars()
                    .next()
                    .ok_or("Missing line info (the string is empty).")?;

                let (resource_id, extra_field_1) = if line_info_first_char == '#' {
                    line_info.drain(..line_info_first_char.len_utf8());
                    (Some(line_info.parse::<i32>()?), None)
                } else {
                    (None, Some(line_info))
                };

                journey.add_metadata_entry(
                    JourneyMetadataType::Line,
                    JourneyMetadataEntry::new(
                        stop_from_id,
                        stop_to_id,
                        resource_id,
                        None,
                        departure_time,
                        arrival_time,
                        extra_field_1,
                        None,
                    ),
                );

                Ok::<(), Box<dyn Error>>(())
            },
        ),
        map_res(
            row_r_combinator(),
            |(
                direction,
                _,
                ref_direction_code,
                _,
                stop_from_id,
                _,
                stop_to_id,
                _,
                departure_time,
                _,
                arrival_time,
            )| {
                let local_data = Rc::clone(data);
                let mut local_data = local_data.borrow_mut();
                let journey = local_data.last_mut().ok_or("Type A row missing.")?;
                let arrival_time = create_time(arrival_time);
                let departure_time = create_time(departure_time);

                let direction_id = if ref_direction_code.is_empty() {
                    None
                } else {
                    let id = *directions_pk_type_converter
                        .get(&ref_direction_code)
                        .ok_or("Unknown legacy ID")?;
                    Some(id)
                };

                journey.add_metadata_entry(
                    JourneyMetadataType::Direction,
                    JourneyMetadataEntry::new(
                        stop_from_id,
                        stop_to_id,
                        direction_id,
                        None,
                        departure_time,
                        arrival_time,
                        Some(direction),
                        None,
                    ),
                );
                Ok::<(), Box<dyn Error>>(())
            },
        ),
        map_res(
            row_ci_co_combinator(),
            |(
                ci_co,
                _,
                num_minutes,
                _,
                stop_from_id,
                _,
                stop_to_id,
                _,
                departure_time,
                _,
                arrival_time,
            )| {
                let local_data = Rc::clone(data);
                let mut local_data = local_data.borrow_mut();
                let journey = local_data.last_mut().ok_or("Type A row missing.")?;
                let arrival_time = create_time(arrival_time);
                let departure_time = create_time(departure_time);

                let metadata_type = if ci_co == "*CI" {
                    JourneyMetadataType::ExchangeTimeBoarding
                } else {
                    JourneyMetadataType::ExchangeTimeDisembarking
                };

                journey.add_metadata_entry(
                    metadata_type,
                    JourneyMetadataEntry::new(
                        stop_from_id,
                        stop_to_id,
                        None,
                        None,
                        departure_time,
                        arrival_time,
                        None,
                        Some(num_minutes),
                    ),
                );
                Ok::<(), Box<dyn Error>>(())
            },
        ),
        map_res(
            row_journey_description_combinator(),
            |(
                stop_id,
                _,
                _stop_name,
                _,
                arrival_time,
                _,
                departure_time,
                _,
                _journey_id,
                _,
                _administration,
            )| {
                let local_data = Rc::clone(data);
                let mut local_data = local_data.borrow_mut();
                let journey = local_data.last_mut().ok_or("Type A row missing.")?;
                let arrival_time = create_time(arrival_time);
                let departure_time = create_time(departure_time);

                journey.add_route_entry(JourneyRouteEntry::new(
                    stop_id,
                    arrival_time,
                    departure_time,
                ));

                Ok::<(), Box<dyn Error>>(())
            },
        ),
    ))
    .parse(line)
    .map_err(|e| format!("Failed to parse line '{}': {}", line, e))?;
    Ok::<(), Box<dyn Error>>(())
}

pub fn parse(
    path: &str,
    transport_types_pk_type_converter: &FxHashMap<String, i32>,
    attributes_pk_type_converter: &FxHashMap<String, i32>,
    directions_pk_type_converter: &FxHashMap<String, i32>,
) -> Result<JourneyAndTypeConverter, Box<dyn Error>> {
    log::info!("Parsing FPLAN...");
    let lines = read_lines(&format!("{path}/FPLAN"), 0)?;

    let auto_increment = AutoIncrement::new();
    let data = Rc::new(RefCell::new(Vec::new()));
    let pk_type_converter = Rc::new(RefCell::new(FxHashSet::default()));

    lines
        .into_iter()
        .filter(|line| !line.trim().is_empty())
        .try_for_each(|line| {
            parse_line(
                &line,
                &data,
                &pk_type_converter,
                &auto_increment,
                transport_types_pk_type_converter,
                attributes_pk_type_converter,
                directions_pk_type_converter,
            )
        })?;

    let data =
        RefCell::<Vec<Journey>>::into_inner(Rc::into_inner(data).ok_or("Unable to get data")?);
    let pk_type_converter = RefCell::<FxHashSet<JourneyId>>::into_inner(
        Rc::into_inner(pk_type_converter).ok_or("Unable to get pk_type_converter")?,
    );

    let data = Journey::vec_to_map(data);
    Ok((ResourceStorage::new(data), pk_type_converter))
}

// ------------------------------------------------------------------------------------------------
// --- Helper Functions
// ------------------------------------------------------------------------------------------------

fn create_time(time: Option<i32>) -> Option<NaiveTime> {
    time.map(|value| {
        create_time_from_value(match value.abs() {
            val if val >= 2400 => val % 2400,
            val => val,
        } as u32)
    })
}

#[cfg(test)]
mod tests {
    use crate::parsing::tests::get_json_values;

    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;
    //use crate::parsing::tests::get_json_values;
    use nom::IResult;
    use pretty_assertions::assert_eq;

    #[test]
    fn parsing_rows() {
        let rows = vec![
            "*Z 002359 000011   101                                     % -- 37649518273 --"
                .to_string(),
            "*G IR  8507000 8509000                                     %".to_string(),
            "*A VE 8507000 8509000 348508                               %".to_string(),
            "*A FS 8507000 8509000                                      %".to_string(),
            "*I JY                        000001370                     %".to_string(),
            "*L 35       8507000 8509000                                %".to_string(),
            "*R H                                                       %".to_string(),
            "*CI 0002 8507000 8507000                                   %".to_string(),
            "8507000 Bern                         00638                 %".to_string(),
            "8508005 Burgdorf              00652  00653                 %".to_string(),
            "8508008 Herzogenbuchsee       00704  00705                 %".to_string(),
            "8509000 Chur                  00948                        %".to_string(),
        ];
        let auto_increment = AutoIncrement::new();
        let data = Rc::new(RefCell::new(Vec::new()));
        let pk_type_converter = Rc::new(RefCell::new(FxHashSet::default()));
        let mut transport_types_pk_type_converter = FxHashMap::<String, i32>::default();
        transport_types_pk_type_converter.insert("IR".to_string(), 100);
        let mut attributes_pk_type_converter = FxHashMap::<String, i32>::default();
        attributes_pk_type_converter.insert("FS".to_string(), 100);
        let directions_pk_type_converter = FxHashMap::<String, i32>::default();

        for line in rows {
            parse_line(
                &line,
                &data,
                &pk_type_converter,
                &auto_increment,
                &transport_types_pk_type_converter,
                &attributes_pk_type_converter,
                &directions_pk_type_converter,
            )
            .unwrap();
        }

        // First row (id: 1)
        let reference = r#"
        {
          "id": 1,
          "legacy_id": 2359,
          "administration": "000011",
          "metadata": {
            "Attribute": [
              {
                "from_stop_id": 8507000,
                "until_stop_id": 8509000,
                "resource_id": 100,
                "bit_field_id": null,
                "departure_time": null,
                "arrival_time": null,
                "extra_field_1": null,
                "extra_field_2": null
              }
            ],
            "TransportType": [
              {
                "from_stop_id": 8507000,
                "until_stop_id": 8509000,
                "resource_id": 100,
                "bit_field_id": null,
                "departure_time": null,
                "arrival_time": null,
                "extra_field_1": null,
                "extra_field_2": null
              }
            ],
            "InformationText": [
              {
                "from_stop_id": null,
                "until_stop_id": null,
                "resource_id": 1370,
                "bit_field_id": null,
                "departure_time": null,
                "arrival_time": null,
                "extra_field_1": "JY",
                "extra_field_2": null
              }
            ],
            "BitField": [
              {
                "from_stop_id": 8507000,
                "until_stop_id": 8509000,
                "resource_id": null,
                "bit_field_id": 348508,
                "departure_time": null,
                "arrival_time": null,
                "extra_field_1": null,
                "extra_field_2": null
              }
            ],
            "ExchangeTimeBoarding": [
              {
                "from_stop_id": 8507000,
                "until_stop_id": 8507000,
                "resource_id": null,
                "bit_field_id": null,
                "departure_time": null,
                "arrival_time": null,
                "extra_field_1": null,
                "extra_field_2": 2
              }
            ],
            "Line": [
              {
                "from_stop_id": 8507000,
                "until_stop_id": 8509000,
                "resource_id": null,
                "bit_field_id": null,
                "departure_time": null,
                "arrival_time": null,
                "extra_field_1": "35",
                "extra_field_2": null
              }
            ],
            "Direction": [
              {
                "from_stop_id": null,
                "until_stop_id": null,
                "resource_id": null,
                "bit_field_id": null,
                "departure_time": null,
                "arrival_time": null,
                "extra_field_1": "H",
                "extra_field_2": null
              }
            ]
          },
          "route": [
            {
              "stop_id": 8507000,
              "arrival_time": null,
              "departure_time": "06:38:00"
            },
            {
              "stop_id": 8508005,
              "arrival_time": "06:52:00",
              "departure_time": "06:53:00"
            },
            {
              "stop_id": 8508008,
              "arrival_time": "07:04:00",
              "departure_time": "07:05:00"
            },
            {
              "stop_id": 8509000,
              "arrival_time": "09:48:00",
              "departure_time": null
            }
          ]
        }"#;

        let data = RefCell::<Vec<Journey>>::into_inner(
            Rc::into_inner(data).ok_or("Unable to get data").unwrap(),
        );
        let (attribute, reference) = get_json_values(&data[0], reference).unwrap();
        assert_eq!(attribute, reference);
    }

    mod row_z {
        // Note this useful idiom: importing names from outer (for mod tests) scope.
        use super::*;
        use pretty_assertions::assert_eq;

        fn row_z_parser(
            input: &str,
        ) -> IResult<&str, (i32, String, i32, Option<i32>, Option<i32>)> {
            let (
                res,
                (
                    journey_id,
                    _,
                    transport_company_id,
                    _,
                    transport_variant,
                    _,
                    num_cycles,
                    _,
                    cycle_dura_min,
                ),
            ) = row_z_combinator().parse(input)?;
            Ok((
                res,
                (
                    journey_id,
                    transport_company_id,
                    transport_variant,
                    num_cycles,
                    cycle_dura_min,
                ),
            ))
        }

        #[test]
        fn success_no_options() {
            let input = "*Z 000003 000011   101         % Fahrtnummer 3, für TU 11 (SBB), mit Variante 101 (ignore)";
            let (
                res,
                (journey_id, transport_company_id, transport_variant, num_cycles, cycle_dura_min),
            ) = row_z_parser(input).unwrap();
            assert_eq!(3, journey_id);
            assert_eq!("000011", transport_company_id);
            assert_eq!(101, transport_variant);
            assert_eq!(None, num_cycles);
            assert_eq!(None, cycle_dura_min);
            assert_eq!(
                res.trim(),
                "% Fahrtnummer 3, für TU 11 (SBB), mit Variante 101 (ignore)"
            );
        }

        #[test]
        fn success_with_options() {
            let input = "*Z 123456 000011   101 012 060 % Fahrtnummer 123456, für TU 11 (SBB), mit Variante 101 (ignore), 12 mal, alle 60 Minuten";
            let (
                res,
                (journey_id, transport_company_id, transport_variant, num_cycles, cycle_dura_min),
            ) = row_z_parser(input).unwrap();
            assert_eq!(123456, journey_id);
            assert_eq!("000011", transport_company_id);
            assert_eq!(101, transport_variant);
            assert_eq!(Some(12), num_cycles);
            assert_eq!(Some(60), cycle_dura_min);
            assert_eq!(
                res.trim(),
                "% Fahrtnummer 123456, für TU 11 (SBB), mit Variante 101 (ignore), 12 mal, alle 60 Minuten"
            );
        }
    }

    mod row_g {
        // Note this useful idiom: importing names from outer (for mod tests) scope.
        use super::*;
        use pretty_assertions::assert_eq;

        fn row_g_parser(input: &str) -> IResult<&str, (String, Option<i32>, Option<i32>)> {
            let (res, (offer, _, stop_from_id, _, stop_to_id)) = row_g_combinator().parse(input)?;
            Ok((res, (offer, stop_from_id, stop_to_id)))
        }

        #[test]
        fn success_with_options() {
            let input = "*G ICE 8500090 8503000 % Angebotskategorie ICE gilt ab HS-Nr. 8500090 bis HS-Nr. 8503000";

            let (res, (offer, stop_from_id, stop_to_id)) = row_g_parser(input).unwrap();
            assert_eq!("ICE", offer);
            assert_eq!(Some(8500090), stop_from_id);
            assert_eq!(Some(8503000), stop_to_id);
            assert_eq!(
                res.trim(),
                "% Angebotskategorie ICE gilt ab HS-Nr. 8500090 bis HS-Nr. 8503000"
            );
        }

        #[test]
        fn success_no_options() {
            let input = "*G ICE                 % Angebotskategorie ICE gilt ab HS-Nr. 8500090 bis HS-Nr. 8503000";

            let (res, (offer, stop_from_id, stop_to_id)) = row_g_parser(input).unwrap();
            assert_eq!("ICE", offer);
            assert_eq!(None, stop_from_id);
            assert_eq!(None, stop_to_id);
            assert_eq!(
                res.trim(),
                "% Angebotskategorie ICE gilt ab HS-Nr. 8500090 bis HS-Nr. 8503000"
            );
        }
    }

    mod row_a_ve {
        // Note this useful idiom: importing names from outer (for mod tests) scope.
        use super::*;
        use pretty_assertions::assert_eq;

        fn row_a_ve_parser(input: &str) -> IResult<&str, (Option<i32>, Option<i32>, Option<i32>)> {
            let (res, (stop_from_id, _, stop_to_id, _, reference)) =
                row_a_ve_combinator().parse(input)?;
            Ok((res, (stop_from_id, stop_to_id, reference)))
        }

        #[test]
        fn success_with_options() {
            let input = "*A VE 8500090 8503000 001417 % Ab HS-Nr. 8500090 bis HS-Nr. 8503000, gelten die Gültigkeitstage 001417 (Bitfeld für bspw. alle Montage)";
            let (res, (stop_from_id, stop_to_id, reference)) = row_a_ve_parser(input).unwrap();

            assert_eq!(Some(8500090), stop_from_id);
            assert_eq!(Some(8503000), stop_to_id);
            assert_eq!(Some(1417), reference);
            assert_eq!(
                res.trim(),
                "% Ab HS-Nr. 8500090 bis HS-Nr. 8503000, gelten die Gültigkeitstage 001417 (Bitfeld für bspw. alle Montage)"
            );
        }

        #[test]
        fn success_no_options() {
            let input = "*A VE                        % Ab HS-Nr. 8500090 bis HS-Nr. 8503000, gelten die Gültigkeitstage 001417 (Bitfeld für bspw. alle Montage)";
            let (res, (stop_from_id, stop_to_id, reference)) = row_a_ve_parser(input).unwrap();

            assert_eq!(None, stop_from_id);
            assert_eq!(None, stop_to_id);
            assert_eq!(None, reference);
            assert_eq!(
                res.trim(),
                "% Ab HS-Nr. 8500090 bis HS-Nr. 8503000, gelten die Gültigkeitstage 001417 (Bitfeld für bspw. alle Montage)"
            );
        }
    }

    mod row_a {
        // Note this useful idiom: importing names from outer (for mod tests) scope.
        use super::*;
        use pretty_assertions::assert_eq;

        fn row_a_parser(
            input: &str,
        ) -> IResult<&str, (String, Option<i32>, Option<i32>, Option<i32>)> {
            let (res, (offer, _, stop_from_id, _, stop_to_id, _, reference)) =
                row_a_combinator().parse(input)?;
            Ok((res, (offer, stop_from_id, stop_to_id, reference)))
        }

        #[test]
        fn success_with_partial_options1() {
            let input = "*A R  8500090 8503000        % Attribut R gilt ab HS-Nr. 8500090 bis HS-Nr. 8503000";
            let (res, (offer, stop_from_id, stop_to_id, reference)) = row_a_parser(input).unwrap();

            assert_eq!("R", offer);
            assert_eq!(Some(8500090), stop_from_id);
            assert_eq!(Some(8503000), stop_to_id);
            assert_eq!(None, reference);
            assert_eq!(
                res.trim(),
                "% Attribut R gilt ab HS-Nr. 8500090 bis HS-Nr. 8503000"
            );
        }

        #[test]
        fn success_partial_options() {
            let input = "*A VR 8500090 8503000        % Attribut VR gilt ab HS-Nr. 8500090 bis HS-Nr. 8503000";
            let (res, (offer, stop_from_id, stop_to_id, reference)) = row_a_parser(input).unwrap();

            assert_eq!("VR", offer);
            assert_eq!(Some(8500090), stop_from_id);
            assert_eq!(Some(8503000), stop_to_id);
            assert_eq!(None, reference);
            assert_eq!(
                res.trim(),
                "% Attribut VR gilt ab HS-Nr. 8500090 bis HS-Nr. 8503000"
            );
        }

        #[test]
        fn success_with_options() {
            let input = "*A WR 8500090 8503000 047873 % Attribut WR gilt ab HS-Nr. 8500090 bis HS-Nr. 8503000 mit den Gültigkeitstagen 047873";
            let (res, (offer, stop_from_id, stop_to_id, reference)) = row_a_parser(input).unwrap();

            assert_eq!("WR", offer);
            assert_eq!(Some(8500090), stop_from_id);
            assert_eq!(Some(8503000), stop_to_id);
            assert_eq!(Some(47873), reference);
            assert_eq!(
                res.trim(),
                "% Attribut WR gilt ab HS-Nr. 8500090 bis HS-Nr. 8503000 mit den Gültigkeitstagen 047873"
            );
        }
    }

    mod row_i {
        // Note this useful idiom: importing names from outer (for mod tests) scope.
        use super::*;
        use pretty_assertions::assert_eq;

        fn row_i_parser(
            input: &str,
        ) -> IResult<
            &str,
            (
                String,
                Option<i32>,
                Option<i32>,
                Option<i32>,
                i32,
                Option<i32>,
                Option<i32>,
            ),
        > {
            let (
                res,
                (
                    info_code,
                    _,
                    stop_from_id,
                    _,
                    stop_to_id,
                    _,
                    validity_ref,
                    _,
                    info_ref,
                    _,
                    departure_time,
                    _,
                    arrival_time,
                ),
            ) = row_i_combinator().parse(input)?;
            Ok((
                res,
                (
                    info_code,
                    stop_from_id,
                    stop_to_id,
                    validity_ref,
                    info_ref,
                    departure_time,
                    arrival_time,
                ),
            ))
        }

        #[test]
        fn success_with_partial_options() {
            let input = "*I hi 8573602 8587744        000018040             % Hinweis auf Infotext (hi) ab HS-Nr. 8573602 bis HS-Nr. 8587744  mit Infotext 18040";
            let (
                res,
                (
                    info_code,
                    stop_from_id,
                    stop_to_id,
                    validity_ref,
                    info_ref,
                    departure_time,
                    arrival_time,
                ),
            ) = row_i_parser(input).unwrap();
            assert_eq!("hi", info_code);
            assert_eq!(Some(8573602), stop_from_id);
            assert_eq!(Some(8587744), stop_to_id);
            assert_eq!(None, validity_ref);
            assert_eq!(18040, info_ref);
            assert_eq!(None, departure_time);
            assert_eq!(None, arrival_time);
            assert_eq!(
                res.trim(),
                "% Hinweis auf Infotext (hi) ab HS-Nr. 8573602 bis HS-Nr. 8587744  mit Infotext 18040"
            );
        }

        #[test]
        fn success_with_options() {
            let input = "*I hi 8578157 8589334        000018037  01126  01159 % Hinweis auf Infotext (hi) ab HS-Nr. 8578157 bis HS-Nr. 8589334 mit Infotext 18037 Abfahrt 11:26 Ankunft 11:59";
            let (
                res,
                (
                    info_code,
                    stop_from_id,
                    stop_to_id,
                    validity_ref,
                    info_ref,
                    departure_time,
                    arrival_time,
                ),
            ) = row_i_parser(input).unwrap();
            assert_eq!("hi", info_code);
            assert_eq!(Some(8578157), stop_from_id);
            assert_eq!(Some(8589334), stop_to_id);
            assert_eq!(None, validity_ref);
            assert_eq!(18037, info_ref);
            assert_eq!(Some(1126), departure_time);
            assert_eq!(Some(1159), arrival_time);
            assert_eq!(
                res.trim(),
                "% Hinweis auf Infotext (hi) ab HS-Nr. 8578157 bis HS-Nr. 8589334 mit Infotext 18037 Abfahrt 11:26 Ankunft 11:59"
            );
        }
    }

    mod row_l {
        // Note this useful idiom: importing names from outer (for mod tests) scope.
        use super::*;
        use pretty_assertions::assert_eq;

        fn row_l_parser(
            input: &str,
        ) -> IResult<&str, (String, Option<i32>, Option<i32>, Option<i32>, Option<i32>)> {
            let (
                res,
                (line_info, _, stop_from_id, _, stop_to_id, _, departure_time, _, arrival_time),
            ) = row_l_combinator().parse(input)?;
            Ok((
                res,
                (
                    line_info,
                    stop_from_id,
                    stop_to_id,
                    departure_time,
                    arrival_time,
                ),
            ))
        }

        #[test]
        fn success_with_options() {
            let input = "*L 8        8578157 8589334  01126  01159 % Linie 8 ab HS-Nr. 8578157 bis HS-Nr. 8589334 Abfahrt 11:26 Ankunft 11:59";
            let (res, (line_info, stop_from_id, stop_to_id, departure_time, arrival_time)) =
                row_l_parser(input).unwrap();
            assert_eq!("8", line_info);
            assert_eq!(Some(8578157), stop_from_id);
            assert_eq!(Some(8589334), stop_to_id);
            assert_eq!(Some(1126), departure_time);
            assert_eq!(Some(1159), arrival_time);
            assert_eq!(
                "% Linie 8 ab HS-Nr. 8578157 bis HS-Nr. 8589334 Abfahrt 11:26 Ankunft 11:59",
                res.trim()
            );
        }

        #[test]
        fn success_with_partial_options() {
            let input = "*L #0000022 8589601 8589913             % Referenz auf Linie No. 22 ab HS-Nr. 8589601 bis HS-Nr. 8589913";
            let (res, (line_info, stop_from_id, stop_to_id, departure_time, arrival_time)) =
                row_l_parser(input).unwrap();
            assert_eq!("#0000022", line_info);
            assert_eq!(Some(8589601), stop_from_id);
            assert_eq!(Some(8589913), stop_to_id);
            assert_eq!(None, departure_time);
            assert_eq!(None, arrival_time);
            assert_eq!(
                "% Referenz auf Linie No. 22 ab HS-Nr. 8589601 bis HS-Nr. 8589913",
                res.trim()
            );
        }

        #[test]
        fn success_with_partial_options61() {
            let input = "*L 61       8500010 8507492                                %";
            let (res, (line_info, stop_from_id, stop_to_id, departure_time, arrival_time)) =
                row_l_parser(input).unwrap();
            assert_eq!("61", line_info);
            assert_eq!(Some(8500010), stop_from_id);
            assert_eq!(Some(8507492), stop_to_id);
            assert_eq!(None, departure_time);
            assert_eq!(None, arrival_time);
            assert_eq!("%", res.trim());
        }
    }

    mod row_r {
        // Note this useful idiom: importing names from outer (for mod tests) scope.
        use super::*;
        use pretty_assertions::assert_eq;

        fn row_r_parser(
            input: &str,
        ) -> IResult<
            &str,
            (
                String,
                String,
                Option<i32>,
                Option<i32>,
                Option<i32>,
                Option<i32>,
            ),
        > {
            let (
                res,
                (
                    direction,
                    _,
                    ref_direction_code,
                    _,
                    stop_from_id,
                    _,
                    stop_to_id,
                    _,
                    departure_time,
                    _,
                    arrival_time,
                ),
            ) = row_r_combinator().parse(input)?;
            Ok((
                res.trim(), // res contains the comments that are useful to determine the direction
                (
                    direction,
                    ref_direction_code,
                    stop_from_id,
                    stop_to_id,
                    departure_time,
                    arrival_time,
                ),
            ))
        }

        #[test]
        fn success_with_options() {
            let input = "*R R R000063 1300146 8574808             % gilt für Rück-Richtung 63 ab HS-Nr. 1300146 bis HS-Nr. 8574808";
            let (
                res, // res contains the comments that are useful to determine the direction
                (
                    direction,
                    ref_direction_code,
                    stop_from_id,
                    stop_to_id,
                    departure_time,
                    arrival_time,
                ),
            ) = row_r_parser(input).unwrap();

            assert_eq!("R", direction);
            assert_eq!("R000063", ref_direction_code);
            assert_eq!(Some(1300146), stop_from_id);
            assert_eq!(Some(8574808), stop_to_id);
            assert_eq!(None, departure_time);
            assert_eq!(None, arrival_time);
            assert_eq!(
                "% gilt für Rück-Richtung 63 ab HS-Nr. 1300146 bis HS-Nr. 8574808",
                res
            );
        }

        #[test]
        fn success_with_partial_options() {
            let input =
                "*R H                                     % gilt für die gesamte Hin-Richtung";
            let (
                res, // res contains the comments that are useful to determine the direction
                (
                    direction,
                    ref_direction_code,
                    stop_from_id,
                    stop_to_id,
                    departure_time,
                    arrival_time,
                ),
            ) = row_r_parser(input).unwrap();
            assert_eq!("H", direction);
            assert_eq!("", ref_direction_code);
            assert_eq!(None, stop_from_id);
            assert_eq!(None, stop_to_id);
            assert_eq!(None, departure_time);
            assert_eq!(None, arrival_time);
            assert_eq!("% gilt für die gesamte Hin-Richtung", res);
        }
    }

    mod row_ci_co {
        // Note this useful idiom: importing names from outer (for mod tests) scope.
        use super::*;
        use pretty_assertions::assert_eq;

        fn row_ci_co_parser(
            input: &str,
        ) -> IResult<
            &str,
            (
                &str,
                i32,
                Option<i32>,
                Option<i32>,
                Option<i32>,
                Option<i32>,
            ),
        > {
            let (
                res,
                (
                    ci_co,
                    _,
                    num_minutes,
                    _,
                    stop_from_id,
                    _,
                    stop_to_id,
                    _,
                    departure_time,
                    _,
                    arrival_time,
                ),
            ) = row_ci_co_combinator().parse(input)?;
            Ok((
                res,
                (
                    ci_co,
                    num_minutes,
                    stop_from_id,
                    stop_to_id,
                    departure_time,
                    arrival_time,
                ),
            ))
        }

        #[test]
        fn success_ci_options() {
            let input = "*CI 0002 8507000 8507000                                   % Check-in 2 Min. ab HS-Nr. 8507000 bis HS-Nr. 8507000";
            let (res, (ci_co, num_minutes, stop_from_id, stop_to_id, departure_time, arrival_time)) =
                row_ci_co_parser(input).unwrap();

            assert_eq!("*CI", ci_co);
            assert_eq!(2, num_minutes);
            assert_eq!(Some(8507000), stop_from_id);
            assert_eq!(Some(8507000), stop_to_id);
            assert_eq!(None, departure_time);
            assert_eq!(None, arrival_time);
            assert_eq!(
                "% Check-in 2 Min. ab HS-Nr. 8507000 bis HS-Nr. 8507000",
                res.trim()
            );
        }

        #[test]
        fn success_with_partial_options() {
            let input = "*CO 0002 8507000 8507000                                   % Check-out 2 Min. ab HS-Nr. 8507000 bis HS-Nr. 8507000";
            let (res, (ci_co, num_minutes, stop_from_id, stop_to_id, departure_time, arrival_time)) =
                row_ci_co_parser(input).unwrap();

            assert_eq!("*CO", ci_co);
            assert_eq!(2, num_minutes);
            assert_eq!(Some(8507000), stop_from_id);
            assert_eq!(Some(8507000), stop_to_id);
            assert_eq!(None, departure_time);
            assert_eq!(None, arrival_time);
            assert_eq!(
                "% Check-out 2 Min. ab HS-Nr. 8507000 bis HS-Nr. 8507000",
                res.trim()
            );
        }
    }

    mod row_journey_description {

        fn row_journey_description_parser(
            input: &str,
        ) -> IResult<&str, (i32, String, Option<i32>, Option<i32>, Option<i32>, String)> {
            let (
                res,
                (
                    stop_id,
                    _,
                    stop_name,
                    _,
                    arrival_time,
                    _,
                    departure_time,
                    _,
                    journey_id,
                    _,
                    administration,
                ),
            ) = row_journey_description_combinator().parse(input)?;
            Ok((
                res,
                (
                    stop_id,
                    stop_name,
                    arrival_time,
                    departure_time,
                    journey_id,
                    administration,
                ),
            ))
        }
        // Note this useful idiom: importing names from outer (for mod tests) scope.
        use super::*;
        use pretty_assertions::assert_eq;

        #[test]
        fn success_journey_options1() {
            let input = "0053301 S Wannsee DB                02014                % HS-Nr. 0053301 Ankunft N/A,   Abfahrt 20:14";
            // let input = "0053291 Wannseebrücke        02015 02015 052344 80____ % HS-Nr. 0053291 Ankunft 20:15, Abfahrt 20:15, Fahrtnummer 052344, Verwaltung 80____ (DB)";
            let (
                res,
                (stop_id, stop_name, arrival_time, departure_time, journey_id, administration),
            ) = row_journey_description_parser(input).unwrap();

            assert_eq!(53301, stop_id);
            assert_eq!("S Wannsee DB", stop_name);
            assert_eq!(None, arrival_time);
            assert_eq!(Some(2014), departure_time);
            assert_eq!(None, journey_id);
            assert_eq!("", administration);
            assert_eq!("% HS-Nr. 0053301 Ankunft N/A,   Abfahrt 20:14", res.trim());
        }

        #[test]
        fn success_journey_options2() {
            let input = "0053202 Am Kl. Wannsee/Am Gr  02016  02016               %";
            let (
                res,
                (stop_id, stop_name, arrival_time, departure_time, journey_id, administration),
            ) = row_journey_description_parser(input).unwrap();

            assert_eq!(53202, stop_id);
            assert_eq!("Am Kl. Wannsee/Am Gr", stop_name);
            assert_eq!(Some(2016), arrival_time);
            assert_eq!(Some(2016), departure_time);
            assert_eq!(None, journey_id);
            assert_eq!("", administration);
            assert_eq!("%", res.trim());
        }

        #[test]
        fn success_journey_all_options() {
            let input = "0053291 Wannseebrücke         02015  02015 052344 80____ % HS-Nr. 0053291 Ankunft 20:15, Abfahrt 20:15, Fahrtnummer 052344, Verwaltung 80____ (DB)";
            let (
                res,
                (stop_id, stop_name, arrival_time, departure_time, journey_id, administration),
            ) = row_journey_description_parser(input).unwrap();

            assert_eq!(53291, stop_id);
            assert_eq!("Wannseebrücke", stop_name);
            assert_eq!(Some(2015), arrival_time);
            assert_eq!(Some(2015), departure_time);
            assert_eq!(Some(52344), journey_id);
            assert_eq!("80____", administration);
            assert_eq!(
                "% HS-Nr. 0053291 Ankunft 20:15, Abfahrt 20:15, Fahrtnummer 052344, Verwaltung 80____ (DB)",
                res.trim()
            );
        }
        // 0000175 Hauenstein-Basistunn -00833 -00833                 %
        #[test]
        fn success_journey_negative_time() {
            // let input = "0053291 Wannseebrücke        02015 02015 052344 80____ % HS-Nr. 0053291 Ankunft 20:15, Abfahrt 20:15, Fahrtnummer 052344, Verwaltung 80____ (DB)";
            let input = "0000175 Hauenstein-Basistunn -00833 -00833                 %";
            let (
                res,
                (stop_id, stop_name, arrival_time, departure_time, journey_id, administration),
            ) = row_journey_description_parser(input).unwrap();

            assert_eq!(175, stop_id);
            assert_eq!("Hauenstein-Basistunn", stop_name);
            assert_eq!(Some(-833), arrival_time);
            assert_eq!(Some(-833), departure_time);
            assert_eq!(None, journey_id);
            assert_eq!("", administration);
            assert_eq!("%", res.trim());
        }
    }
}
