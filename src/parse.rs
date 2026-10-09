//! HRDF 5.40.41 (realisation specification 2.0.7) files are fixed-column text files.

use std::{
    fs::File,
    io::{BufRead, BufReader},
    num::NonZeroU32,
    path::Path,
    str::FromStr,
};

use chrono::NaiveDate;
use rustc_hash::FxHashMap;
use zip::ZipArchive;

use crate::{
    AdminTransferTime, BitField, CoordinateSystem, Coordinates, Error, Footpath, Journey,
    JourneyTransferTime, LineSelector, LineTransferTime, Result, Section, Stop, StopTime,
    ThroughService, Timetable,
};

pub fn timetable(path: &Path) -> Result<Timetable> {
    let mut zip = ZipArchive::new(File::open(path)?)?;
    let mut tt = Timetable::default();

    let mut dates = Vec::new();
    read(&mut zip, "ECKDATEN", |line| {
        if dates.len() < 2 {
            dates.push(
                NaiveDate::parse_from_str(line.trim(), "%d.%m.%Y").map_err(|e| e.to_string())?,
            );
        }
        Ok(())
    })?;
    let [start, end] = dates[..].try_into().map_err(|_| Error::Parse {
        file: "ECKDATEN",
        line: 0,
        msg: "missing start or end date".into(),
    })?;
    (tt.start, tt.end) = (start, end);

    read(&mut zip, "BITFELD", |line| {
        let hex = col(line, 8, 103);
        let bytes = (0..hex.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(hex.get(i..i + 2).unwrap_or("?"), 16))
            .collect::<Result<_, _>>()
            .map_err(|_| format!("invalid bit field {hex:?}"))?;
        tt.bit_fields.insert(num(col(line, 1, 6))?, BitField(bytes));
        Ok(())
    })?;

    read(&mut zip, "BAHNHOF", |line| {
        let id = num(col(line, 1, 7))?;
        // Names look like "Basel SBB$<1>$Basel$<2>"; the first one is the official name.
        let name = line
            .get(12..)
            .unwrap_or("")
            .split("$<")
            .next()
            .unwrap_or("")
            .trim()
            .into();
        let stop = Stop {
            id,
            name,
            transfer_allowed: true,
            ..Default::default()
        };
        tt.stops.insert(id, stop);
        Ok(())
    })?;
    read(&mut zip, "BFKOORD_LV95", |line| {
        let [id, easting, northing] = fields(line)?;
        if let Some(stop) = tt.stops.get_mut(&num(id)?) {
            stop.lv95 = Some(Coordinates::new(
                CoordinateSystem::LV95,
                num(easting)?,
                num(northing)?,
            ));
        }
        Ok(())
    })?;
    read(&mut zip, "BFKOORD_WGS", |line| {
        let [id, longitude, latitude] = fields(line)?;
        if let Some(stop) = tt.stops.get_mut(&num(id)?) {
            stop.wgs84 = Some(Coordinates::new(
                CoordinateSystem::WGS84,
                num(latitude)?,
                num(longitude)?,
            ));
        }
        Ok(())
    })?;
    read(&mut zip, "KMINFO", |line| {
        let [id, value] = fields(line)?;
        if let Some(stop) = tt.stops.get_mut(&num(id)?) {
            stop.transfer_allowed = num::<i32>(value)? != 0;
        }
        Ok(())
    })?;

    read(&mut zip, "UMSTEIGB", |line| {
        let [id, ic, other] = fields(line)?;
        let (id, minutes) = (num(id)?, (num(ic)?, num(other)?));
        if id == 9999999 {
            tt.default_transfer_time = minutes;
        } else if let Some(stop) = tt.stops.get_mut(&id) {
            stop.transfer_time = Some(minutes);
        }
        Ok(())
    })?;
    read(&mut zip, "UMSTEIGV", |line| {
        let [stop, from, to, minutes] = fields(line)?;
        tt.admin_transfer_times.push(AdminTransferTime {
            stop_id: if stop == "@@@@@@@" {
                None
            } else {
                Some(num(stop)?)
            },
            from: from.into(),
            to: to.into(),
            minutes: num(minutes)?,
        });
        Ok(())
    })?;
    read(&mut zip, "UMSTEIGZ", |line| {
        tt.journey_transfer_times.push(JourneyTransferTime {
            stop_id: num(col(line, 1, 7))?,
            from: (num(col(line, 9, 14))?, col(line, 16, 21).into()),
            to: (num(col(line, 23, 28))?, col(line, 30, 35).into()),
            minutes: num(col(line, 37, 39))?,
            bit_field: opt_num(col(line, 42, 47))?.unwrap_or(0),
        });
        Ok(())
    })?;

    read(&mut zip, "UMSTEIGL", |line| {
        let any = |s: &str| (s != "*").then(|| s.to_string());
        let selector = |at: usize| LineSelector {
            admin: any(col(line, at, at + 5)),
            transport: any(col(line, at + 7, at + 9)),
            line: any(col(line, at + 11, at + 18)),
            direction: any(col(line, at + 20, at + 20)).and_then(|d| d.chars().next()),
        };
        tt.line_transfer_times.push(LineTransferTime {
            stop_id: match col(line, 1, 7) {
                "@@@@@@@" => None,
                stop => Some(num(stop)?),
            },
            from: selector(9),
            to: selector(31),
            minutes: num(col(line, 53, 55))?,
        });
        Ok(())
    })?;

    read(&mut zip, "METABHF", |line| {
        // "*A" lines describe the transfer below them; "group: stops" lines group stops.
        if !line.starts_with('*') && !line.contains(':') {
            let [from, to, minutes] = fields(line)?;
            tt.footpaths.push(Footpath {
                from: num(from)?,
                to: num(to)?,
                minutes: num(minutes)?,
            });
        }
        Ok(())
    })?;
    read(&mut zip, "DURCHBI", |line| {
        let [number_1, admin_1, stop_id, number_2, admin_2, bit_field] = fields(line)?;
        tt.through_services.push(ThroughService {
            from: (num(number_1)?, admin_1.into()),
            to: (num(number_2)?, admin_2.into()),
            stop_id: num(stop_id)?,
            bit_field: num(bit_field)?,
        });
        Ok(())
    })?;

    // Operators: "00001 K "AFA" L ..." names operator 1, "00001 : 000813" gives its administration.
    let mut names = FxHashMap::default();
    read(&mut zip, "BETRIEB_DE", |line| {
        let id = col(line, 1, 5).to_string();
        match col(line, 7, 7) {
            "K" => _ = names.insert(id, line.split('"').nth(1).unwrap_or("").to_string()),
            ":" => {
                _ = names
                    .get(&id)
                    .map(|name| tt.operators.insert(col(line, 9, 14).into(), name.clone()))
            }
            _ => {}
        }
        Ok(())
    })?;

    let mut fplan = Fplan::default();
    read(&mut zip, "RICHTUNG", |line| {
        fplan
            .directions
            .insert(col(line, 1, 7).into(), tt.directions.len() as u32);
        tt.directions
            .push(line.get(8..).unwrap_or("").trim().into());
        Ok(())
    })?;
    read(&mut zip, "LINIE", |line| {
        // "0000022 N T 56": the short name of line 22.
        if col(line, 9, 11) == "N T" {
            fplan.line_names.insert(
                num(col(line, 1, 7))?,
                line.get(12..).unwrap_or("").trim().into(),
            );
        }
        Ok(())
    })?;
    read(&mut zip, "FPLAN", |line| fplan.line(line))?;
    fplan.finish();
    (tt.journeys, tt.lines) = (fplan.journeys, fplan.lines);

    // Offer categories: "B    6 A  0 B        0        #002" links code B to category 002, named
    // "category002 Bus" in the <Englisch> section.
    let (mut categories, mut names, mut english) = (Vec::new(), FxHashMap::default(), false);
    read(&mut zip, "ZUGART", |line| {
        if line.starts_with('<') {
            english = line == "<Englisch>";
        } else if let (true, Some((key, name))) = (english, line.split_once(' ')) {
            if let Some(number) = key.strip_prefix("category") {
                names.insert(number.to_string(), name.trim().to_string());
            }
        } else if let Some(number) = line
            .split_whitespace()
            .last()
            .and_then(|t| t.strip_prefix('#'))
        {
            categories.push((col(line, 1, 3).to_string(), number.to_string()));
        }
        Ok(())
    })?;
    tt.transport_names = (categories.into_iter())
        .filter_map(|(code, number)| names.get(&number).map(|name| (code, name.clone())))
        .collect();

    // Platforms: "8505307 000025 000011 #0000002 1955 223315" puts journey 25 of administration
    // 000011 at platform 2 of stop 8505307 (at 19:55, on the days of bit field 223315), and
    // "8505307 #0000002 G '3'" names it. ponytail: one platform per call, the one of the
    // journey's own days when the platform changes with the day.
    let admin = |admin: &str| -> [u8; 6] {
        std::array::from_fn(|i| *admin.as_bytes().get(i).unwrap_or(&b' '))
    };
    let mut journeys_by_key: FxHashMap<(i32, [u8; 6]), Vec<u32>> = FxHashMap::default();
    for (i, journey) in tt.journeys.iter().enumerate() {
        journeys_by_key
            .entry((journey.number, admin(&journey.admin)))
            .or_default()
            .push(i as u32);
    }
    read(&mut zip, "GLEISE_LV95", |line| {
        if col(line, 9, 9) == "#" {
            if col(line, 18, 18) == "G" {
                let name = line.get(19..).unwrap_or("").trim().trim_matches('\'');
                tt.platforms.insert(
                    (num(col(line, 1, 7))?, num(col(line, 10, 16))?),
                    name.into(),
                );
            }
            return Ok(());
        }
        let (stop_id, number) = (num(col(line, 1, 7))?, num(col(line, 9, 14))?);
        let platform = NonZeroU32::new(num(col(line, 24, 30))?);
        let at = opt_num::<i16>(col(line, 32, 35))?.map(|hhmm| hhmm / 100 * 60 + hhmm % 100);
        let bit_field = opt_num::<i32>(col(line, 37, 42))?.unwrap_or(0);
        for &j in journeys_by_key
            .get(&(number, admin(col(line, 16, 21))))
            .into_iter()
            .flatten()
        {
            let journey = &mut tt.journeys[j as usize];
            let own_days = journey.sections.iter().any(|s| s.bit_field == bit_field);
            let at_time =
                |c: &StopTime| at.is_none_or(|t| c.departure == Some(t) || c.arrival == Some(t));
            for call in journey
                .stops
                .iter_mut()
                .filter(|c| c.stop_id == stop_id && at_time(c))
            {
                if call.platform.is_none() || own_days {
                    call.platform = platform;
                }
            }
        }
        Ok(())
    })?;
    Ok(tt)
}

/// Calls `f` on every line of `file`, adding the position to its errors.
fn read(
    zip: &mut ZipArchive<File>,
    file: &'static str,
    mut f: impl FnMut(&str) -> Result<(), String>,
) -> Result<()> {
    log::info!("Parsing {file}");
    let mut reader = BufReader::with_capacity(1 << 20, zip.by_name(file)?);
    let mut line = String::new();
    let mut number = 0;
    while reader.read_line(&mut line)? > 0 {
        number += 1;
        let content = line
            .trim_start_matches('\u{feff}')
            .trim_end_matches(['\n', '\r']);
        f(content).map_err(|msg| Error::Parse {
            file,
            line: number,
            msg,
        })?;
        line.clear();
    }
    Ok(())
}

/// Trimmed content of the 1-based, inclusive column range; empty past the end of the line.
fn col(line: &str, from: usize, to: usize) -> &str {
    line.get(from - 1..to.min(line.len())).unwrap_or("").trim()
}

/// The first `N` space-separated fields before the "%" comment.
fn fields<const N: usize>(line: &str) -> Result<[&str; N], String> {
    let fields: Vec<_> = line
        .split('%')
        .next()
        .unwrap_or("")
        .split_whitespace()
        .take(N)
        .collect();
    fields
        .try_into()
        .map_err(|_| format!("expected {N} fields"))
}

fn num<T: FromStr>(s: &str) -> Result<T, String> {
    s.parse().map_err(|_| format!("invalid number {s:?}"))
}

fn opt_num<T: FromStr>(s: &str) -> Result<Option<T>, String> {
    if s.is_empty() {
        Ok(None)
    } else {
        num(s).map(Some)
    }
}

/// "-01846" is 18:46 with boarding (or alighting) forbidden. Hours (HHH) can exceed 23.
fn time(s: &str) -> Result<Option<(i16, bool)>, String> {
    if s.is_empty() {
        return Ok(None);
    }
    let hhhmm: i16 = num(s.trim_start_matches('-'))?;
    Ok(Some((hhhmm / 100 * 60 + hhhmm % 100, !s.starts_with('-'))))
}

/// An *A VE line: the bit field of the stops from `from` (at `departure`) to `to` (at
/// `arrival`). Missing stops mean the start or end of the journey.
struct RawSection {
    from: Option<i32>,
    to: Option<i32>,
    departure: Option<i16>,
    arrival: Option<i16>,
    bit_field: i32,
}

/// FPLAN parsing state: a journey is complete when the next *Z line starts.
#[derive(Default)]
struct Fplan {
    journeys: Vec<Journey>,
    current: Journey,
    sections: Vec<RawSection>,
    /// (repetitions, minutes between them) of a cyclic journey.
    cycles: (i16, i16),
    /// Distinct line names and their index, LINIE short names by line number.
    lines: Vec<String>,
    line_ids: FxHashMap<String, u32>,
    line_names: FxHashMap<i32, String>,
    /// RICHTUNG code -> index into `Timetable::directions`.
    directions: FxHashMap<String, u32>,
}

impl Fplan {
    fn line(&mut self, line: &str) -> Result<(), String> {
        match line.get(..2) {
            Some("*Z") => {
                self.finish();
                self.current.number = num(col(line, 4, 9))?;
                self.current.admin = col(line, 11, 16).into();
                self.cycles = (
                    opt_num(col(line, 24, 26))?.unwrap_or(0),
                    opt_num(col(line, 28, 30))?.unwrap_or(0),
                );
            }
            Some("*G") if self.current.transport.is_empty() => {
                self.current.transport = col(line, 4, 6).into();
            }
            Some("*L") if self.current.line.is_none() => {
                let name = match col(line, 4, 11).strip_prefix('#') {
                    Some(number) => self
                        .line_names
                        .get(&num(number)?)
                        .cloned()
                        .unwrap_or_default(),
                    None => col(line, 4, 11).into(),
                };
                let next = self.lines.len() as u32;
                let id = *self.line_ids.entry(name.clone()).or_insert(next);
                if id == next {
                    self.lines.push(name);
                }
                self.current.line = Some(id);
            }
            Some("*R") if self.current.direction.is_none() => {
                self.current.direction = col(line, 4, 4).chars().next();
                self.current.towards = self.directions.get(col(line, 6, 12)).copied();
            }
            Some("*A") if col(line, 4, 5) == "VE" => self.sections.push(RawSection {
                from: opt_num(col(line, 7, 13))?,
                to: opt_num(col(line, 15, 21))?,
                bit_field: opt_num(col(line, 23, 28))?.unwrap_or(0),
                departure: time(col(line, 30, 35))?.map(|t| t.0),
                arrival: time(col(line, 37, 42))?.map(|t| t.0),
            }),
            Some(s) if s.starts_with('*') => {}
            None => {}
            _ => {
                // The stop name (columns 9-28) can contain multi-byte characters, so the times
                // are located by character position.
                let times = line.char_indices().nth(29).map_or("", |(i, _)| &line[i..]);
                let (arrival, departure) = (time(col(times, 1, 6))?, time(col(times, 8, 13))?);
                self.current.stops.push(StopTime {
                    stop_id: num(col(line, 1, 7))?,
                    arrival: arrival.map(|t| t.0),
                    departure: departure.map(|t| t.0),
                    can_alight: arrival.is_some_and(|t| t.1),
                    can_board: departure.is_some_and(|t| t.1),
                    platform: None,
                });
            }
        }
        Ok(())
    }

    fn finish(&mut self) {
        let mut journey = std::mem::take(&mut self.current);
        let raw_sections = std::mem::take(&mut self.sections);
        if journey.stops.is_empty() {
            return;
        }
        journey.sections = sections(&journey, &raw_sections);
        let (repetitions, interval) = self.cycles;
        let copies: Vec<_> = (1..=repetitions)
            .map(|k| {
                let mut copy = journey.clone();
                for stop in &mut copy.stops {
                    stop.arrival = stop.arrival.map(|t| t + k * interval);
                    stop.departure = stop.departure.map(|t| t + k * interval);
                }
                copy
            })
            .collect();
        self.journeys.push(journey);
        self.journeys.extend(copies);
    }
}

/// Resolves *A VE lines to stop index ranges. Stops that no section covers get every bit field
/// of the journey, and a journey without *A VE line runs every day.
fn sections(journey: &Journey, raw_sections: &[RawSection]) -> Vec<Section> {
    let stops = &journey.stops;
    let section = |first: usize, last: usize, bit_field| Section {
        first: first as u16,
        last: last as u16,
        bit_field,
    };
    if raw_sections.is_empty() {
        return vec![section(0, stops.len() - 1, 0)];
    }

    let mut result = Vec::new();
    let mut covered = vec![false; stops.len()];
    for raw in raw_sections {
        match section_range(stops, raw) {
            Some((first, last)) => {
                covered[first..=last].fill(true);
                result.push(section(first, last, raw.bit_field));
            }
            None => log::warn!(
                "Journey ({}, {}): *A VE section {:?} -> {:?} is not on the route",
                journey.number,
                journey.admin,
                raw.from,
                raw.to
            ),
        }
    }

    let mut bit_fields: Vec<_> = raw_sections.iter().map(|raw| raw.bit_field).collect();
    bit_fields.sort_unstable();
    bit_fields.dedup();
    let mut i = 0;
    while i < stops.len() {
        let first = i;
        while i < stops.len() && !covered[i] {
            i += 1;
        }
        if i > first {
            result.extend(bit_fields.iter().map(|&b| section(first, i - 1, b)));
        }
        i += 1;
    }
    result
}

/// Stop indexes (both included) of a section: from its from stop (at its departure time, if
/// given) to the first following call at its to stop (at its arrival time, if given).
fn section_range(stops: &[StopTime], raw: &RawSection) -> Option<(usize, usize)> {
    let first = match raw.from {
        None => 0,
        Some(id) => stops.iter().position(|s| {
            s.stop_id == id && raw.departure.is_none_or(|t| s.departure == Some(t))
        })?,
    };
    let last = match raw.to {
        None => stops.len() - 1,
        Some(id) => {
            // Without an arrival time, a section ending where it starts (a loop) ends at a later
            // call, not at its start.
            let from = first + usize::from(raw.arrival.is_none() && raw.from == Some(id));
            let last = from
                + stops.get(from..)?.iter().position(|s| {
                    s.stop_id == id && raw.arrival.is_none_or(|t| s.arrival == Some(t))
                })?;
            // Consecutive calls at the to stop with the same arrival time cannot be told apart;
            // the section ends at the last of them.
            let same_call = |s: &&StopTime| {
                s.stop_id == id && raw.arrival.is_some_and(|t| s.arrival == Some(t))
            };
            last + stops[last + 1..].iter().take_while(same_call).count()
        }
    };
    Some((first, last))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn stop_line(id: i32, name: &str, arrival: &str, departure: &str) -> String {
        format!("{id:07} {name:<20} {arrival:>6} {departure:>6}                 %")
    }

    fn parse(lines: &[String]) -> Vec<Journey> {
        let mut fplan = Fplan::default();
        lines.iter().for_each(|line| fplan.line(line).unwrap());
        fplan.finish();
        assert!(fplan.lines.iter().all(|l| l == "S3"));
        fplan.journeys
    }

    #[test]
    fn journey_with_sections_bans_late_times_and_cycles() {
        let journeys = parse(&[
            "*Z 000123 000011   000 002 030                             %".into(),
            "*G IR  8500010 8500030                                     %".into(),
            "*A VE 8500010 8500020 000001                               %".into(),
            "*A VE 8500020 8500030 000002                               %".into(),
            "*A R  8500010 8500030                                      %".into(),
            "*L S3       8500010 8500030                                %".into(),
            "*R H                                                       %".into(),
            stop_line(8500010, "Basel SBB", "", "02350"),
            stop_line(8500020, "Lötschberg-Basistunn", "-02410", "-02412"),
            stop_line(8500030, "Bern", "02500", ""),
        ]);

        assert_eq!(journeys.len(), 3);
        let j = &journeys[0];
        assert_eq!(
            (j.number, j.admin.as_str(), j.transport.as_str()),
            (123, "000011", "IR")
        );
        assert_eq!(
            j.stops[1],
            StopTime {
                stop_id: 8500020,
                arrival: Some(1450),
                departure: Some(1452),
                can_alight: false,
                can_board: false,
                platform: None,
            }
        );
        assert_eq!(
            (j.stops[0].departure, j.stops[0].can_board),
            (Some(1430), true)
        );
        assert_eq!(j.stops[2].arrival, Some(1500));
        let ranges: Vec<_> = j
            .sections
            .iter()
            .map(|s| (s.first, s.last, s.bit_field))
            .collect();
        assert_eq!(ranges, [(0, 1, 1), (1, 2, 2)]);
        assert_eq!(journeys[2].stops[0].departure, Some(1430 + 60));
        assert_eq!((j.line, j.direction), (Some(0), Some('H')));
    }

    #[test]
    fn loop_section_without_times_ends_at_the_later_call() {
        let journeys = parse(&[
            "*Z 000001 000011   000                                     %".into(),
            "*A VE 8500010 8500010 000007                               %".into(),
            stop_line(8500010, "A", "", "00800"),
            stop_line(8500020, "B", "00805", "00806"),
            stop_line(8500010, "A", "00810", "00811"),
            stop_line(8500030, "C", "00815", ""),
        ]);
        let ranges: Vec<_> = journeys[0]
            .sections
            .iter()
            .map(|s| (s.first, s.last, s.bit_field))
            .collect();
        // Stop 3 is not covered, so it gets every bit field of the journey.
        assert_eq!(ranges, [(0, 2, 7), (3, 3, 7)]);
    }

    #[test]
    fn bit_field_days_start_after_two_padding_bits() {
        let bits = BitField(vec![0b1110_0000]);
        assert!(bits.runs_on(0));
        assert!(!bits.runs_on(1));
        assert!(!bits.runs_on(100));
    }
}
