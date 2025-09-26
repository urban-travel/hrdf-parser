/// # BAHNHOF file
///
/// ## List of stops A detailed description of the stops (incl. Meta-stops (see METABHF file)) can be found here.
///
/// The file contains stops that are referenced in various files:
///
/// - Stop number, from DiDok (in future atlas), with a 7-digit number >= 1000000
/// - The first two numbers are the UIC country code
/// - Stop name with up to 4 types of designations:
///     - Up to “$<1>”: official designation from DiDok/atlas
///     - Up to “$<2>”: long designation from DiDok/atlas
///     - Up to “$<3>”: Abbreviation from DiDok/atlas
///     - Up to “$<4>”: alternative designations from the timetable collection
///
///
///
///
/// ## Example (excerpt):
///
/// `
/// ...
/// 8500009     Pregassona, Scuola Media$<1>
/// 8500010     Basel SBB$<1>$BS$<3>$Bale$<4>$Basilea FFS$<4>$Bâle CFF$<4>
/// 8500016     Basel St. Johann$<1>$BSSJ$<3>
/// ...
/// 8501212     Chavannes-R., UNIL-Mouline$<1>$Chavannes-près-Renens, UNIL-Mouline$<2>$MOUI$<3>
/// ...
/// `
///
/// Auxiliary stops have an ID < 1000000.They serve as a meta operating point and as an alternative to the name
/// of the DiDok/atlas system. They allow you to search for services with these names in an online timetable
/// without knowing the exact name of the stop according to DiDok/atlas.
///
/// ## Example – Search for Basel instead of “Basel SBB” (excerpt):
///
/// `
/// ...
/// 0000021     Barcelona$<1>    % Hilfs-Hs-Nr. 000021, off. Bez. Barcelona
/// 0000022     Basel$<1>        % Hilfs-Hs-Nr. 000022, off. Bez. Basel
/// 0000024     Bern Bümpliz$<1> % Hilfs-Hs-Nr. 000024, off. Bez. Bern Bümpliz
/// ...
/// `
///
/// # BFKOORD_* files
///
/// List of stops with their geo-coordinates. File contains:
///
/// - Stop number
/// - Longitude
/// - Latitude
/// - Height
///
/// ## Example (excerpt):
///
/// `
/// ...lv95-Datei:
/// 8500009    2718660    1098199   0      % HS-Nr. 8500009 LV-Läng. 2718660 LV-Breit. 1098199 Höhe 0 //Pregassona, Scuola Media
/// 8500010    2611363    1266310   0      % HS-Nr. 8500010 LV-Läng. 2611363 LV-Breit. 1266310 Höhe 0 //Basel SBB
/// 8500016    2610076    1268853   0      % HS-Nr. 8500016 LV-Läng. 2610076 LV-Breit. 1268853 Höhe 0 //Basel St. Johann
/// ...wgs84-Datei:
/// 8500009    8.971045   46.024911 0      % HS-Nr. 8500009 LV-Läng. 8.971045 LV-Breit. 46.024911 Höhe 0 //Pregassona, Scuola Media
/// 8500010    7.589563   47.547412 0      % HS-Nr. 8500010 LV-Läng. 7.589563 LV-Breit. 47.547412 Höhe 0 //Basel SBB
/// 8500016    7.572529   47.570306 0      % HS-Nr. 8500016 LV-Läng. 7.572529 LV-Breit. 47.570306 Höhe 0 //Basel St. Johann
/// ...
/// `
///
/// # BFPRIOS file
///
/// Definition of the priority of the stops The transfer priority allows you to select the transfer point if there are several transfer options. It is shown with a value between 0 and 16, where 0 is the highest priority and 16 is the lowest priority. File contains:
///
/// - HS no.
/// - Priority
/// - HS name
///
/// ## Example (excerpt):
///
/// If it is possible to change trains in Pregassona, Basel SBB or Basel St. Johann with otherwise equivalent train connections, Basel SBB is preferred.
///
/// `
/// ...
/// 8500009 16 Pregassona, Scuola Media % HS-Nr. 8500009 Prio Niedrig (16)
/// 8500010  4 Basel SBB                % HS-Nr. 8500010 Prio Erhöht  (4)
/// 8500016 16 Basel St. Johann         % HS-Nr. 8500016 Prio Niedrig (16)
/// ...
/// `
///
/// # KMINFO file
///
/// This file is primarily relevant for HAFAS. HAFAS recognises transfer points automatically.
/// This file should therefore only be used to assign numbers of 2 30000 and 0 (see below).
/// In Switzerland, however, it contains more figures. Specifically, various numbers between 0 and 30000.
/// The same figures indicate a similarly manageable changeover.
/// The file differs from BFPRIOS in that it defines closures and transfers in general,
/// i.e. a location can or cannot be used for transfers. The further division is a
/// configuration of the changeover logic used in addition to BFPRIOS. File contains:
///
/// - HS no.
/// - Transfer station
///     - 30000 = transfer point
///     - 0 = Blocking
///     - All other numbers are also used to represent transfer points (see above).
/// - HS name
///
/// ## Example (excerpt):
///
/// `
/// ...
/// 8500009    30 Pregassona, Scuola Media % HS-Nr. 8500009 Umstiegprio. 30 in Pregassona
/// 8500010  5000 Basel SBB                % HS-Nr. 8500009 Umstiegprio. 5000 in Basel SBB -> somit ein bevorzugter Umstiegsort
/// 8500016    23 Basel St. Johann         % ...
/// ...
/// `
///
/// # UMSTEIGB file
///
/// General transfer time or per stop. The file contains:
///
/// - a general default value for all stops if no other, more specific value is defined
///
/// Example (excerpt):
///
/// `
/// 9999999 02 02 STANDARD % Standard Umsteigezeit 2
/// `
///
/// - one transfer time per stop:
///     - Transfer time in minutes between service category (means of transport type) IC-IC
///     - Transfer time for all other offer categories
///     - HaltestellenName
///
/// ## Example (excerpt):
///
/// `
/// ...
/// 8389120 05 05 Verona, stazione FS % HS-Nr 8389120, Umsteigzeit IC-IC = 5, Umsteigzeit sonst = 5, HS = Verona
/// 8500010 05 05 Basel SBB           % HS-Nr 8500010, Umsteigzeit IC-IC = 5, Umsteigzeit sonst = 5, HS = Basel
/// 8500020 03 03 Muttenz             % HS-Nr 8500020, Umsteigzeit IC-IC = 3, Umsteigzeit sonst = 3, HS = Muttenz
/// ...
/// `
///
/// # BHFART
///
/// Definition of the type of stops, i.e. whether the stop should be able to serve as a start and/or destination,
/// or as a via location, and whether it has a global ID (for Switzerland the Swiss Location ID (SLOID)).
///
/// The BHFART_60 variant of the BHFART file also contains the risers (with an “a” as a prefix)
/// of the stations (with an “A” as a prefix). So if the example below says “A”,
/// it describes a stop and not a platform belonging to this stop. A stop can
/// have several platforms (i.e., for example, places to board and alight at the
/// stop in question). File contains:
///
/// - Restrictions:
///     - These stops are not to be offered as start, destination or via entries
///     - B = Selection and routing restrictions
///         - Selection restriction (usually “3” – start/finish restricted)
///         - Routing restriction (usually empty “”)
/// - and the Global ID of the stop and track:
///     - G = Global ID (in Switzerland: SLOID)
///         - Type designator (“a”/”A”, “A” only for *_60)
///         - SLOID
///
/// The format is included:
///
/// - Stop number
/// - Code (e.g.: see above) M*W
/// - Code details (e.g.: see above, a, A)
/// - Value (e.g.: see above) 3, “”, SLOID)
///
/// ## Example (excerpt):
///
/// `
/// .....bhfart
/// % Beschränkungen
/// 0000132 B 3                     % Bahn-2000-Strecke % HS-Nr. 0000132 Auswahlbeschränkung
/// 0000133 B 3                     % Centovalli        % HS-Nr. 0000133 Auswahlbeschränkung
/// ...
/// % Globale IDs
/// ...
/// 8500009 G a ch:1:sloid:9        % HS-Nr. 8500009, Typ: SLOID-HS, SLOID = ch:1:sloid:9
/// 8500010 G a ch:1:sloid:10       % HS-Nr. 8500010, Typ: SLOID-HS, SLOID = ch:1:sloid:10
/// 8500016 G a ch:1:sloid:16       % HS-Nr. 8500016, Typ: SLOID-HS, SLOID = ch:1:sloid:16
/// .....bhfart_60
/// % Beschränkungen
/// 0000132 B 3                     % Bahn-2000-Strecke % HS-Nr. 0000132 Auswahlbeschränkung
/// 0000133 B 3                     % Centovalli        % HS-Nr. 0000133 Auswahlbeschränkung
/// ...
/// % Globale IDs
/// ...
/// 8500010 G A ch:1:sloid:10       % HS-Nr. 8500010, Typ: SLOID-HS,    SLOID = ch:1:sloid:10
/// 8500010 G a ch:1:sloid:10:3:5   % HS-Nr. 8500010, Typ: SLOID-Steig, SLOID = ch:1:sloid:10:3:5
/// 8500010 G a ch:1:sloid:10:22:35 % HS-Nr. 8500010, Typ: SLOID-Steig, SLOID = ch:1:sloid:10:22:35
/// 8500010 G a ch:1:sloid:10:3:6   % ...
/// 8500010 G a ch:1:sloid:10:2:4   % ...
/// 8500010 G a ch:1:sloid:10:4:8   % ...
/// 8500010 G a ch:1:sloid:10:4:7   % ...
/// 8500010 G a ch:1:sloid:10:7:15  % ...
/// 8500010 G a ch:1:sloid:10:8:16  % ...
/// 8500010 G a ch:1:sloid:10:7:14  % ...
/// 8500010 G a ch:1:sloid:10:5:10  % ...
/// 8500010 G a ch:1:sloid:10:6:11  % ...
/// 8500010 G a ch:1:sloid:10:6:12  % ...
/// 8500010 G a ch:1:sloid:10:0:20  % ...
/// 8500010 G a ch:1:sloid:10:21:30 % ...
/// 8500010 G a ch:1:sloid:10:21:31 % ...
/// 8500010 G a ch:1:sloid:10:2:3   % ...
/// 8500010 G a ch:1:sloid:10:1:1   % ...
/// 8500010 G a ch:1:sloid:10:1:2   % ...
/// 8500010 G a ch:1:sloid:10:22:33 % ...
/// 8500010 G a ch:1:sloid:10:8:17  % ...
/// 8500010 G a ch:1:sloid:10:0:19  % HS-Nr. 8500010, Typ: SLOID-Steig, SLOID = ch:1:sloid:10:0:19
/// 8500010 G a ch:1:sloid:10:5:9   % HS-Nr. 8500010, Typ: SLOID-Steig, SLOID = ch:1:sloid:10:5:9
/// ...
/// `
///
/// Caveat: There are currently no different sloids for sectors and sector groups.
/// However, these can have their own coordinates. Depending on the application, the
/// sloid (if it is used as an id) should be supplemented
/// with “: “+”designation” (e.g. ch:1:sloid:7000:501:34:AB) in the internal system.
/// However, this is NOT a new official ID.
///
/// 8 file(s).
/// File(s) read by the parser:
/// BAHNHOF, BFKOORD_LV95, BFKOORD_WGS, BFPRIOS, KMINFO, UMSTEIGB, BHFART_60
/// ---
/// Files not used by the parser:
/// BHFART
use std::{error::Error, vec};

use nom::{
    Parser,
    branch::alt,
    bytes::complete::{tag, take_until},
    character::complete::{digit1, i16, i32, space1},
    combinator::{map, map_res, opt},
    multi::many0,
    number::complete::double,
    sequence::{preceded, terminated},
};
use rustc_hash::FxHashMap;

use crate::{
    models::{CoordinateSystem, Coordinates, Model, Stop, Version},
    parsing::{
        ColumnDefinition, ExpectedType, FastRowMatcher, FileParser, ParsedValue, RowDefinition,
        RowParser,
        helpers::{
            i32_from_n_digits_parser, read_lines, string_from_n_chars_parser,
            string_till_eol_parser,
        },
    },
    storage::ResourceStorage,
};

type StopStorageAndExchangeTimes = (ResourceStorage<Stop>, (i16, i16));

struct StopLine {
    stop_id: i32,
    designation: String,
    long_name: Option<String>,
    abbreviation: Option<String>,
    synonyms: Option<Vec<String>>,
}

struct CoordLine {
    stop_id: i32,
    x: f64,
    y: f64,
    altitude: f64,
}

struct PriosLine {
    stop_id: i32,
    exchange_priority: i16,
    name: String,
}

struct FlagsLine {
    stop_id: i32,
    exchange_flag: i16,
}

struct TimesLines {
    stop_id: i32,
    exchange_time_inter_city: i16,
    exchange_time_other: i16,
}

enum DescriptionLine {
    Comment,
    Restriction { stop_id: i32, restrictions: i16 },
    Sloid { stop_id: i32, sloid: String },
    Boarding { stop_id: i32, sloid: String },
    Country { stop_id: i32, country_code: String },
    Canton { stop_id: i32, canton_id: i32 },
}

fn comment_combinator<'a>()
-> impl Parser<&'a str, Output = DescriptionLine, Error = nom::error::Error<&'a str>> {
    map(tag("%"), |_| DescriptionLine::Comment)
}

fn restriction_combinator<'a>()
-> impl Parser<&'a str, Output = DescriptionLine, Error = nom::error::Error<&'a str>> {
    map(
        (
            i32,
            preceded(preceded(space1, tag("B")), preceded(space1, i16)),
        ),
        |(stop_id, restrictions)| DescriptionLine::Restriction {
            stop_id,
            restrictions,
        },
    )
}

fn sloid_combinator<'a>()
-> impl Parser<&'a str, Output = DescriptionLine, Error = nom::error::Error<&'a str>> {
    map(
        (
            i32,
            preceded(
                preceded(space1, tag("G A")),
                preceded(space1, string_till_eol_parser()),
            ),
        ),
        |(stop_id, sloid)| DescriptionLine::Sloid { stop_id, sloid },
    )
}

fn boarding_combinator<'a>()
-> impl Parser<&'a str, Output = DescriptionLine, Error = nom::error::Error<&'a str>> {
    map(
        (
            i32,
            preceded(
                preceded(space1, tag("G a")),
                preceded(space1, string_till_eol_parser()),
            ),
        ),
        |(stop_id, sloid)| DescriptionLine::Boarding { stop_id, sloid },
    )
}

fn country_combinator<'a>()
-> impl Parser<&'a str, Output = DescriptionLine, Error = nom::error::Error<&'a str>> {
    map(
        (
            i32,
            preceded(
                preceded(space1, tag("L")),
                preceded(space1, string_from_n_chars_parser(2)),
            ),
        ),
        |(stop_id, country_code)| DescriptionLine::Country {
            stop_id,
            country_code,
        },
    )
}

fn canton_combinator<'a>()
-> impl Parser<&'a str, Output = DescriptionLine, Error = nom::error::Error<&'a str>> {
    map(
        (
            i32,
            preceded(preceded(space1, tag("I KT")), preceded(space1, i32)),
        ),
        |(stop_id, canton_id)| DescriptionLine::Canton { stop_id, canton_id },
    )
}

fn parse_description_line(
    line: &str,
    stops: &mut FxHashMap<i32, Stop>,
) -> Result<(), Box<dyn Error>> {
    let (_, description_line) = alt((
        comment_combinator(),
        restriction_combinator(),
        sloid_combinator(),
        boarding_combinator(),
        country_combinator(),
        canton_combinator(),
    ))
    .parse(line)
    .map_err(|e| format!("Error {e} while parsing {line}"))?;

    match description_line {
        DescriptionLine::Comment => {
            // Do nothing it's a comment
        }
        DescriptionLine::Restriction {
            stop_id,
            restrictions,
        } => {
            if let Some(stop) = stops.get_mut(&stop_id) {
                stop.set_restrictions(restrictions);
            } else {
                log::info!("Unknown stop ID: {stop_id} for restrictions");
            }
        }
        DescriptionLine::Sloid { stop_id, sloid } => {
            if let Some(stop) = stops.get_mut(&stop_id) {
                stop.set_sloid(sloid);
            } else {
                log::info!("Unknown stop ID: {stop_id} for sloid");
            }
        }
        DescriptionLine::Boarding { stop_id, sloid } => {
            if let Some(stop) = stops.get_mut(&stop_id) {
                stop.add_boarding_area(sloid);
            } else {
                log::info!("Unknown stop ID: {stop_id} for boarding area");
            }
        }
        DescriptionLine::Country {
            stop_id: _,
            country_code: _,
        } => {
            // TODO: For the moment this line is not used
        }
        DescriptionLine::Canton {
            stop_id: _,
            canton_id: _,
        } => {
            // TODO: For the moment this line is not used
        }
    }
    Ok(())
}

fn designation_number_combinator<'a>()
-> impl Parser<&'a str, Output = i8, Error = nom::error::Error<&'a str>> {
    map_res(
        terminated(preceded(tag("$<"), digit1), tag(">")),
        |num: &str| num.parse::<i8>(),
    )
}

fn station_combinator<'a>()
-> impl Parser<&'a str, Output = StopLine, Error = nom::error::Error<&'a str>> {
    map_res(
        (
            i32,
            preceded(space1, map(take_until("$<"), |s: &str| String::from(s))),
            designation_number_combinator(),
            many0((
                preceded(tag("$"), take_until("$<")),
                designation_number_combinator(),
            )),
        ),
        |(stop_id, designation, num, optional_designations)| {
            if num != 1 {
                Err(format!("Error: absent principal name, got {num} instead"))
            } else {
                let mut long_name = None;
                let mut abbreviation = None;
                let mut synonyms = Vec::new();

                for (d, tag) in optional_designations {
                    if tag == 2 {
                        long_name = Some(String::from(d));
                    } else if tag == 3 {
                        abbreviation = Some(String::from(d));
                    } else if tag == 4 {
                        synonyms.push(String::from(d))
                    } else {
                        return Err(format!(
                            "Error: invalid num must be in range [1, 4], got {tag} instead"
                        ));
                    }
                }
                Ok(StopLine {
                    stop_id,
                    designation,
                    long_name,
                    abbreviation,
                    synonyms: if synonyms.is_empty() {
                        None
                    } else {
                        Some(synonyms)
                    },
                })
            }
        },
    )
}

fn coordinates_combinator<'a>()
-> impl Parser<&'a str, Output = CoordLine, Error = nom::error::Error<&'a str>> {
    map(
        (
            i32,
            preceded(space1, double),
            preceded(space1, double),
            preceded(space1, double),
        ),
        |(stop_id, x, y, altitude)| CoordLine {
            stop_id,
            x,
            y,
            altitude,
        },
    )
}

fn prios_combinator<'a>()
-> impl Parser<&'a str, Output = PriosLine, Error = nom::error::Error<&'a str>> {
    map(
        (
            i32,
            preceded(space1, i16),
            preceded(space1, string_till_eol_parser()),
        ),
        |(stop_id, exchange_priority, name)| PriosLine {
            stop_id,
            exchange_priority,
            name,
        },
    )
}

fn flags_combinator<'a>()
-> impl Parser<&'a str, Output = FlagsLine, Error = nom::error::Error<&'a str>> {
    map((i32, preceded(space1, i16)), |(stop_id, exchange_flag)| {
        FlagsLine {
            stop_id,
            exchange_flag,
        }
    })
}

fn times_combinator<'a>()
-> impl Parser<&'a str, Output = TimesLines, Error = nom::error::Error<&'a str>> {
    map(
        (i32, preceded(space1, i16), preceded(space1, i16)),
        |(stop_id, exchange_time_inter_city, exchange_time_other)| TimesLines {
            stop_id,
            exchange_time_inter_city,
            exchange_time_other,
        },
    )
}

fn parse_stop_line(line: &str, stops: &mut FxHashMap<i32, Stop>) -> Result<(), Box<dyn Error>> {
    let (
        _,
        StopLine {
            stop_id,
            designation,
            long_name,
            abbreviation,
            synonyms,
        },
    ) = station_combinator()
        .parse(line)
        .map_err(|e| format!("Error {e} while parsing {line}"))?;

    stops.insert(
        stop_id,
        Stop::new(stop_id, designation, long_name, abbreviation, synonyms),
    );
    Ok(())
}

fn parse_coord_line(
    line: &str,
    stops: &mut FxHashMap<i32, Stop>,
    coordinate_system: CoordinateSystem,
) -> Result<(), Box<dyn Error>> {
    let (
        _,
        CoordLine {
            stop_id,
            x,
            y,
            altitude: _, // altitude is not stored at the moment
        },
    ) = coordinates_combinator()
        .parse(line)
        .map_err(|e| format!("Error {e} while parsing {line}"))?;

    let stop = stops
        .get_mut(&stop_id)
        .ok_or(format!("Unknown stop ID {stop_id}"))?;

    match coordinate_system {
        CoordinateSystem::LV95 => {
            stop.set_lv95_coordinates(Coordinates::new(coordinate_system, x, y))
        }
        CoordinateSystem::WGS84 => {
            stop.set_wgs84_coordinates(Coordinates::new(coordinate_system, y, x)) // x, y
            // are stored in reverse order
        }
    }

    Ok(())
}

fn parse_prios_line(line: &str, stops: &mut FxHashMap<i32, Stop>) -> Result<(), Box<dyn Error>> {
    let (
        _,
        PriosLine {
            stop_id,
            exchange_priority,
            name: _,
        },
    ) = prios_combinator()
        .parse(line)
        .map_err(|e| format!("Error {e} while parsing {line}"))?;

    let stop = stops
        .get_mut(&stop_id)
        .ok_or(format!("Unknown stop ID {stop_id}"))?;
    stop.set_exchange_priority(exchange_priority);

    Ok(())
}

fn parse_flags_line(line: &str, stops: &mut FxHashMap<i32, Stop>) -> Result<(), Box<dyn Error>> {
    let (
        _,
        FlagsLine {
            stop_id,
            exchange_flag,
        },
    ) = flags_combinator()
        .parse(line)
        .map_err(|e| format!("Error {e} while parsing {line}"))?;

    let stop = stops
        .get_mut(&stop_id)
        .ok_or(format!("Unknown stop ID {stop_id}"))?;
    stop.set_exchange_flag(exchange_flag);

    Ok(())
}

fn parse_times_line(
    line: &str,
    stops: &mut FxHashMap<i32, Stop>,
) -> Result<Option<(i16, i16)>, Box<dyn Error>> {
    let (
        _,
        TimesLines {
            stop_id,
            exchange_time_inter_city,
            exchange_time_other,
        },
    ) = times_combinator()
        .parse(line)
        .map_err(|e| format!("Error {e} while parsing {line}"))?;

    let exchange_time = Some((exchange_time_inter_city, exchange_time_other));

    if stop_id == 9999999 {
        // The first row of the file has the stop ID number 9999999.
        // It contains default exchange times to be used when a stop has no specific exchange time.
        Ok(exchange_time)
    } else {
        let stop = stops
            .get_mut(&stop_id)
            .ok_or(format!("Unknown Stop ID {stop_id}"))?;
        stop.set_exchange_time(exchange_time);
        Ok(None)
    }
}

pub fn parse(version: Version, path: &str) -> Result<StopStorageAndExchangeTimes, Box<dyn Error>> {
    log::info!("Parsing BAHNHOF...");

    let mut stops = FxHashMap::default();

    read_lines(&format!("{path}/BAHNHOF"), 0)?
        .into_iter()
        .filter(|line| !line.trim().is_empty())
        .try_for_each(|line| {
            parse_stop_line(&line, &mut stops).map_err(|e| format!("Error: {e}, for line: {line}"))
        })?;

    log::info!("Parsing BFKOORD_LV95...");
    read_lines(&format!("{path}/BFKOORD_LV95"), 0)?
        .into_iter()
        .filter(|line| !line.trim().is_empty())
        .try_for_each(|line| {
            parse_coord_line(&line, &mut stops, CoordinateSystem::LV95)
                .map_err(|e| format!("Error: {e}, for line: {line}"))
        })?;

    log::info!("Parsing BFKOORD_WGS...");
    read_lines(&format!("{path}/BFKOORD_WGS"), 0)?
        .into_iter()
        .filter(|line| !line.trim().is_empty())
        .try_for_each(|line| {
            parse_coord_line(&line, &mut stops, CoordinateSystem::WGS84)
                .map_err(|e| format!("Error: {e}, for line: {line}"))
        })?;

    log::info!("Parsing BFPRIOS...");
    read_lines(&format!("{path}/BFPRIOS"), 0)?
        .into_iter()
        .filter(|line| !line.trim().is_empty())
        .try_for_each(|line| {
            parse_prios_line(&line, &mut stops).map_err(|e| format!("Error: {e}, for line: {line}"))
        })?;

    log::info!("Parsing KMINFO...");
    read_lines(&format!("{path}/KMINFO"), 0)?
        .into_iter()
        .filter(|line| !line.trim().is_empty())
        .try_for_each(|line| {
            parse_flags_line(&line, &mut stops).map_err(|e| format!("Error: {e}, for line: {line}"))
        })?;

    log::info!("Parsing UMSTEIGB...");
    let default_exchange_time = read_lines(&format!("{path}/UMSTEIGB"), 0)?
        .into_iter()
        .filter(|line| !line.trim().is_empty())
        .map(|line| {
            parse_times_line(&line, &mut stops).map_err(|e| format!("Error: {e}, for line: {line}"))
        })
        .try_fold(None, |acc, curr| match (curr, acc) {
            (Err(e), _) => Err(e),
            (Ok(None), None) => Ok(None),
            (_, Some(w)) => Ok(Some(w)),
            (Ok(Some(v)), None) => Ok(Some(v)),
        })?
        .ok_or("Error default exchante time not defined")?;

    log::info!("Parsing BHFART...");
    read_lines(&format!("{path}/BHFART"), 0)?
        .into_iter()
        .filter(|line| !line.trim().is_empty())
        .try_for_each(|line| {
            parse_description_line(&line, &mut stops)
                .map_err(|e| format!("Error: {e}, for line: {line}"))
        })?;

    Ok((ResourceStorage::new(stops), default_exchange_time))
}

pub fn old_parse(
    version: Version,
    path: &str,
) -> Result<StopStorageAndExchangeTimes, Box<dyn Error>> {
    log::info!("Parsing BAHNHOF...");
    #[rustfmt::skip]
    let row_parser = RowParser::new(vec![
        // This row is used to create a Stop instance.
        RowDefinition::from(vec![
            ColumnDefinition::new(1, 7, ExpectedType::Integer32),
            ColumnDefinition::new(13, -1, ExpectedType::String), // Should be 13-62, but some entries go beyond column 62.
        ]),
    ]);
    let parser = FileParser::new(&format!("{path}/BAHNHOF"), row_parser)?;

    let data = parser
        .parse()
        .map(|x| x.map(|(_, _, values)| create_instance(values))?)
        .collect::<Result<Vec<_>, _>>()?;
    let mut data = Stop::vec_to_map(data);

    log::info!("Parsing BFKOORD_LV95...");
    load_coordinates(version, path, CoordinateSystem::LV95, &mut data)?;
    log::info!("Parsing BFKOORD_WGS...");
    load_coordinates(version, path, CoordinateSystem::WGS84, &mut data)?;
    log::info!("Parsing BFPRIOS...");
    load_exchange_priorities(path, &mut data)?;
    log::info!("Parsing KMINFO...");
    load_exchange_flags(path, &mut data)?;
    log::info!("Parsing UMSTEIGB...");
    let default_exchange_time = load_exchange_times(path, &mut data)?;
    log::info!("Parsing BHFART...");
    load_descriptions(version, path, &mut data)?;

    Ok((ResourceStorage::new(data), default_exchange_time))
}

fn load_coordinates(
    version: Version,
    path: &str,
    coordinate_system: CoordinateSystem,
    data: &mut FxHashMap<i32, Stop>,
) -> Result<(), Box<dyn Error>> {
    #[rustfmt::skip]
    let row_parser = RowParser::new(vec![
        // This row contains the LV95/WGS84 coordinates.
        RowDefinition::from(
            match version {
                Version::V_5_40_41_2_0_4 => vec![
                    ColumnDefinition::new(1, 7, ExpectedType::Integer32),
                    ColumnDefinition::new(9, 18, ExpectedType::Float),
                    ColumnDefinition::new(20, 29, ExpectedType::Float),
                    ColumnDefinition::new(31, 36, ExpectedType::Integer16),
                ],
                Version::V_5_40_41_2_0_5 | Version::V_5_40_41_2_0_6 | Version::V_5_40_41_2_0_7 => vec![
                    ColumnDefinition::new(1, 7, ExpectedType::Integer32),
                    ColumnDefinition::new(9, 19, ExpectedType::Float),
                    ColumnDefinition::new(21, 31, ExpectedType::Float),
                    ColumnDefinition::new(33, 39, ExpectedType::Integer16),
                ],
            }
        ),
    ]);
    let filename = match coordinate_system {
        CoordinateSystem::LV95 => "BFKOORD_LV95",
        CoordinateSystem::WGS84 => "BFKOORD_WGS",
    };
    let parser = FileParser::new(&format!("{path}/{filename}"), row_parser)?;

    parser.parse().try_for_each(|x| {
        let (_, _, values) = x?;
        set_coordinates(values, coordinate_system, data)?;
        Ok(())
    })
}

fn load_exchange_priorities(
    path: &str,
    data: &mut FxHashMap<i32, Stop>,
) -> Result<(), Box<dyn Error>> {
    #[rustfmt::skip]
    let row_parser = RowParser::new(vec![
        // This row contains the changing priority.
        RowDefinition::from(vec![
            ColumnDefinition::new(1, 7, ExpectedType::Integer32),
            ColumnDefinition::new(9, 10, ExpectedType::Integer16),
        ]),
    ]);
    let parser = FileParser::new(&format!("{path}/BFPRIOS"), row_parser)?;

    parser.parse().try_for_each(|x| {
        let (_, _, values) = x?;
        set_exchange_priority(values, data)?;
        Ok(())
    })
}

fn load_exchange_flags(path: &str, data: &mut FxHashMap<i32, Stop>) -> Result<(), Box<dyn Error>> {
    #[rustfmt::skip]
    let row_parser = RowParser::new(vec![
        // This row contains the changing flag.
        RowDefinition::from(vec![
            ColumnDefinition::new(1, 7, ExpectedType::Integer32),
            ColumnDefinition::new(9, 13, ExpectedType::Integer16),
        ]),
    ]);
    let parser = FileParser::new(&format!("{path}/KMINFO"), row_parser)?;

    parser.parse().try_for_each(|x| {
        let (_, _, values) = x?;
        set_exchange_flag(values, data)?;
        Ok(())
    })
}

fn load_exchange_times(
    path: &str,
    data: &mut FxHashMap<i32, Stop>,
) -> Result<(i16, i16), Box<dyn Error>> {
    #[rustfmt::skip]
    let row_parser = RowParser::new(vec![
        // This row contains the changing time.
        RowDefinition::from(vec![
            ColumnDefinition::new(1, 7, ExpectedType::Integer32),
            ColumnDefinition::new(9, 10, ExpectedType::Integer16),
            ColumnDefinition::new(12, 13, ExpectedType::Integer16),
        ]),
    ]);
    let parser = FileParser::new(&format!("{path}/UMSTEIGB"), row_parser)?;

    let mut default_exchange_time = (0, 0);

    parser.parse().try_for_each(|x| {
        let (_, _, values) = x?;
        if let Some(x) = set_exchange_time(values, data)? {
            default_exchange_time = x;
        }
        Ok::<(), Box<dyn Error>>(())
    })?;

    Ok(default_exchange_time)
}

fn load_descriptions(
    version: Version,
    path: &str,
    data: &mut FxHashMap<i32, Stop>,
) -> Result<(), Box<dyn Error>> {
    const ROW_A: i32 = 1;
    const ROW_B: i32 = 2;
    const ROW_C: i32 = 3;
    const ROW_D: i32 = 4;
    const ROW_E: i32 = 5;
    const ROW_F: i32 = 6;

    #[rustfmt::skip]
    let row_parser = RowParser::new(vec![
        // This row is ignored.
        RowDefinition::new(ROW_A, Box::new(FastRowMatcher::new(1, 1, "%", true)), Vec::new()),
        // This row contains the restrictions.
        RowDefinition::new(ROW_B, Box::new(FastRowMatcher::new(9, 1, "B", true)), vec![
            ColumnDefinition::new(1, 7, ExpectedType::Integer32),
            ColumnDefinition::new(11, 12, ExpectedType::Integer16),
        ]),
        // This row contains the SLOID.
        RowDefinition::new(ROW_C, Box::new(FastRowMatcher::new(11, 1, "A", true)), vec![
            ColumnDefinition::new(1, 7, ExpectedType::Integer32),
            ColumnDefinition::new(13, -1, ExpectedType::String),
        ]),
        // This row contains the boarding areas.
        RowDefinition::new(ROW_D, Box::new(FastRowMatcher::new(11, 1, "a", true)), vec![
            ColumnDefinition::new(1, 7, ExpectedType::Integer32),
            ColumnDefinition::new(13, -1, ExpectedType::String),
        ]),
        // This row contains the country
        RowDefinition::new(ROW_E, Box::new(FastRowMatcher::new(9, 1, "L", true)), vec![
            ColumnDefinition::new(1, 7, ExpectedType::Integer32),
            ColumnDefinition::new(11, 12, ExpectedType::String),
        ]),
        // This row contains the KT (kanton) information
        RowDefinition::new(ROW_F, Box::new(FastRowMatcher::new(9, 1, "I", true)), vec![
            ColumnDefinition::new(1, 7, ExpectedType::Integer32),
            ColumnDefinition::new(11, 12, ExpectedType::String),
            ColumnDefinition::new(14, 22, ExpectedType::Integer32),
        ]),
    ]);

    let bhfart = match version {
        Version::V_5_40_41_2_0_4 | Version::V_5_40_41_2_0_5 | Version::V_5_40_41_2_0_6 => {
            "BHFART_60"
        }
        Version::V_5_40_41_2_0_7 => "BHFART",
    };
    let parser = FileParser::new(&format!("{path}/{bhfart}"), row_parser)?;

    parser.parse().try_for_each(|x| {
        let (id, _, values) = x?;
        match id {
            ROW_A => {}
            ROW_B => set_restrictions(values, data)?,
            ROW_C => set_sloid(values, data)?,
            ROW_D => add_boarding_area(values, data)?,
            ROW_E => {
                // TODO: add possibility to use Land data
            }
            ROW_F => {
                // TODO: add possibility to use KT information and the associated number
            }
            _ => unreachable!(),
        }
        Ok(())
    })
}

// ------------------------------------------------------------------------------------------------
// --- Data Processing Functions
// ------------------------------------------------------------------------------------------------

fn create_instance(mut values: Vec<ParsedValue>) -> Result<Stop, Box<dyn Error>> {
    let id: i32 = values.remove(0).into();
    let designations: String = values.remove(0).into();

    let (name, long_name, abbreviation, synonyms) = parse_designations(designations)?;

    Ok(Stop::new(id, name, long_name, abbreviation, synonyms))
}

fn set_coordinates(
    mut values: Vec<ParsedValue>,
    coordinate_system: CoordinateSystem,
    data: &mut FxHashMap<i32, Stop>,
) -> Result<(), Box<dyn Error>> {
    let stop_id: i32 = values.remove(0).into();
    let mut xy1: f64 = values.remove(0).into();
    let mut xy2: f64 = values.remove(0).into();
    // Altitude is not stored, as it is not provided for 95% of stops.
    let _altitude: i16 = values.remove(0).into();

    if coordinate_system == CoordinateSystem::WGS84 {
        // WGS84 coordinates are stored in reverse order for some unknown reason.
        (xy1, xy2) = (xy2, xy1);
    }

    let stop = data.get_mut(&stop_id).ok_or("Unknown ID")?;
    let coordinate = Coordinates::new(coordinate_system, xy1, xy2);

    match coordinate_system {
        CoordinateSystem::LV95 => stop.set_lv95_coordinates(coordinate),
        CoordinateSystem::WGS84 => stop.set_wgs84_coordinates(coordinate),
    }

    Ok(())
}

fn set_exchange_priority(
    mut values: Vec<ParsedValue>,
    data: &mut FxHashMap<i32, Stop>,
) -> Result<(), Box<dyn Error>> {
    let stop_id: i32 = values.remove(0).into();
    let exchange_priority: i16 = values.remove(0).into();

    let stop = data.get_mut(&stop_id).ok_or("Unknown ID")?;
    stop.set_exchange_priority(exchange_priority);

    Ok(())
}

fn set_exchange_flag(
    mut values: Vec<ParsedValue>,
    data: &mut FxHashMap<i32, Stop>,
) -> Result<(), Box<dyn Error>> {
    let stop_id: i32 = values.remove(0).into();
    let exchange_flag: i16 = values.remove(0).into();

    let stop = data.get_mut(&stop_id).ok_or("Unknown ID")?;
    stop.set_exchange_flag(exchange_flag);

    Ok(())
}

fn set_exchange_time(
    mut values: Vec<ParsedValue>,
    data: &mut FxHashMap<i32, Stop>,
) -> Result<Option<(i16, i16)>, Box<dyn Error>> {
    let stop_id: i32 = values.remove(0).into();
    let exchange_time_inter_city: i16 = values.remove(0).into();
    let exchange_time_other: i16 = values.remove(0).into();

    let exchange_time = Some((exchange_time_inter_city, exchange_time_other));

    if stop_id == 9999999 {
        // The first row of the file has the stop ID number 9999999.
        // It contains default exchange times to be used when a stop has no specific exchange time.
        Ok(exchange_time)
    } else {
        let stop = data.get_mut(&stop_id).ok_or("Unknown ID")?;
        stop.set_exchange_time(exchange_time);
        Ok(None)
    }
}

fn set_restrictions(
    mut values: Vec<ParsedValue>,
    data: &mut FxHashMap<i32, Stop>,
) -> Result<(), Box<dyn Error>> {
    let stop_id: i32 = values.remove(0).into();
    let restrictions: i16 = values.remove(0).into();

    if let Some(stop) = data.get_mut(&stop_id) {
        stop.set_restrictions(restrictions);
    } else {
        log::info!("Unknown ID: {stop_id} for restrictions");
    }

    Ok(())
}

fn set_sloid(
    mut values: Vec<ParsedValue>,
    data: &mut FxHashMap<i32, Stop>,
) -> Result<(), Box<dyn Error>> {
    let stop_id: i32 = values.remove(0).into();
    let sloid: String = values.remove(0).into();

    if let Some(stop) = data.get_mut(&stop_id) {
        stop.set_sloid(sloid);
    } else {
        log::info!("Unknown ID: {stop_id} for sloid");
    }

    Ok(())
}

fn add_boarding_area(
    mut values: Vec<ParsedValue>,
    data: &mut FxHashMap<i32, Stop>,
) -> Result<(), Box<dyn Error>> {
    let stop_id: i32 = values.remove(0).into();
    let sloid: String = values.remove(0).into();

    if let Some(stop) = data.get_mut(&stop_id) {
        stop.add_boarding_area(sloid);
    } else {
        log::info!("Unknown ID: {stop_id} for boarding area");
    }

    Ok(())
}

// ------------------------------------------------------------------------------------------------
// --- Helper Functions
// ------------------------------------------------------------------------------------------------

type NameAndAlternatives = (String, Option<String>, Option<String>, Option<Vec<String>>);

fn parse_designations(designations: String) -> Result<NameAndAlternatives, Box<dyn Error>> {
    let designations = designations
        .split('>')
        .filter(|&s| !s.is_empty())
        .map(|s| -> Result<(i32, String), Box<dyn Error>> {
            let s = s.replace('$', "");
            let mut parts = s.split('<');

            let v = parts.next().ok_or("Missing value part")?.to_string();
            let k = parts.next().ok_or("Missing value part")?.parse::<i32>()?;

            Ok((k, v))
        })
        .try_fold(
            FxHashMap::default(),
            |mut acc: std::collections::HashMap<i32, Vec<String>, _>, item| {
                let (k, v) = item?;
                acc.entry(k).or_default().push(v);
                Ok::<_, Box<dyn Error>>(acc)
            },
        )?;

    let name = designations.get(&1).ok_or("Missing stop name")?[0].clone();
    let long_name = designations.get(&2).map(|x| x[0].clone());
    let abbreviation = designations.get(&3).map(|x| x[0].clone());
    let synonyms = designations.get(&4).cloned();

    Ok((name, long_name, abbreviation, synonyms))
}
