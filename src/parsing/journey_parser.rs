/// # Journey parser
///
/// List of journeys and by far the largest and most comprehensive file in the HRDF export.
///
/// This file contains:
///
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
/// `
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
///
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
/// *I hi 8573602 8587744       000018040             % Hinweis auf Infotext (hi) ab HS-Nr. 8573602 bis HS-Nr. 8587744  mit Infotext 18040
/// *I hi 8578157 8589334       000018037 01126 01159 % Hinweis auf Infotext (hi) ab HS-Nr. 8578157 bis HS-Nr. 8589334 mit Infotext 18037 Abfahrt 11:26 Ankunft 11:59
/// ...
/// `
///
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
///
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
/// *R H                         % gilt für die gesamte Hin-Richtung
/// *R R R000063 1300146 8574808 % gilt für Rück-Richtung 63 ab HS-Nr. 1300146 bis HS-Nr. 8574808
/// ...
/// `
///
/// ## GR/SH-lines
///
/// - *GR lines: supported but not available in Switzerland.
/// - *SH lines: supported but not available in Switzerland.
///
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
/// *CI 0002 8507000 8507000 % Check-in 2 Min. ab HS-Nr. 8507000 bis HS-Nr. 8507000
/// ...
/// *CO 0002 8507000 8507000 % Check-out 2 Min. ab HS-Nr. 8507000 bis HS-Nr. 8507000
/// ...
/// `
///
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
/// 0053301 S Wannsee DB               02014               % HS-Nr. 0053301 Ankunft N/A,   Abfahrt 20:14
/// 0053291 Wannseebrücke        02015 02015 052344 80____ % HS-Nr. 0053291 Ankunft 20:15, Abfahrt 20:15, Fahrtnummer 052344, Verwaltung 80____ (DB)
/// 0053202 Am Kl. Wannsee/Am Gr 02016 02016               %
/// `
///
/// 1 file(s).
/// File(s) read by the parser:
/// FPLAN
use chrono::NaiveTime;
use rustc_hash::{FxHashMap, FxHashSet};

use crate::{
    JourneyId, Result,
    error::ErrorKind,
    models::{Journey, JourneyMetadataEntry, JourneyMetadataType, JourneyRouteEntry, Model},
    parsing::{
        ColumnDefinition, ExpectedType, FastRowMatcher, FileParser, ParsedValue, RowDefinition,
        RowParser,
    },
    storage::ResourceStorage,
    utils::{AutoIncrement, create_time_from_value},
};

type JourneyAndTypeConverter = (ResourceStorage<Journey>, FxHashSet<JourneyId>);

enum RowType {
    RowA = 1,
    RowB = 2,
    RowC = 3,
    RowD = 4,
    RowE = 5,
    RowF = 6,
    RowG = 7,
    RowH = 8,
    RowI = 9,
}

fn journey_row_parser() -> RowParser {
    RowParser::new(vec![
        // This row is used to create a Journey instance.
        RowDefinition::new(
            RowType::RowA as i32,
            Box::new(FastRowMatcher::new(1, 2, "*Z", true)),
            vec![
                ColumnDefinition::new(4, 9, ExpectedType::Integer32),
                ColumnDefinition::new(11, 16, ExpectedType::String),
            ],
        ),
        RowDefinition::new(
            RowType::RowB as i32,
            Box::new(FastRowMatcher::new(1, 2, "*G", true)),
            vec![
                ColumnDefinition::new(4, 6, ExpectedType::String),
                ColumnDefinition::new(8, 14, ExpectedType::OptionInteger32),
                ColumnDefinition::new(16, 22, ExpectedType::OptionInteger32),
            ],
        ),
        RowDefinition::new(
            RowType::RowC as i32,
            Box::new(FastRowMatcher::new(1, 5, "*A VE", true)),
            vec![
                ColumnDefinition::new(7, 13, ExpectedType::OptionInteger32),
                ColumnDefinition::new(15, 21, ExpectedType::OptionInteger32),
                ColumnDefinition::new(23, 28, ExpectedType::OptionInteger32),
            ],
        ),
        RowDefinition::new(
            RowType::RowD as i32,
            Box::new(FastRowMatcher::new(1, 2, "*A", true)),
            vec![
                ColumnDefinition::new(4, 5, ExpectedType::String),
                ColumnDefinition::new(7, 13, ExpectedType::OptionInteger32),
                ColumnDefinition::new(15, 21, ExpectedType::OptionInteger32),
            ],
        ),
        RowDefinition::new(
            RowType::RowE as i32,
            Box::new(FastRowMatcher::new(1, 2, "*I", true)),
            vec![
                ColumnDefinition::new(4, 5, ExpectedType::String),
                ColumnDefinition::new(7, 13, ExpectedType::OptionInteger32),
                ColumnDefinition::new(15, 21, ExpectedType::OptionInteger32),
                ColumnDefinition::new(23, 28, ExpectedType::OptionInteger32),
                ColumnDefinition::new(30, 38, ExpectedType::Integer32),
                ColumnDefinition::new(40, 45, ExpectedType::OptionInteger32),
                ColumnDefinition::new(47, 52, ExpectedType::OptionInteger32),
            ],
        ),
        RowDefinition::new(
            RowType::RowF as i32,
            Box::new(FastRowMatcher::new(1, 2, "*L", true)),
            vec![
                ColumnDefinition::new(4, 11, ExpectedType::String),
                ColumnDefinition::new(13, 19, ExpectedType::OptionInteger32),
                ColumnDefinition::new(21, 27, ExpectedType::OptionInteger32),
                ColumnDefinition::new(29, 34, ExpectedType::OptionInteger32),
                ColumnDefinition::new(36, 41, ExpectedType::OptionInteger32),
            ],
        ),
        RowDefinition::new(
            RowType::RowG as i32,
            Box::new(FastRowMatcher::new(1, 2, "*R", true)),
            vec![
                ColumnDefinition::new(4, 4, ExpectedType::String),
                ColumnDefinition::new(6, 12, ExpectedType::String),
                ColumnDefinition::new(14, 20, ExpectedType::OptionInteger32),
                ColumnDefinition::new(22, 28, ExpectedType::OptionInteger32),
                ColumnDefinition::new(30, 35, ExpectedType::OptionInteger32),
                ColumnDefinition::new(37, 42, ExpectedType::OptionInteger32),
            ],
        ),
        // *CI
        RowDefinition::new(
            RowType::RowH as i32,
            Box::new(FastRowMatcher::new(1, 3, "*CI", true)),
            vec![
                ColumnDefinition::new(1, 3, ExpectedType::String),
                ColumnDefinition::new(5, 8, ExpectedType::Integer32),
                ColumnDefinition::new(10, 16, ExpectedType::OptionInteger32),
                ColumnDefinition::new(18, 24, ExpectedType::OptionInteger32),
            ],
        ),
        // *CO
        RowDefinition::new(
            RowType::RowH as i32,
            Box::new(FastRowMatcher::new(1, 3, "*CO", true)),
            vec![
                ColumnDefinition::new(1, 3, ExpectedType::String),
                ColumnDefinition::new(5, 8, ExpectedType::Integer32),
                ColumnDefinition::new(10, 16, ExpectedType::OptionInteger32),
                ColumnDefinition::new(18, 24, ExpectedType::OptionInteger32),
            ],
        ),
        RowDefinition::new(
            RowType::RowI as i32,
            Box::new(FastRowMatcher::new(1, 0, "", true)),
            vec![
                ColumnDefinition::new(1, 7, ExpectedType::Integer32),
                ColumnDefinition::new(30, 35, ExpectedType::OptionInteger32),
                ColumnDefinition::new(37, 42, ExpectedType::OptionInteger32),
            ],
        ),
    ])
}

fn journey_row_converter(
    parser: FileParser,
    transport_types_pk_type_converter: &FxHashMap<String, i32>,
    attributes_pk_type_converter: &FxHashMap<String, i32>,
    directions_pk_type_converter: &FxHashMap<String, i32>,
) -> Result<(FxHashMap<i32, Journey>, FxHashSet<JourneyId>)> {
    let auto_increment = AutoIncrement::new();
    let mut data = Vec::new();
    let mut pk_type_converter = FxHashSet::default();

    for x in parser.parse() {
        let (id, _, values) = x?;
        if RowType::RowA as i32 == id {
            data.push(create_instance(
                values,
                &auto_increment,
                &mut pk_type_converter,
            ));
        } else {
            let journey = data.last_mut().ok_or(ErrorKind::RowMissing { typ: "A" })?;

            if id == RowType::RowB as i32 {
                set_transport_type(values, journey, transport_types_pk_type_converter)?;
            } else if id == RowType::RowC as i32 {
                set_bit_field(values, journey);
            } else if id == RowType::RowD as i32 {
                add_attribute(values, journey, attributes_pk_type_converter)?;
            } else if id == RowType::RowE as i32 {
                add_information_text(values, journey);
            } else if id == RowType::RowF as i32 {
                set_line(values, journey)?;
            } else if id == RowType::RowG as i32 {
                set_direction(values, journey, directions_pk_type_converter)?;
            } else if id == RowType::RowH as i32 {
                set_boarding_or_disembarking_exchange_time(values, journey);
            } else if id == RowType::RowI as i32 {
                add_route_entry(values, journey);
            } else {
                unreachable!();
            }
        }
    }

    let data = Journey::vec_to_map(data);

    Ok((data, pk_type_converter))
}

pub fn parse(
    path: &str,
    transport_types_pk_type_converter: &FxHashMap<String, i32>,
    attributes_pk_type_converter: &FxHashMap<String, i32>,
    directions_pk_type_converter: &FxHashMap<String, i32>,
) -> Result<JourneyAndTypeConverter> {
    log::info!("Parsing FPLAN...");
    let row_parser = journey_row_parser();
    let parser = FileParser::new(&format!("{path}/FPLAN"), row_parser)?;

    let (data, pk_type_converter) = journey_row_converter(
        parser,
        transport_types_pk_type_converter,
        attributes_pk_type_converter,
        directions_pk_type_converter,
    )?;
    Ok((ResourceStorage::new(data), pk_type_converter))
}

// ------------------------------------------------------------------------------------------------
// --- Data Processing Functions
// ------------------------------------------------------------------------------------------------

// RowA parsing

fn row_a_from_parsed_values(mut values: Vec<ParsedValue>) -> (i32, String) {
    let legacy_id: i32 = values.remove(0).into();
    let administration: String = values.remove(0).into();
    (legacy_id, administration)
}

fn create_instance(
    values: Vec<ParsedValue>,
    auto_increment: &AutoIncrement,
    pk_type_converter: &mut FxHashSet<JourneyId>,
) -> Journey {
    let (legacy_id, administration) = row_a_from_parsed_values(values);

    let id = auto_increment.next();

    pk_type_converter.insert((legacy_id, administration.to_owned()));
    Journey::new(id, legacy_id, administration)
}

// RowB parsing

fn row_b_from_parsed_values(mut values: Vec<ParsedValue>) -> (String, Option<i32>, Option<i32>) {
    let designation: String = values.remove(0).into();
    let from_stop_id: Option<i32> = values.remove(0).into();
    let until_stop_id: Option<i32> = values.remove(0).into();
    (designation, from_stop_id, until_stop_id)
}

fn set_transport_type(
    values: Vec<ParsedValue>,
    journey: &mut Journey,
    transport_types_pk_type_converter: &FxHashMap<String, i32>,
) -> Result<()> {
    let (designation, from_stop_id, until_stop_id) = row_b_from_parsed_values(values);
    let transport_type_id = *transport_types_pk_type_converter
        .get(&designation)
        .ok_or(ErrorKind::UnknownLegacyId)?;

    journey.add_metadata_entry(
        JourneyMetadataType::TransportType,
        JourneyMetadataEntry::new(
            from_stop_id,
            until_stop_id,
            Some(transport_type_id),
            None,
            None,
            None,
            None,
            None,
        ),
    );

    Ok(())
}

// RowC parsing

fn row_c_from_parsed_values(
    mut values: Vec<ParsedValue>,
) -> (Option<i32>, Option<i32>, Option<i32>) {
    let from_stop_id: Option<i32> = values.remove(0).into();
    let until_stop_id: Option<i32> = values.remove(0).into();
    let bit_field_id: Option<i32> = values.remove(0).into();
    (from_stop_id, until_stop_id, bit_field_id)
}

fn set_bit_field(values: Vec<ParsedValue>, journey: &mut Journey) {
    let (from_stop_id, until_stop_id, bit_field_id) = row_c_from_parsed_values(values);
    journey.add_metadata_entry(
        JourneyMetadataType::BitField,
        JourneyMetadataEntry::new(
            from_stop_id,
            until_stop_id,
            None,
            bit_field_id,
            None,
            None,
            None,
            None,
        ),
    );
}

// Parsing RowD

fn row_d_from_parsed_values(mut values: Vec<ParsedValue>) -> (String, Option<i32>, Option<i32>) {
    let designation: String = values.remove(0).into();
    let from_stop_id: Option<i32> = values.remove(0).into();
    let until_stop_id: Option<i32> = values.remove(0).into();
    (designation, from_stop_id, until_stop_id)
}

fn add_attribute(
    values: Vec<ParsedValue>,
    journey: &mut Journey,
    attributes_pk_type_converter: &FxHashMap<String, i32>,
) -> Result<()> {
    let (designation, from_stop_id, until_stop_id) = row_d_from_parsed_values(values);

    let attribute_id = *attributes_pk_type_converter
        .get(&designation)
        .ok_or(ErrorKind::UnknownLegacyId)?;

    journey.add_metadata_entry(
        JourneyMetadataType::Attribute,
        JourneyMetadataEntry::new(
            from_stop_id,
            until_stop_id,
            Some(attribute_id),
            None,
            None,
            None,
            None,
            None,
        ),
    );

    Ok(())
}

// Parsing RowE

fn row_e_from_parsed_values(
    mut values: Vec<ParsedValue>,
) -> (
    String,
    Option<i32>,
    Option<i32>,
    Option<i32>,
    i32,
    Option<i32>,
    Option<i32>,
) {
    let code: String = values.remove(0).into();
    let from_stop_id: Option<i32> = values.remove(0).into();
    let until_stop_id: Option<i32> = values.remove(0).into();
    let bit_field_id: Option<i32> = values.remove(0).into();
    let information_text_id: i32 = values.remove(0).into();
    let departure_time: Option<i32> = values.remove(0).into();
    let arrival_time: Option<i32> = values.remove(0).into();
    (
        code,
        from_stop_id,
        until_stop_id,
        bit_field_id,
        information_text_id,
        departure_time,
        arrival_time,
    )
}

fn add_information_text(values: Vec<ParsedValue>, journey: &mut Journey) {
    let (
        code,
        from_stop_id,
        until_stop_id,
        bit_field_id,
        information_text_id,
        departure_time,
        arrival_time,
    ) = row_e_from_parsed_values(values);
    let arrival_time = create_time(arrival_time);
    let departure_time = create_time(departure_time);

    journey.add_metadata_entry(
        JourneyMetadataType::InformationText,
        JourneyMetadataEntry::new(
            from_stop_id,
            until_stop_id,
            Some(information_text_id),
            bit_field_id,
            departure_time,
            arrival_time,
            Some(code),
            None,
        ),
    );
}

// Parsing RowF

fn row_f_from_parsed_values(
    mut values: Vec<ParsedValue>,
) -> (String, Option<i32>, Option<i32>, Option<i32>, Option<i32>) {
    let line_designation: String = values.remove(0).into();
    let from_stop_id: Option<i32> = values.remove(0).into();
    let until_stop_id: Option<i32> = values.remove(0).into();
    let departure_time: Option<i32> = values.remove(0).into();
    let arrival_time: Option<i32> = values.remove(0).into();

    (
        line_designation,
        from_stop_id,
        until_stop_id,
        departure_time,
        arrival_time,
    )
}

fn set_line(values: Vec<ParsedValue>, journey: &mut Journey) -> Result<()> {
    let (line_designation, from_stop_id, until_stop_id, departure_time, arrival_time) =
        row_f_from_parsed_values(values);
    let arrival_time = create_time(arrival_time);
    let departure_time = create_time(departure_time);

    let line_designation_first_char = line_designation
        .chars()
        .next()
        .ok_or(ErrorKind::MissingDesignation)?;
    let (resource_id, extra_field_1) = if line_designation_first_char == '#' {
        (Some(line_designation[1..].parse::<i32>()?), None)
    } else {
        (None, Some(line_designation))
    };

    journey.add_metadata_entry(
        JourneyMetadataType::Line,
        JourneyMetadataEntry::new(
            from_stop_id,
            until_stop_id,
            resource_id,
            None,
            departure_time,
            arrival_time,
            extra_field_1,
            None,
        ),
    );

    Ok(())
}

// Parsing RowG

fn row_g_from_parsed_values(
    mut values: Vec<ParsedValue>,
) -> (
    String,
    String,
    Option<i32>,
    Option<i32>,
    Option<i32>,
    Option<i32>,
) {
    let direction_type: String = values.remove(0).into();
    let direction_id: String = values.remove(0).into();
    let from_stop_id: Option<i32> = values.remove(0).into();
    let until_stop_id: Option<i32> = values.remove(0).into();
    let departure_time: Option<i32> = values.remove(0).into();
    let arrival_time: Option<i32> = values.remove(0).into();

    (
        direction_type,
        direction_id,
        from_stop_id,
        until_stop_id,
        departure_time,
        arrival_time,
    )
}

fn set_direction(
    values: Vec<ParsedValue>,
    journey: &mut Journey,
    directions_pk_type_converter: &FxHashMap<String, i32>,
) -> Result<()> {
    let (direction_type, direction_id, from_stop_id, until_stop_id, departure_time, arrival_time) =
        row_g_from_parsed_values(values);
    let arrival_time = create_time(arrival_time);
    let departure_time = create_time(departure_time);

    let direction_id = if direction_id.is_empty() {
        None
    } else {
        let id = *directions_pk_type_converter
            .get(&direction_id)
            .ok_or(ErrorKind::UnknownLegacyId)?;
        Some(id)
    };

    journey.add_metadata_entry(
        JourneyMetadataType::Direction,
        JourneyMetadataEntry::new(
            from_stop_id,
            until_stop_id,
            direction_id,
            None,
            departure_time,
            arrival_time,
            Some(direction_type),
            None,
        ),
    );

    Ok(())
}

// Parsing RowH

fn row_h_from_parsed_values(
    mut values: Vec<ParsedValue>,
) -> (String, i32, Option<i32>, Option<i32>) {
    let ci_co: String = values.remove(0).into();
    let exchange_time: i32 = values.remove(0).into();
    let from_stop_id: Option<i32> = values.remove(0).into();
    let until_stop_id: Option<i32> = values.remove(0).into();

    (ci_co, exchange_time, from_stop_id, until_stop_id)
}

fn set_boarding_or_disembarking_exchange_time(values: Vec<ParsedValue>, journey: &mut Journey) {
    let (ci_co, exchange_time, from_stop_id, until_stop_id) = row_h_from_parsed_values(values);

    let metadata_type = if ci_co == "*CI" {
        JourneyMetadataType::ExchangeTimeBoarding
    } else {
        JourneyMetadataType::ExchangeTimeDisembarking
    };

    journey.add_metadata_entry(
        metadata_type,
        JourneyMetadataEntry::new(
            from_stop_id,
            until_stop_id,
            None,
            None,
            None,
            None,
            None,
            Some(exchange_time),
        ),
    );
}

// Parsing RowI

fn row_i_from_parsed_values(mut values: Vec<ParsedValue>) -> (i32, Option<i32>, Option<i32>) {
    let stop_id: i32 = values.remove(0).into();
    let arrival_time: Option<i32> = values.remove(0).into();
    let departure_time: Option<i32> = values.remove(0).into();
    (stop_id, arrival_time, departure_time)
}

fn add_route_entry(values: Vec<ParsedValue>, journey: &mut Journey) {
    let (stop_id, arrival_time, departure_time) = row_i_from_parsed_values(values);
    let arrival_time = create_time(arrival_time);
    let departure_time = create_time(departure_time);

    journey.add_route_entry(JourneyRouteEntry::new(
        stop_id,
        arrival_time,
        departure_time,
    ));
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
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;
    //use crate::parsing::tests::get_json_values;
    use pretty_assertions::assert_eq;

    #[test]
    fn parsing_rows_v207() {
        let rows = vec![
            "*Z 000003 000011   101                                     % -- 37649518927 --"
                .to_string(),
            "*G ICE 8500090 8503000                                     %".to_string(),
            "*A VE 8500090 8503000 281004                               %".to_string(),
            "*A VR 8500090 8503000                                      %".to_string(),
            "*A WR 8500090 8503000                                      %".to_string(),
            "*I JY                        000000000                     %".to_string(),
            "*R H                                                       %".to_string(),
            "8500090 Basel Bad Bf                 00740                 %".to_string(),
            "8500010 Basel SBB             00748  00806                 %".to_string(),
            "0000175 Hauenstein-Basistunn -00833 -00833                 %".to_string(),
            "8503000 Zürich HB             00900                        %".to_string(),
        ];
        let parser = FileParser {
            row_parser: journey_row_parser(),
            rows: rows.clone(),
        };
        let mut parser_iterator = parser.parse();

        {
            let (id, _, parsed_values) = parser_iterator.next().unwrap().unwrap();
            assert_eq!(id, RowType::RowA as i32);
            let (legacy_id, administration) = row_a_from_parsed_values(parsed_values);
            assert_eq!(3, legacy_id);
            assert_eq!("000011", &administration);
        }
        {
            let (id, _, parsed_values) = parser_iterator.next().unwrap().unwrap();
            assert_eq!(id, RowType::RowB as i32);
            let (designation, from_stop_id, until_stop_id) =
                row_b_from_parsed_values(parsed_values);
            assert_eq!("ICE", &designation);
            assert_eq!(Some(8500090), from_stop_id);
            assert_eq!(Some(8503000), until_stop_id);
        }
        {
            let (id, _, parsed_values) = parser_iterator.next().unwrap().unwrap();
            assert_eq!(id, RowType::RowC as i32);
            let (from_stop_id, until_stop_id, bit_field_id) =
                row_c_from_parsed_values(parsed_values);
            assert_eq!(Some(8500090), from_stop_id);
            assert_eq!(Some(8503000), until_stop_id);
            assert_eq!(Some(281004), bit_field_id);
        }
        {
            let (id, _, parsed_values) = parser_iterator.next().unwrap().unwrap();
            assert_eq!(id, RowType::RowD as i32);
            let (designation, from_stop_id, until_stop_id) =
                row_d_from_parsed_values(parsed_values);
            assert_eq!("VR", &designation);
            assert_eq!(Some(8500090), from_stop_id);
            assert_eq!(Some(8503000), until_stop_id);
        }
        {
            let (id, _, parsed_values) = parser_iterator.next().unwrap().unwrap();
            assert_eq!(id, RowType::RowD as i32);
            let (designation, from_stop_id, until_stop_id) =
                row_d_from_parsed_values(parsed_values);
            assert_eq!("WR", &designation);
            assert_eq!(Some(8500090), from_stop_id);
            assert_eq!(Some(8503000), until_stop_id);
        }
        {
            let (id, _, parsed_values) = parser_iterator.next().unwrap().unwrap();
            assert_eq!(id, RowType::RowE as i32);
            let (
                code,
                from_stop_id,
                until_stop_id,
                bit_field_id,
                information_text_id,
                departure_time,
                arrival_time,
            ) = row_e_from_parsed_values(parsed_values);
            assert_eq!("JY", &code);
            assert_eq!(None, from_stop_id);
            assert_eq!(None, until_stop_id);
            assert_eq!(None, bit_field_id);
            assert_eq!(0, information_text_id);
            assert_eq!(None, departure_time);
            assert_eq!(None, arrival_time);
        }
        {
            let (id, _, parsed_values) = parser_iterator.next().unwrap().unwrap();
            assert_eq!(id, RowType::RowG as i32);
            let (
                direction_type,
                direction_id,
                from_stop_id,
                until_stop_id,
                departure_time,
                arrival_time,
            ) = row_g_from_parsed_values(parsed_values);
            // "*R H                                                       %".to_string(),
            assert_eq!("H", &direction_type);
            assert_eq!("", &direction_id);
            assert_eq!(None, from_stop_id);
            assert_eq!(None, until_stop_id);
            assert_eq!(None, departure_time);
            assert_eq!(None, arrival_time);
        }
        {
            let (id, _, parsed_values) = parser_iterator.next().unwrap().unwrap();
            assert_eq!(id, RowType::RowI as i32);
            let (stop_id, arrival_time, departure_time) = row_i_from_parsed_values(parsed_values);
            assert_eq!(8500090, stop_id);
            assert_eq!(None, arrival_time);
            assert_eq!(Some(740), departure_time);
        }
        {
            let (id, _, parsed_values) = parser_iterator.next().unwrap().unwrap();
            assert_eq!(id, RowType::RowI as i32);
            let (stop_id, arrival_time, departure_time) = row_i_from_parsed_values(parsed_values);
            assert_eq!(8500010, stop_id);
            assert_eq!(Some(748), arrival_time);
            assert_eq!(Some(806), departure_time);
        }
        {
            let (id, _, parsed_values) = parser_iterator.next().unwrap().unwrap();
            assert_eq!(id, RowType::RowI as i32);
            let (stop_id, arrival_time, departure_time) = row_i_from_parsed_values(parsed_values);
            assert_eq!(175, stop_id);
            assert_eq!(Some(-833), arrival_time);
            assert_eq!(Some(-833), departure_time);
        }
        {
            let (id, _, parsed_values) = parser_iterator.next().unwrap().unwrap();
            assert_eq!(id, RowType::RowI as i32);
            let (stop_id, arrival_time, departure_time) = row_i_from_parsed_values(parsed_values);
            assert_eq!(8503000, stop_id);
            assert_eq!(Some(900), arrival_time);
            assert_eq!(None, departure_time);
        }
    }

    #[test]
    fn parsing_rows_alt_v207() {
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
        let parser = FileParser {
            row_parser: journey_row_parser(),
            rows: rows.clone(),
        };
        let mut parser_iterator = parser.parse();

        {
            let (id, _, parsed_values) = parser_iterator.next().unwrap().unwrap();
            assert_eq!(id, RowType::RowA as i32);
            let (legacy_id, administration) = row_a_from_parsed_values(parsed_values);
            assert_eq!(2359, legacy_id);
            assert_eq!("000011", &administration);
        }
        {
            let (id, _, parsed_values) = parser_iterator.next().unwrap().unwrap();
            assert_eq!(id, RowType::RowB as i32);
            let (designation, from_stop_id, until_stop_id) =
                row_b_from_parsed_values(parsed_values);
            assert_eq!("IR", &designation);
            assert_eq!(Some(8507000), from_stop_id);
            assert_eq!(Some(8509000), until_stop_id);
        }
        {
            let (id, _, parsed_values) = parser_iterator.next().unwrap().unwrap();
            assert_eq!(id, RowType::RowC as i32);
            let (from_stop_id, until_stop_id, bit_field_id) =
                row_c_from_parsed_values(parsed_values);
            assert_eq!(Some(8507000), from_stop_id);
            assert_eq!(Some(8509000), until_stop_id);
            assert_eq!(Some(348508), bit_field_id);
        }
        {
            let (id, _, parsed_values) = parser_iterator.next().unwrap().unwrap();
            assert_eq!(id, RowType::RowD as i32);
            let (designation, from_stop_id, until_stop_id) =
                row_d_from_parsed_values(parsed_values);
            assert_eq!("FS", &designation);
            assert_eq!(Some(8507000), from_stop_id);
            assert_eq!(Some(8509000), until_stop_id);
        }
        {
            let (id, _, parsed_values) = parser_iterator.next().unwrap().unwrap();
            assert_eq!(id, RowType::RowE as i32);
            let (
                code,
                from_stop_id,
                until_stop_id,
                bit_field_id,
                information_text_id,
                departure_time,
                arrival_time,
            ) = row_e_from_parsed_values(parsed_values);
            assert_eq!("JY", &code);
            assert_eq!(None, from_stop_id);
            assert_eq!(None, until_stop_id);
            assert_eq!(None, bit_field_id);
            assert_eq!(1370, information_text_id);
            assert_eq!(None, departure_time);
            assert_eq!(None, arrival_time);
        }
        {
            let (id, _, parsed_values) = parser_iterator.next().unwrap().unwrap();
            assert_eq!(id, RowType::RowF as i32);
            let (line_designation, from_stop_id, until_stop_id, departure_time, arrival_time) =
                row_f_from_parsed_values(parsed_values);
            assert_eq!("35", &line_designation);
            assert_eq!(Some(8507000), from_stop_id);
            assert_eq!(Some(8509000), until_stop_id);
            assert_eq!(None, departure_time);
            assert_eq!(None, arrival_time);
        }
        {
            let (id, _, parsed_values) = parser_iterator.next().unwrap().unwrap();
            assert_eq!(id, RowType::RowG as i32);
            let (
                direction_type,
                direction_id,
                from_stop_id,
                until_stop_id,
                departure_time,
                arrival_time,
            ) = row_g_from_parsed_values(parsed_values);
            assert_eq!("H", &direction_type);
            assert_eq!("", &direction_id);
            assert_eq!(None, from_stop_id);
            assert_eq!(None, until_stop_id);
            assert_eq!(None, departure_time);
            assert_eq!(None, arrival_time);
        }
        {
            let (id, _, parsed_values) = parser_iterator.next().unwrap().unwrap();
            assert_eq!(id, RowType::RowH as i32);
            let (ci_co, exchange_time, from_stop_id, until_stop_id) =
                row_h_from_parsed_values(parsed_values);

            assert_eq!("*CI", &ci_co);
            assert_eq!(2, exchange_time);
            assert_eq!(Some(8507000), from_stop_id);
            assert_eq!(Some(8507000), until_stop_id);
        }
        // "8509000 Chur                  00948                        %".to_string(),
        {
            let (id, _, parsed_values) = parser_iterator.next().unwrap().unwrap();
            assert_eq!(id, RowType::RowI as i32);
            let (stop_id, arrival_time, departure_time) = row_i_from_parsed_values(parsed_values);
            assert_eq!(8507000, stop_id);
            assert_eq!(None, arrival_time);
            assert_eq!(Some(638), departure_time);
        }
        {
            let (id, _, parsed_values) = parser_iterator.next().unwrap().unwrap();
            assert_eq!(id, RowType::RowI as i32);
            let (stop_id, arrival_time, departure_time) = row_i_from_parsed_values(parsed_values);
            assert_eq!(8508005, stop_id);
            assert_eq!(Some(652), arrival_time);
            assert_eq!(Some(653), departure_time);
        }
        {
            let (id, _, parsed_values) = parser_iterator.next().unwrap().unwrap();
            assert_eq!(id, RowType::RowI as i32);
            let (stop_id, arrival_time, departure_time) = row_i_from_parsed_values(parsed_values);
            assert_eq!(8508008, stop_id);
            assert_eq!(Some(704), arrival_time);
            assert_eq!(Some(705), departure_time);
        }
        {
            let (id, _, parsed_values) = parser_iterator.next().unwrap().unwrap();
            assert_eq!(id, RowType::RowI as i32);
            let (stop_id, arrival_time, departure_time) = row_i_from_parsed_values(parsed_values);
            assert_eq!(8509000, stop_id);
            assert_eq!(Some(948), arrival_time);
            assert_eq!(None, departure_time);
        }
    }

    // #[test]
    // fn type_converter_row_a_v207() {
    //     let rows = vec![
    //         "GK 0   4  5".to_string(),
    //         "<deu>".to_string(),
    //         "GK  Zollkontrolle möglich, mehr Zeit einrechnen".to_string(),
    //         "<fra>".to_string(),
    //         "GK  Contrôle douanier possible, prévoir davantage de temps".to_string(),
    //         "<ita>".to_string(),
    //         "GK  Possibile controllo doganale, prevedere più tempo".to_string(),
    //         "<eng>".to_string(),
    //         "GK  Possible customs check, please allow extra time".to_string(),
    //     ];
    //     let parser = FileParser {
    //         row_parser: attribute_row_parser(Version::V_5_40_41_2_0_7).unwrap(),
    //         rows,
    //     };
    //     let (data, pk_type_converter) = attribute_row_converter(parser).unwrap();
    //     assert_eq!(*pk_type_converter.get("GK").unwrap(), 1);
    //     let attribute = data.get(&1).unwrap();
    //     let reference = r#"
    //         {
    //             "id":1,
    //             "designation":"GK",
    //             "stop_scope":0,
    //             "main_sorting_priority":4,
    //             "secondary_sorting_priority":5,
    //             "description":{
    //                 "German":"Zollkontrolle möglich, mehr Zeit einrechnen",
    //                 "English":"Possible customs check, please allow extra time",
    //                 "French":"Contrôle douanier possible, prévoir davantage de temps",
    //                 "Italian":"Possibile controllo doganale, prevedere più tempo"
    //             }
    //         }"#;
    //     let (attribute, reference) = get_json_values(attribute, reference).unwrap();
    //     assert_eq!(attribute, reference);
    // }
}
