use std::error::Error;

use chrono::{Days, NaiveDate};
use rustc_hash::{FxHashMap, FxHashSet};
use serde::{Deserialize, Serialize};

use crate::{
    models::{
        Attribute, BitField, Direction, ExchangeTimeAdministration, ExchangeTimeTrip,
        ExchangeTimeLine, Holiday, InformationText, Trip, TripPlatform, Line, Model,
        Platform, Stop, StopConnection, ThroughService, TimetableMetadataEntry, TransportCompany,
        TransportType, Version,
    },
    parsing,
    utils::{count_days_between_two_dates, timetable_end_date, timetable_start_date},
};

// ------------------------------------------------------------------------------------------------
// --- DataStorage
// ------------------------------------------------------------------------------------------------

#[derive(Debug, Serialize, Deserialize)]
pub struct DataStorage {
    // Time-relevant data.
    bit_fields: ResourceStorage<BitField>,
    holidays: ResourceStorage<Holiday>,
    timetable_metadata: ResourceStorage<TimetableMetadataEntry>,

    // Basic data.
    attributes: ResourceStorage<Attribute>,
    information_texts: ResourceStorage<InformationText>,
    directions: ResourceStorage<Direction>,
    lines: ResourceStorage<Line>,
    transport_companies: ResourceStorage<TransportCompany>,
    transport_types: ResourceStorage<TransportType>,

    // Stop data
    stops: ResourceStorage<Stop>,
    stop_connections: ResourceStorage<StopConnection>,

    // Timetable data
    trips: ResourceStorage<Trip>,
    trip_platform: ResourceStorage<TripPlatform>,
    platforms: ResourceStorage<Platform>,
    through_service: ResourceStorage<ThroughService>,

    // Exchange times
    exchange_times_administration: ResourceStorage<ExchangeTimeAdministration>,
    exchange_times_trip: ResourceStorage<ExchangeTimeTrip>,
    exchange_times_line: ResourceStorage<ExchangeTimeLine>,

    // Maps
    bit_fields_by_day: FxHashMap<NaiveDate, FxHashSet<i32>>,
    bit_fields_by_stop_id: FxHashMap<i32, FxHashSet<i32>>,
    trips_by_stop_id_and_bit_field_id: FxHashMap<(i32, i32), Vec<i32>>,
    stop_connections_by_stop_id: FxHashMap<i32, FxHashSet<i32>>,
    exchange_times_administration_map: FxHashMap<(Option<i32>, String, String), i32>,
    exchange_times_trip_map: FxHashMap<(i32, i32, i32), FxHashSet<i32>>,

    // Additional global data
    default_exchange_time: (i16, i16), // (InterCity exchange time, Exchange time for all other trip types)
}

#[allow(unused)]
impl DataStorage {
    pub fn new(version: Version, path: &str) -> Result<Self, Box<dyn Error>> {
        // Time-relevant data
        let bit_fields = parsing::load_bit_fields(path)?;
        let holidays = parsing::load_holidays(path)?;
        let timetable_metadata = parsing::load_timetable_metadata(path)?;

        // Basic data
        let (attributes, attributes_pk_type_converter) = parsing::load_attributes(path)?;
        let (directions, directions_pk_type_converter) = parsing::load_directions(path)?;
        let information_texts = parsing::load_information_texts(path)?;
        let lines = parsing::load_lines(path)?;
        let transport_companies = parsing::load_transport_companies(path)?;
        let (transport_types, transport_types_pk_type_converter) =
            parsing::load_transport_types(path)?;

        // Stop data
        let stop_connections = parsing::load_stop_connections(path, &attributes_pk_type_converter)?;
        let (stops, default_exchange_time) = parsing::load_stops(version, path)?;

        // Timetable data
        let (trips, trips_pk_type_converter) = parsing::load_trips(
            path,
            &transport_types_pk_type_converter,
            &attributes_pk_type_converter,
            &directions_pk_type_converter,
        )?;
        let (trip_platform, platforms) =
            parsing::load_platforms(path, &trips_pk_type_converter)?;
        let through_service = parsing::load_through_service(path, &trips_pk_type_converter)?;

        // Exchange times
        let exchange_times_administration = parsing::load_exchange_times_administration(path)?;
        let exchange_times_trip =
            parsing::load_exchange_times_trip(path, &trips_pk_type_converter)?;
        let exchange_times_line =
            parsing::load_exchange_times_line(path, &transport_types_pk_type_converter)?;

        log::info!("Building bit_fields_by_day...");
        let bit_fields_by_day = create_bit_fields_by_day(&bit_fields, &timetable_metadata)?;
        log::info!("Building bit_fields_by_stop_id...");
        let bit_fields_by_stop_id = create_bit_fields_by_stop_id(&trips);
        log::info!("Building trips_by_stop_id_and_bit_field_id...");
        let trips_by_stop_id_and_bit_field_id =
            create_trips_by_stop_id_and_bit_field_id(&trips);
        log::info!("Building stop_connections_by_stop_id...");
        let stop_connections_by_stop_id = create_stop_connections_by_stop_id(&stop_connections);
        log::info!("Building exchange_times_administration_map...");
        let exchange_times_administration_map =
            create_exchange_times_administration_map(&exchange_times_administration);
        log::info!("Building exchange_times_trip_map...");
        let exchange_times_trip_map = create_exchange_times_trip_map(&exchange_times_trip);

        let mut data_storage = Self {
            // Time-relevant data
            bit_fields,
            holidays,
            timetable_metadata,
            // Basic data
            attributes,
            information_texts,
            directions,
            lines,
            transport_companies,
            transport_types,
            // Stop data
            stop_connections,
            stops,
            // Timetable data
            trips,
            trip_platform,
            platforms,
            through_service,
            // Exchange times
            exchange_times_administration,
            exchange_times_trip,
            exchange_times_line,
            // Maps
            bit_fields_by_day,
            bit_fields_by_stop_id,
            trips_by_stop_id_and_bit_field_id,
            stop_connections_by_stop_id,
            exchange_times_administration_map,
            exchange_times_trip_map,
            // Additional global data
            default_exchange_time,
        };

        Ok(data_storage)
    }

    // Getters/Setters

    pub fn bit_fields(&self) -> &ResourceStorage<BitField> {
        &self.bit_fields
    }

    pub fn trips(&self) -> &ResourceStorage<Trip> {
        &self.trips
    }

    pub fn lines(&self) -> &ResourceStorage<Line> {
        &self.lines
    }

    pub fn platforms(&self) -> &ResourceStorage<Platform> {
        &self.platforms
    }

    pub fn stop_connections(&self) -> &ResourceStorage<StopConnection> {
        &self.stop_connections
    }

    pub fn stops(&self) -> &ResourceStorage<Stop> {
        &self.stops
    }

    pub fn transport_types(&self) -> &ResourceStorage<TransportType> {
        &self.transport_types
    }

    pub fn timetable_metadata(&self) -> &ResourceStorage<TimetableMetadataEntry> {
        &self.timetable_metadata
    }

    pub fn exchange_times_administration(&self) -> &ResourceStorage<ExchangeTimeAdministration> {
        &self.exchange_times_administration
    }

    pub fn exchange_times_trip(&self) -> &ResourceStorage<ExchangeTimeTrip> {
        &self.exchange_times_trip
    }

    pub fn exchange_times_line(&self) -> &ResourceStorage<ExchangeTimeLine> {
        &self.exchange_times_line
    }

    pub fn bit_fields_by_day(&self) -> &FxHashMap<NaiveDate, FxHashSet<i32>> {
        &self.bit_fields_by_day
    }

    pub fn bit_fields_by_stop_id(&self) -> &FxHashMap<i32, FxHashSet<i32>> {
        &self.bit_fields_by_stop_id
    }

    pub fn trips_by_stop_id_and_bit_field_id(&self) -> &FxHashMap<(i32, i32), Vec<i32>> {
        &self.trips_by_stop_id_and_bit_field_id
    }

    pub fn stop_connections_by_stop_id(&self) -> &FxHashMap<i32, FxHashSet<i32>> {
        &self.stop_connections_by_stop_id
    }

    pub fn exchange_times_administration_map(
        &self,
    ) -> &FxHashMap<(Option<i32>, String, String), i32> {
        &self.exchange_times_administration_map
    }

    pub fn exchange_times_trip_map(&self) -> &FxHashMap<(i32, i32, i32), FxHashSet<i32>> {
        &self.exchange_times_trip_map
    }

    pub fn default_exchange_time(&self) -> (i16, i16) {
        self.default_exchange_time
    }
}

// ------------------------------------------------------------------------------------------------
// --- ResourceStorage
// ------------------------------------------------------------------------------------------------

#[derive(Debug, Serialize, Deserialize)]
pub struct ResourceStorage<M: Model<M>> {
    data: FxHashMap<M::K, M>,
}

impl<M: Model<M>> ResourceStorage<M> {
    pub fn new(data: FxHashMap<M::K, M>) -> Self {
        Self { data }
    }

    pub fn data(&self) -> &FxHashMap<M::K, M> {
        &self.data
    }

    /// unwrap: Do not call this function if the key is not associated with data.
    pub fn find(&self, k: M::K) -> &M {
        &self.data().get(&k).unwrap()
    }

    pub fn entries(&self) -> Vec<&M> {
        self.data.values().collect()
    }

    pub fn resolve_ids(&self, ids: &FxHashSet<M::K>) -> Vec<&M> {
        ids.iter().map(|&id| self.find(id)).collect()
    }
}

// ------------------------------------------------------------------------------------------------
// --- Maps
// ------------------------------------------------------------------------------------------------

fn create_bit_fields_by_day(
    bit_fields: &ResourceStorage<BitField>,
    timetable_metadata: &ResourceStorage<TimetableMetadataEntry>,
) -> Result<FxHashMap<NaiveDate, FxHashSet<i32>>, Box<dyn Error>> {
    let start_date = timetable_start_date(timetable_metadata)?;
    let num_days =
        count_days_between_two_dates(start_date, timetable_end_date(timetable_metadata)?);

    let dates: Vec<NaiveDate> = (0..num_days)
        .into_iter()
        .map(|i| {
            start_date
                // unwrap: Converting i from usize to u64 will never fail.
                .checked_add_days(Days::new(i.try_into().unwrap()))
                // unwrap: Adding days will never fail.
                .unwrap()
        })
        .collect();

    let mut map = FxHashMap::default();
    dates.iter().for_each(|date| {
        map.entry(*date).or_insert(FxHashSet::default()).insert(0);
    });

    let result = bit_fields.data().keys().fold(map, |mut acc, bit_field_id| {
        let bit_field = bit_fields.find(*bit_field_id);
        let indexes: Vec<usize> = bit_field
            .bits()
            .iter()
            // The first two bits must be ignored.
            .skip(2)
            .enumerate()
            .filter(|(i, &x)| *i < num_days && x == 1)
            .map(|(i, _)| i)
            .collect();

        indexes.iter().for_each(|&i| {
            acc.entry(dates[i])
                .or_insert(FxHashSet::default())
                .insert(bit_field.id());
        });

        acc
    });
    Ok(result)
}

fn create_bit_fields_by_stop_id(
    trips: &ResourceStorage<Trip>,
) -> FxHashMap<i32, FxHashSet<i32>> {
    trips
        .entries()
        .into_iter()
        .fold(FxHashMap::default(), |mut acc, trip| {
            trip.route().iter().for_each(|route_entry| {
                acc.entry(route_entry.stop_id())
                    .or_insert(FxHashSet::default())
                    // If the trip has no bit_field_id, the default value is 0. A value of 0 means that the trip operates every day.
                    .insert(trip.bit_field_id().unwrap_or(0));
            });
            acc
        })
}

fn create_trips_by_stop_id_and_bit_field_id(
    trips: &ResourceStorage<Trip>,
) -> FxHashMap<(i32, i32), Vec<i32>> {
    trips
        .entries()
        .into_iter()
        .fold(FxHashMap::default(), |mut acc, trip| {
            trip.route().iter().for_each(|route_entry| {
                // If the trip has no bit_field_id, the default value is 0. A value of 0 means that the trip operates every day.
                acc.entry((route_entry.stop_id(), trip.bit_field_id().unwrap_or(0)))
                    .or_insert(Vec::new())
                    .push(trip.id());
            });
            acc
        })
}

fn create_stop_connections_by_stop_id(
    stop_connections: &ResourceStorage<StopConnection>,
) -> FxHashMap<i32, FxHashSet<i32>> {
    stop_connections
        .entries()
        .into_iter()
        .fold(FxHashMap::default(), |mut acc, stop_connection| {
            acc.entry(stop_connection.stop_id_1())
                .or_insert(FxHashSet::default())
                .insert(stop_connection.id());
            acc
        })
}

fn create_exchange_times_trip_map(
    exchange_times_trip: &ResourceStorage<ExchangeTimeTrip>,
) -> FxHashMap<(i32, i32, i32), FxHashSet<i32>> {
    exchange_times_trip.entries().into_iter().fold(
        FxHashMap::default(),
        |mut acc, exchange_time| {
            let key = (
                exchange_time.stop_id(),
                exchange_time.trip_id_1(),
                exchange_time.trip_id_2(),
            );

            acc.entry(key)
                .or_insert(FxHashSet::default())
                .insert(exchange_time.id());
            acc
        },
    )
}

fn create_exchange_times_administration_map(
    exchange_times_administration: &ResourceStorage<ExchangeTimeAdministration>,
) -> FxHashMap<(Option<i32>, String, String), i32> {
    exchange_times_administration.entries().into_iter().fold(
        FxHashMap::default(),
        |mut acc, exchange_time| {
            let key = (
                exchange_time.stop_id(),
                exchange_time.administration_1().into(),
                exchange_time.administration_2().into(),
            );

            acc.insert(key, exchange_time.id());
            acc
        },
    )
}
