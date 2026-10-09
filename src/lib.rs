#![doc = include_str!("../README.md")]
mod load;
mod parse;

use std::num::NonZeroU32;

use chrono::NaiveDate;
use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};

pub use load::Loader;

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Zip(#[from] zip::result::ZipError),
    #[error(transparent)]
    Http(#[from] reqwest::Error),
    #[error(transparent)]
    Cache(#[from] postcard::Error),
    #[error("{file}:{line}: {msg}")]
    Parse {
        file: &'static str,
        line: usize,
        msg: String,
    },
}

/// The routing-relevant content of one HRDF timetable.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Timetable {
    /// First and last day of the timetable period (ECKDATEN).
    pub start: NaiveDate,
    pub end: NaiveDate,
    /// Service days by bit field id (BITFELD). Id 0 means every day.
    pub bit_fields: FxHashMap<i32, BitField>,
    pub stops: FxHashMap<i32, Stop>,
    pub journeys: Vec<Journey>,
    /// Walking transfers between two stops (METABHF), one direction each.
    pub footpaths: Vec<Footpath>,
    /// Journeys continuing as another journey without changing vehicle (DURCHBI).
    pub through_services: Vec<ThroughService>,
    /// Minimum transfer time when no other rule applies (UMSTEIGB 9999999): (IC to IC, any other).
    pub default_transfer_time: (i16, i16),
    /// Transfer times between two administrations (UMSTEIGV).
    pub admin_transfer_times: Vec<AdminTransferTime>,
    /// Transfer times between two journeys (UMSTEIGZ).
    pub journey_transfer_times: Vec<JourneyTransferTime>,
    /// Transfer times between two lines (UMSTEIGL).
    pub line_transfer_times: Vec<LineTransferTime>,
    /// Line names (LINIE or *L lines) and direction texts (RICHTUNG), referenced by journeys.
    pub lines: Vec<String>,
    pub directions: Vec<String>,
    /// Operator short names by administration (BETRIEB).
    pub operators: FxHashMap<String, String>,
    /// Platform (track, bus bay...) names by stop and platform number (GLEISE).
    pub platforms: FxHashMap<(i32, u32), String>,
    /// English names of the offer categories (ZUGART), e.g. "B" is "Bus".
    pub transport_names: FxHashMap<String, String>,
}

impl Timetable {
    /// Whether `bit_field` has service on `date`.
    pub fn runs_on(&self, bit_field: i32, date: NaiveDate) -> bool {
        let Ok(day) = usize::try_from((date - self.start).num_days()) else {
            return false;
        };
        date <= self.end
            && (bit_field == 0
                || self
                    .bit_fields
                    .get(&bit_field)
                    .is_some_and(|b| b.runs_on(day)))
    }
}

/// Service days as stored in BITFELD: bit `2 + n` is set when there is service `n` days after the
/// timetable start.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct BitField(pub Vec<u8>);

impl BitField {
    pub fn runs_on(&self, day: usize) -> bool {
        let bit = day + 2;
        self.0
            .get(bit / 8)
            .is_some_and(|b| b >> (7 - bit % 8) & 1 == 1)
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Stop {
    pub id: i32,
    pub name: String,
    pub lv95: Option<Coordinates>,
    pub wgs84: Option<Coordinates>,
    /// False when KMINFO forbids changing vehicles here (value 0).
    pub transfer_allowed: bool,
    /// Minimum transfer time at this stop (UMSTEIGB): (IC to IC, any other).
    pub transfer_time: Option<(i16, i16)>,
}

/// A journey and its validity. A journey is identified by its number and administration.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Journey {
    pub number: i32,
    pub admin: String,
    /// Offer category code of the first *G line, e.g. "IC", "S", "B".
    pub transport: String,
    /// Index into `Timetable::lines` of the first *L line.
    pub line: Option<u32>,
    /// 'H' (outbound) or 'R' (return) from the first *R line, and its direction text (index into
    /// `Timetable::directions`).
    pub direction: Option<char>,
    pub towards: Option<u32>,
    pub stops: Vec<StopTime>,
    /// The stops each bit field applies to (*A VE lines), so a journey can run further on some
    /// days than on others.
    pub sections: Vec<Section>,
}

/// Times are minutes from midnight of the journey's service day and may exceed 24h.
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct StopTime {
    pub stop_id: i32,
    pub arrival: Option<i16>,
    pub departure: Option<i16>,
    pub can_alight: bool,
    pub can_board: bool,
    /// The platform at the stop: a key of `Timetable::platforms` with `stop_id`.
    pub platform: Option<NonZeroU32>,
}

/// Stop indexes `first..=last` of a journey run on the days of `bit_field`.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Section {
    pub first: u16,
    pub last: u16,
    pub bit_field: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Footpath {
    pub from: i32,
    pub to: i32,
    pub minutes: i16,
}

/// Journey numbers are paired with their administration.
#[derive(Debug, Serialize, Deserialize)]
pub struct ThroughService {
    pub from: (i32, String),
    pub to: (i32, String),
    pub stop_id: i32,
    pub bit_field: i32,
}

/// `stop_id` None means the rule applies at every stop.
#[derive(Debug, Serialize, Deserialize)]
pub struct AdminTransferTime {
    pub stop_id: Option<i32>,
    pub from: String,
    pub to: String,
    pub minutes: i16,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JourneyTransferTime {
    pub stop_id: i32,
    pub from: (i32, String),
    pub to: (i32, String),
    pub minutes: i16,
    pub bit_field: i32,
}

/// `stop_id` None means the rule applies at every stop.
#[derive(Debug, Serialize, Deserialize)]
pub struct LineTransferTime {
    pub stop_id: Option<i32>,
    pub from: LineSelector,
    pub to: LineSelector,
    pub minutes: i16,
}

/// The journeys of a line; None ("*" in the file) matches anything.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct LineSelector {
    pub admin: Option<String>,
    pub transport: Option<String>,
    pub line: Option<String>,
    pub direction: Option<char>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub enum CoordinateSystem {
    #[default]
    LV95,
    WGS84,
}

/// LV95 coordinates are (easting, northing), WGS84 ones (latitude, longitude).
#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize)]
pub struct Coordinates {
    coordinate_system: CoordinateSystem,
    x: f64,
    y: f64,
}

impl Coordinates {
    pub fn new(coordinate_system: CoordinateSystem, x: f64, y: f64) -> Self {
        Self {
            coordinate_system,
            x,
            y,
        }
    }

    pub fn easting(&self) -> Option<f64> {
        (self.coordinate_system == CoordinateSystem::LV95).then_some(self.x)
    }

    pub fn northing(&self) -> Option<f64> {
        (self.coordinate_system == CoordinateSystem::LV95).then_some(self.y)
    }

    pub fn latitude(&self) -> Option<f64> {
        (self.coordinate_system == CoordinateSystem::WGS84).then_some(self.x)
    }

    pub fn longitude(&self) -> Option<f64> {
        (self.coordinate_system == CoordinateSystem::WGS84).then_some(self.y)
    }
}
