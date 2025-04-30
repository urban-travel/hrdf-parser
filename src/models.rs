use std::{
    collections::BTreeSet,
    hash::{DefaultHasher, Hash, Hasher},
};

use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};
use strum_macros::{self, Display, EnumString};

use crate::{
    storage::DataStorage,
    utils::{add_1_day, sub_1_day},
};

// ------------------------------------------------------------------------------------------------
// --- Model
// ------------------------------------------------------------------------------------------------

pub trait Model<M: Model<M>> {
    // Primary key type.
    type K: Copy + Eq + Hash + Serialize + for<'a> Deserialize<'a>;

    fn id(&self) -> M::K;

    fn vec_to_map(data: Vec<M>) -> FxHashMap<M::K, M> {
        data.into_iter()
            .fold(FxHashMap::default(), |mut acc, item| {
                acc.insert(item.id(), item);
                acc
            })
    }
}

macro_rules! impl_Model {
    ($m:ty) => {
        impl Model<$m> for $m {
            type K = i32;

            fn id(&self) -> Self::K {
                self.id
            }
        }
    };
}

// ------------------------------------------------------------------------------------------------
// --- Attribute
// ------------------------------------------------------------------------------------------------

#[derive(Debug, Serialize, Deserialize)]
pub struct Attribute {
    id: i32,
    designation: String,
    stop_scope: i16,
    main_sorting_priority: i16,
    secondary_sorting_priority: i16,
    description: FxHashMap<Language, String>,
}

impl_Model!(Attribute);

impl Attribute {
    pub fn new(
        id: i32,
        designation: String,
        stop_scope: i16,
        main_sorting_priority: i16,
        secondary_sorting_priority: i16,
    ) -> Self {
        Self {
            id,
            designation,
            stop_scope,
            main_sorting_priority,
            secondary_sorting_priority,
            description: FxHashMap::default(),
        }
    }

    // Getters/Setters

    pub fn set_description(&mut self, language: Language, value: &str) {
        self.description.insert(language, value.to_string());
    }
}

// ------------------------------------------------------------------------------------------------
// --- BitField
// ------------------------------------------------------------------------------------------------

#[derive(Debug, Serialize, Deserialize)]
pub struct BitField {
    id: i32,
    bits: Vec<u8>,
}

impl_Model!(BitField);

impl BitField {
    pub fn new(id: i32, bits: Vec<u8>) -> Self {
        Self { id, bits }
    }

    // Getters/Setters

    pub fn bits(&self) -> &Vec<u8> {
        &self.bits
    }
}

// ------------------------------------------------------------------------------------------------
// --- Color
// ------------------------------------------------------------------------------------------------

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Color {
    r: i16,
    g: i16,
    b: i16,
}

#[allow(unused)]
impl Color {
    pub fn new(r: i16, g: i16, b: i16) -> Self {
        Self { r, g, b }
    }

    // Getters/Setters

    pub fn r(&self) -> i16 {
        self.r
    }

    pub fn g(&self) -> i16 {
        self.g
    }

    pub fn b(&self) -> i16 {
        self.b
    }
}

// ------------------------------------------------------------------------------------------------
// --- CoordinateSystem
// ------------------------------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, Default, Display, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum CoordinateSystem {
    #[default]
    LV95,
    WGS84,
}

// ------------------------------------------------------------------------------------------------
// --- Coordinates
// ------------------------------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize)]
pub struct Coordinates {
    coordinate_system: CoordinateSystem,
    x: f64,
    y: f64,
}

#[allow(unused)]
impl Coordinates {
    pub fn new(coordinate_system: CoordinateSystem, x: f64, y: f64) -> Self {
        Self {
            coordinate_system,
            x,
            y,
        }
    }

    // Getters/Setters

    pub fn easting(&self) -> Option<f64> {
        match self.coordinate_system {
            CoordinateSystem::LV95 => Some(self.x),
            CoordinateSystem::WGS84 => None,
        }
    }

    pub fn northing(&self) -> Option<f64> {
        match self.coordinate_system {
            CoordinateSystem::LV95 => Some(self.y),
            CoordinateSystem::WGS84 => None,
        }
    }

    pub fn latitude(&self) -> Option<f64> {
        match self.coordinate_system {
            CoordinateSystem::WGS84 => Some(self.x),
            CoordinateSystem::LV95 => None,
        }
    }

    pub fn longitude(&self) -> Option<f64> {
        match self.coordinate_system {
            CoordinateSystem::WGS84 => Some(self.y),
            CoordinateSystem::LV95 => None,
        }
    }
}

// ------------------------------------------------------------------------------------------------
// --- Direction
// ------------------------------------------------------------------------------------------------

#[derive(Debug, Serialize, Deserialize)]
pub struct Direction {
    id: i32,
    name: String,
}

impl_Model!(Direction);

impl Direction {
    pub fn new(id: i32, name: String) -> Self {
        Self { id, name }
    }
}

// ------------------------------------------------------------------------------------------------
// --- DirectionType
// ------------------------------------------------------------------------------------------------

#[derive(
    Clone, Copy, Debug, Default, Display, Eq, Hash, PartialEq, EnumString, Serialize, Deserialize,
)]
pub enum DirectionType {
    #[default]
    #[strum(serialize = "R")]
    Outbound,

    #[strum(serialize = "H")]
    Return,
}

// ------------------------------------------------------------------------------------------------
// --- Holiday
// ------------------------------------------------------------------------------------------------

#[derive(Debug, Serialize, Deserialize)]
pub struct Holiday {
    id: i32,
    date: NaiveDate,
    name: FxHashMap<Language, String>,
}

impl_Model!(Holiday);

impl Holiday {
    pub fn new(id: i32, date: NaiveDate, name: FxHashMap<Language, String>) -> Self {
        Self { id, date, name }
    }
}

// ------------------------------------------------------------------------------------------------
// --- ExchangeTimeAdministration
// ------------------------------------------------------------------------------------------------

#[derive(Debug, Serialize, Deserialize)]
pub struct ExchangeTimeAdministration {
    id: i32,
    stop_id: Option<i32>, // A None value means that the exchange time applies to all stops if there is no specific entry for the stop and the 2 administrations.
    administration_1: String,
    administration_2: String,
    duration: i16, // Exchange time from administration 1 to administration 2 is in minutes.
}

impl_Model!(ExchangeTimeAdministration);

impl ExchangeTimeAdministration {
    pub fn new(
        id: i32,
        stop_id: Option<i32>,
        administration_1: String,
        administration_2: String,
        duration: i16,
    ) -> Self {
        Self {
            id,
            stop_id,
            administration_1,
            administration_2,
            duration,
        }
    }

    // Getters/Setters

    pub fn stop_id(&self) -> Option<i32> {
        self.stop_id
    }

    pub fn administration_1(&self) -> &str {
        &self.administration_1
    }

    pub fn administration_2(&self) -> &str {
        &self.administration_2
    }

    pub fn duration(&self) -> i16 {
        self.duration
    }
}

// ------------------------------------------------------------------------------------------------
// --- ExchangeTimeJourney
// ------------------------------------------------------------------------------------------------

#[derive(Debug, Serialize, Deserialize)]
pub struct ExchangeTimeJourney {
    id: i32,
    stop_id: i32,
    journey_id_1: i32,
    journey_id_2: i32,
    duration: i16, // Exchange time from journey 1 to journey 2 is in minutes.
    is_guaranteed: bool,
    bit_field_id: Option<i32>,
}

impl_Model!(ExchangeTimeJourney);

impl ExchangeTimeJourney {
    pub fn new(
        id: i32,
        stop_id: i32,
        journey_id_1: i32,
        journey_id_2: i32,
        duration: i16,
        is_guaranteed: bool,
        bit_field_id: Option<i32>,
    ) -> Self {
        Self {
            id,
            stop_id,
            journey_id_1,
            journey_id_2,
            duration,
            is_guaranteed,
            bit_field_id,
        }
    }

    // Getters/Setters

    pub fn stop_id(&self) -> i32 {
        self.stop_id
    }

    pub fn journey_id_1(&self) -> i32 {
        self.journey_id_1
    }

    pub fn journey_id_2(&self) -> i32 {
        self.journey_id_2
    }

    pub fn duration(&self) -> i16 {
        self.duration
    }

    pub fn bit_field_id(&self) -> Option<i32> {
        self.bit_field_id
    }
}

// ------------------------------------------------------------------------------------------------
// --- ExchangeTimeLine
// ------------------------------------------------------------------------------------------------

#[derive(Debug, Serialize, Deserialize)]
pub struct ExchangeTimeLine {
    id: i32,
    stop_id: Option<i32>,
    line_1: LineInfo,
    line_2: LineInfo,
    duration: i16, // Exchange time from line 1 to line 2 is in minutes.
    is_guaranteed: bool,
}

impl_Model!(ExchangeTimeLine);

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct LineInfo {
    administration: String,
    transport_type_id: i32,
    line_id: Option<String>,
    direction: Option<DirectionType>,
}

impl LineInfo {
    pub(crate) fn new(
        administration: String,
        transport_type_id: i32,
        line_id: Option<String>,
        direction: Option<DirectionType>,
    ) -> Self {
        Self {
            administration,
            transport_type_id,
            line_id,
            direction,
        }
    }
}

impl ExchangeTimeLine {
    pub(crate) fn new(
        id: i32,
        stop_id: Option<i32>,
        line_1: LineInfo,
        line_2: LineInfo,
        duration: i16,
        is_guaranteed: bool,
    ) -> Self {
        Self {
            id,
            stop_id,
            line_1,
            line_2,
            duration,
            is_guaranteed,
        }
    }
}

// ------------------------------------------------------------------------------------------------
// --- InformationText
// ------------------------------------------------------------------------------------------------

#[derive(Debug, Serialize, Deserialize)]
pub struct InformationText {
    id: i32,
    content: FxHashMap<Language, String>,
}

impl_Model!(InformationText);

impl InformationText {
    pub fn new(id: i32) -> Self {
        Self {
            id,
            content: FxHashMap::default(),
        }
    }

    // Getters/Setters

    pub fn set_content(&mut self, language: Language, value: &str) {
        self.content.insert(language, value.to_string());
    }
}

// ------------------------------------------------------------------------------------------------
// --- Journey
// ------------------------------------------------------------------------------------------------

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Journey {
    id: i32,
    legacy_id: i32,
    administration: String,
    metadata: FxHashMap<JourneyMetadataType, Vec<JourneyMetadataEntry>>,
    route: Vec<JourneyRouteEntry>,
}

impl_Model!(Journey);

impl Journey {
    pub fn new(id: i32, legacy_id: i32, administration: String) -> Self {
        Self {
            id,
            legacy_id,
            administration,
            metadata: FxHashMap::default(),
            route: Vec::new(),
        }
    }

    // Getters/Setters

    pub fn administration(&self) -> &str {
        &self.administration
    }

    fn metadata(&self) -> &FxHashMap<JourneyMetadataType, Vec<JourneyMetadataEntry>> {
        &self.metadata
    }

    pub fn route(&self) -> &Vec<JourneyRouteEntry> {
        &self.route
    }

    // Functions

    pub fn add_metadata_entry(&mut self, k: JourneyMetadataType, v: JourneyMetadataEntry) {
        self.metadata.entry(k).or_default().push(v);
    }

    pub fn add_route_entry(&mut self, entry: JourneyRouteEntry) {
        self.route.push(entry);
    }

    pub fn bit_field_id(&self) -> Option<i32> {
        // unwrap: There will always be a BitField entry.
        let entry = &self.metadata().get(&JourneyMetadataType::BitField).unwrap()[0];
        entry.bit_field_id
    }

    pub fn transport_type_id(&self) -> i32 {
        // unwrap: There will always be a TransportType entry.
        let entry = &self
            .metadata()
            .get(&JourneyMetadataType::TransportType)
            .unwrap()[0];
        // unwrap: It's guaranteed to have value here.
        entry.resource_id.unwrap()
    }

    pub fn transport_type<'a>(&'a self, data_storage: &'a DataStorage) -> &'a TransportType {
        data_storage
            .transport_types()
            .find(self.transport_type_id())
            .unwrap_or_else(|| panic!("Transport type {:?} not found.", self.transport_type_id()))
    }

    pub fn first_stop_id(&self) -> i32 {
        // unwrap: The route always contains at least 2 entries.
        self.route.first().unwrap().stop_id()
    }

    pub fn last_stop_id(&self) -> i32 {
        // unwrap: The route always contains at least 2 entries.
        self.route.last().unwrap().stop_id()
    }

    pub fn is_last_stop(&self, stop_id: i32, ignore_loop: bool) -> bool {
        if ignore_loop && self.first_stop_id() == self.last_stop_id() {
            false
        } else {
            stop_id == self.last_stop_id()
        }
    }

    pub fn count_stops(&self, departure_stop_id: i32, arrival_stop_id: i32) -> usize {
        self.route()
            .iter()
            .skip_while(|stop| stop.stop_id() != departure_stop_id)
            .take_while(|stop| stop.stop_id() != arrival_stop_id)
            .count()
            + 1
    }

    pub fn hash_route(&self, departure_stop_id: i32) -> Option<u64> {
        let index = self
            .route
            .iter()
            .position(|route_entry| route_entry.stop_id() == departure_stop_id)?;

        let mut hasher = DefaultHasher::new();
        self.route
            .iter()
            .skip(index)
            .map(|route_entry| route_entry.stop_id())
            .collect::<BTreeSet<_>>()
            .hash(&mut hasher);
        Some(hasher.finish())
    }

    /// unwrap: Do not call this function if the stop is not part of the route.
    /// unwrap: Do not call this function if the stop has no departure time (only the last stop has no departure time).
    pub fn departure_time_of(&self, stop_id: i32) -> (NaiveTime, bool) {
        let route = self.route();
        let index = route
            .iter()
            .position(|route_entry| route_entry.stop_id() == stop_id)
            .unwrap();
        let departure_time = route[index].departure_time().unwrap();

        (
            departure_time,
            // The departure time is on the next day if this evaluates to true.
            departure_time < route.first().unwrap().departure_time().unwrap(),
        )
    }

    /// The date must correspond to the route's first entry.
    /// Do not call this function if the stop is not part of the route.
    /// Do not call this function if the stop has no departure time (only the last stop has no departure time).
    pub fn departure_at_of(&self, stop_id: i32, date: NaiveDate) -> NaiveDateTime {
        match self.departure_time_of(stop_id) {
            (departure_time, false) => NaiveDateTime::new(date, departure_time),
            (departure_time, true) => NaiveDateTime::new(add_1_day(date), departure_time),
        }
    }

    /// The date must be associated with the origin_stop_id.
    /// Do not call this function if the stop is not part of the route.
    pub fn departure_at_of_with_origin(
        &self,
        stop_id: i32,
        date: NaiveDate,
        // If it's not a departure date, it's an arrival date.
        is_departure_date: bool,
        origin_stop_id: i32,
    ) -> NaiveDateTime {
        let (departure_time, is_next_day) = self.departure_time_of(stop_id);
        let (_, origin_is_next_day) = if is_departure_date {
            self.departure_time_of(origin_stop_id)
        } else {
            self.arrival_time_of(origin_stop_id)
        };

        match (is_next_day, origin_is_next_day) {
            (true, false) => NaiveDateTime::new(add_1_day(date), departure_time),
            (false, true) => NaiveDateTime::new(sub_1_day(date), departure_time),
            _ => NaiveDateTime::new(date, departure_time),
        }
    }

    /// unwrap: Do not call this function if the stop is not part of the route.
    /// unwrap: Do not call this function if the stop has no arrival time (only the first stop has no arrival time).
    pub fn arrival_time_of(&self, stop_id: i32) -> (NaiveTime, bool) {
        let route = self.route();
        let index = route
            .iter()
            // The first route entry has no arrival time.
            .skip(1)
            .position(|route_entry| route_entry.stop_id() == stop_id)
            .map(|i| i + 1)
            .unwrap();
        let arrival_time = route[index].arrival_time().unwrap();

        (
            arrival_time,
            // The arrival time is on the next day if this evaluates to true.
            arrival_time < route.first().unwrap().departure_time().unwrap(),
        )
    }

    /// The date must be associated with the origin_stop_id.
    pub fn arrival_at_of_with_origin(
        &self,
        stop_id: i32,
        date: NaiveDate,
        // If it's not a departure date, it's an arrival date.
        is_departure_date: bool,
        origin_stop_id: i32,
    ) -> NaiveDateTime {
        let (arrival_time, is_next_day) = self.arrival_time_of(stop_id);
        let (_, origin_is_next_day) = if is_departure_date {
            self.departure_time_of(origin_stop_id)
        } else {
            self.arrival_time_of(origin_stop_id)
        };

        match (is_next_day, origin_is_next_day) {
            (true, false) => NaiveDateTime::new(add_1_day(date), arrival_time),
            (false, true) => NaiveDateTime::new(sub_1_day(date), arrival_time),
            _ => NaiveDateTime::new(date, arrival_time),
        }
    }

    /// Excluding departure stop.
    pub fn route_section(
        &self,
        departure_stop_id: i32,
        arrival_stop_id: i32,
    ) -> Vec<&JourneyRouteEntry> {
        let mut route_iter = self.route().iter();

        for route_entry in route_iter.by_ref() {
            if route_entry.stop_id() == departure_stop_id {
                break;
            }
        }

        let mut result = Vec::new();

        for route_entry in route_iter {
            result.push(route_entry);

            if route_entry.stop_id() == arrival_stop_id {
                break;
            }
        }

        result
    }
}

// ------------------------------------------------------------------------------------------------
// --- JourneyMetadataType
// ------------------------------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, Default, Display, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum JourneyMetadataType {
    #[default]
    Attribute,
    BitField,
    Direction,
    InformationText,
    Line,
    ExchangeTimeBoarding,
    ExchangeTimeDisembarking,
    TransportType,
}

// ------------------------------------------------------------------------------------------------
// --- JourneyMetadataEntry
// ------------------------------------------------------------------------------------------------

#[derive(Debug, Serialize, Deserialize)]
pub struct JourneyMetadataEntry {
    from_stop_id: Option<i32>,
    until_stop_id: Option<i32>,
    resource_id: Option<i32>,
    bit_field_id: Option<i32>,
    departure_time: Option<NaiveTime>,
    arrival_time: Option<NaiveTime>,
    extra_field_1: Option<String>,
    extra_field_2: Option<i32>,
}

impl JourneyMetadataEntry {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        from_stop_id: Option<i32>,
        until_stop_id: Option<i32>,
        resource_id: Option<i32>,
        bit_field_id: Option<i32>,
        departure_time: Option<NaiveTime>,
        arrival_time: Option<NaiveTime>,
        extra_field_1: Option<String>,
        extra_field_2: Option<i32>,
    ) -> Self {
        Self {
            from_stop_id,
            until_stop_id,
            resource_id,
            bit_field_id,
            departure_time,
            arrival_time,
            extra_field_1,
            extra_field_2,
        }
    }
}

// ------------------------------------------------------------------------------------------------
// --- JourneyRouteEntry
// ------------------------------------------------------------------------------------------------

#[derive(Debug, Serialize, Deserialize)]
pub struct JourneyRouteEntry {
    stop_id: i32,
    arrival_time: Option<NaiveTime>,
    departure_time: Option<NaiveTime>,
}

impl JourneyRouteEntry {
    pub fn new(
        stop_id: i32,
        arrival_time: Option<NaiveTime>,
        departure_time: Option<NaiveTime>,
    ) -> Self {
        Self {
            stop_id,
            arrival_time,
            departure_time,
        }
    }

    // Getters/Setters

    pub fn stop_id(&self) -> i32 {
        self.stop_id
    }

    pub fn arrival_time(&self) -> &Option<NaiveTime> {
        &self.arrival_time
    }

    pub fn departure_time(&self) -> &Option<NaiveTime> {
        &self.departure_time
    }

    // Functions

    pub fn stop<'a>(&'a self, data_storage: &'a DataStorage) -> &'a Stop {
        data_storage
            .stops()
            .find(self.stop_id())
            .unwrap_or_else(|| panic!("Stop {:?} not found.", self.stop_id()))
    }
}

// ------------------------------------------------------------------------------------------------
// --- JourneyPlatform
// ------------------------------------------------------------------------------------------------

#[derive(Debug, Serialize, Deserialize)]
pub struct JourneyPlatform {
    journey_id: i32,
    platform_id: i32,
    time: Option<NaiveTime>,
    bit_field_id: Option<i32>,
}

impl JourneyPlatform {
    pub fn new(
        journey_id: i32,
        platform_id: i32,
        time: Option<NaiveTime>,
        bit_field_id: Option<i32>,
    ) -> Self {
        Self {
            journey_id,
            platform_id,
            time,
            bit_field_id,
        }
    }
}

impl Model<JourneyPlatform> for JourneyPlatform {
    type K = (i32, i32);

    fn id(&self) -> Self::K {
        (self.journey_id, self.platform_id)
    }
}

// ------------------------------------------------------------------------------------------------
// --- Language
// ------------------------------------------------------------------------------------------------

#[derive(
    Clone, Copy, Debug, Default, Display, Eq, Hash, PartialEq, EnumString, Serialize, Deserialize,
)]
pub enum Language {
    #[default]
    #[strum(serialize = "deu")]
    German,

    #[strum(serialize = "fra")]
    French,

    #[strum(serialize = "ita")]
    Italian,

    #[strum(serialize = "eng")]
    English,
}

// ------------------------------------------------------------------------------------------------
// --- Line
// ------------------------------------------------------------------------------------------------

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Line {
    id: i32,
    name: String,
    short_name: String,
    long_name: String,
    text_color: Color,
    background_color: Color,
}

impl_Model!(Line);

impl Line {
    pub fn new(id: i32, name: String) -> Self {
        Self {
            id,
            name,
            short_name: String::default(),
            long_name: String::default(),
            text_color: Color::default(),
            background_color: Color::default(),
        }
    }

    // Getters/Setters

    pub fn set_short_name(&mut self, value: String) {
        self.short_name = value;
    }

    pub fn set_long_name(&mut self, value: String) {
        self.long_name = value;
    }

    pub fn set_text_color(&mut self, value: Color) {
        self.text_color = value;
    }

    pub fn set_background_color(&mut self, value: Color) {
        self.background_color = value;
    }
}

// ------------------------------------------------------------------------------------------------
// --- Platform
// ------------------------------------------------------------------------------------------------

#[derive(Debug, Serialize, Deserialize)]
pub struct Platform {
    id: i32,
    name: String,
    sectors: Option<String>,
    stop_id: i32,
    sloid: String,
    lv95_coordinates: Coordinates,
    wgs84_coordinates: Coordinates,
}

impl_Model!(Platform);

impl Platform {
    pub fn new(id: i32, name: String, sectors: Option<String>, stop_id: i32) -> Self {
        Self {
            id,
            name,
            sectors,
            stop_id,
            sloid: String::default(),
            lv95_coordinates: Coordinates::default(),
            wgs84_coordinates: Coordinates::default(),
        }
    }

    // Getters/Setters

    pub fn set_sloid(&mut self, value: String) {
        self.sloid = value;
    }

    pub fn set_lv95_coordinates(&mut self, value: Coordinates) {
        self.lv95_coordinates = value;
    }

    pub fn set_wgs84_coordinates(&mut self, value: Coordinates) {
        self.wgs84_coordinates = value;
    }
}

// ------------------------------------------------------------------------------------------------
// --- Stop
// ------------------------------------------------------------------------------------------------

#[derive(Debug, Serialize, Deserialize)]
pub struct Stop {
    id: i32,
    name: String,
    long_name: Option<String>,
    abbreviation: Option<String>,
    synonyms: Option<Vec<String>>,
    lv95_coordinates: Option<Coordinates>,
    wgs84_coordinates: Option<Coordinates>,
    exchange_priority: i16,
    exchange_flag: i16,
    exchange_time: Option<(i16, i16)>, // (InterCity exchange time, Exchange time for all other journey types)
    restrictions: i16,
    sloid: String,
    boarding_areas: Vec<String>,
}

impl_Model!(Stop);

impl Stop {
    pub fn new(
        id: i32,
        name: String,
        long_name: Option<String>,
        abbreviation: Option<String>,
        synonyms: Option<Vec<String>>,
    ) -> Self {
        Self {
            id,
            name,
            long_name,
            abbreviation,
            synonyms,
            lv95_coordinates: None,
            wgs84_coordinates: None,
            exchange_priority: 8, // 8 is the default priority.
            exchange_flag: 0,
            exchange_time: None,
            restrictions: 0,
            sloid: String::default(),
            boarding_areas: Vec::new(),
        }
    }

    // Getters/Setters

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn lv95_coordinates(&self) -> Option<Coordinates> {
        self.lv95_coordinates
    }

    pub fn set_lv95_coordinates(&mut self, value: Coordinates) {
        self.lv95_coordinates = Some(value);
    }

    pub fn wgs84_coordinates(&self) -> Option<Coordinates> {
        self.wgs84_coordinates
    }

    pub fn set_wgs84_coordinates(&mut self, value: Coordinates) {
        self.wgs84_coordinates = Some(value);
    }

    pub fn set_exchange_priority(&mut self, value: i16) {
        self.exchange_priority = value;
    }

    pub fn exchange_flag(&self) -> i16 {
        self.exchange_flag
    }

    pub fn set_exchange_flag(&mut self, value: i16) {
        self.exchange_flag = value;
    }

    pub fn exchange_time(&self) -> Option<(i16, i16)> {
        self.exchange_time
    }

    pub fn set_exchange_time(&mut self, value: Option<(i16, i16)>) {
        self.exchange_time = value;
    }

    pub fn set_restrictions(&mut self, value: i16) {
        self.restrictions = value;
    }

    pub fn set_sloid(&mut self, value: String) {
        self.sloid = value;
    }

    // Functions

    pub fn add_boarding_area(&mut self, value: String) {
        self.boarding_areas.push(value);
    }

    pub fn can_be_used_as_exchange_point(&self) -> bool {
        self.exchange_flag() != 0
    }
}

// ------------------------------------------------------------------------------------------------
// --- StopConnection
// ------------------------------------------------------------------------------------------------

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct StopConnection {
    id: i32,
    stop_id_1: i32,
    stop_id_2: i32,
    duration: i16, // Exchange time from stop 1 to stop 2 is in minutes.
    attribute: i32,
}

impl_Model!(StopConnection);

impl StopConnection {
    pub fn new(id: i32, stop_id_1: i32, stop_id_2: i32, duration: i16) -> Self {
        Self {
            id,
            stop_id_1,
            stop_id_2,
            duration,
            attribute: 0,
        }
    }

    // Getters/Setters

    pub fn stop_id_1(&self) -> i32 {
        self.stop_id_1
    }

    pub fn stop_id_2(&self) -> i32 {
        self.stop_id_2
    }

    pub fn duration(&self) -> i16 {
        self.duration
    }

    pub fn set_attribute(&mut self, value: i32) {
        self.attribute = value;
    }
}

// ------------------------------------------------------------------------------------------------
// --- ThroughService
// ------------------------------------------------------------------------------------------------

#[derive(Debug, Serialize, Deserialize)]
pub struct ThroughService {
    id: i32,
    journey_1_id: i32,
    journey_1_stop_id: i32, // Last stop of journey 1.
    journey_2_id: i32,
    journey_2_stop_id: Option<i32>, // First stop of journey 2.
    bit_field_id: i32,
}

impl_Model!(ThroughService);

impl ThroughService {
    pub fn new(
        id: i32,
        journey_1_id: i32,
        journey_1_stop_id: i32,
        journey_2_id: i32,
        journey_2_stop_id: Option<i32>,
        bit_field_id: i32,
    ) -> Self {
        Self {
            id,
            journey_1_id,
            journey_1_stop_id,
            journey_2_id,
            journey_2_stop_id,
            bit_field_id,
        }
    }
}

// ------------------------------------------------------------------------------------------------
// --- TimetableMetadataEntry
// ------------------------------------------------------------------------------------------------

#[derive(Debug, Serialize, Deserialize)]
pub struct TimetableMetadataEntry {
    id: i32,
    key: String,
    value: String,
}

impl_Model!(TimetableMetadataEntry);

impl TimetableMetadataEntry {
    pub fn new(id: i32, key: String, value: String) -> Self {
        Self { id, key, value }
    }

    // Getters/Setters

    pub fn key(&self) -> &str {
        &self.key
    }

    pub fn value(&self) -> &str {
        &self.value
    }

    /// unwrap: Do not call this function if the value is not a date.
    #[allow(non_snake_case)]
    pub fn value_as_NaiveDate(&self) -> NaiveDate {
        NaiveDate::parse_from_str(self.value(), "%Y-%m-%d").unwrap()
    }
}

// ------------------------------------------------------------------------------------------------
// --- TransportCompany
// ------------------------------------------------------------------------------------------------

#[derive(Debug, Serialize, Deserialize)]
pub struct TransportCompany {
    id: i32,
    short_name: FxHashMap<Language, String>,
    long_name: FxHashMap<Language, String>,
    full_name: FxHashMap<Language, String>,
    administrations: Vec<String>,
}

impl_Model!(TransportCompany);

impl TransportCompany {
    pub fn new(id: i32, administrations: Vec<String>) -> Self {
        Self {
            id,
            short_name: FxHashMap::default(),
            long_name: FxHashMap::default(),
            full_name: FxHashMap::default(),
            administrations,
        }
    }

    // Getters/Setters

    pub fn set_short_name(&mut self, language: Language, value: &str) {
        self.short_name.insert(language, value.to_string());
    }

    pub fn set_long_name(&mut self, language: Language, value: &str) {
        self.long_name.insert(language, value.to_string());
    }

    pub fn set_full_name(&mut self, language: Language, value: &str) {
        self.full_name.insert(language, value.to_string());
    }
}

// ------------------------------------------------------------------------------------------------
// --- TransportType
// ------------------------------------------------------------------------------------------------

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct TransportType {
    id: i32,
    designation: String,
    product_class_id: i16,
    tarrif_group: String,
    output_control: i16,
    short_name: String,
    surchage: i16,
    flag: String,
    product_class_name: FxHashMap<Language, String>,
    category_name: FxHashMap<Language, String>,
}

impl_Model!(TransportType);

impl TransportType {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: i32,
        designation: String,
        product_class_id: i16,
        tarrif_group: String,
        output_control: i16,
        short_name: String,
        surchage: i16,
        flag: String,
    ) -> Self {
        Self {
            id,
            designation,
            product_class_id,
            tarrif_group,
            output_control,
            short_name,
            surchage,
            flag,
            product_class_name: FxHashMap::default(),
            category_name: FxHashMap::default(),
        }
    }

    // Getters/Setters

    pub fn designation(&self) -> &str {
        &self.designation
    }

    pub fn product_class_id(&self) -> i16 {
        self.product_class_id
    }

    pub fn set_product_class_name(&mut self, language: Language, value: &str) {
        self.product_class_name.insert(language, value.to_string());
    }

    pub fn set_category_name(&mut self, language: Language, value: &str) {
        self.category_name.insert(language, value.to_string());
    }
}

// ------------------------------------------------------------------------------------------------
// --- Version
// ------------------------------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, Display, Eq, Hash, PartialEq, Serialize, Deserialize)]
#[allow(non_camel_case_types)]
pub enum Version {
    V_5_40_41_2_0_4,
    V_5_40_41_2_0_5,
    V_5_40_41_2_0_6,
    V_5_40_41_2_0_7,
}
