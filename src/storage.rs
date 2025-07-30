use chrono::{Days, NaiveDate};
use rustc_hash::{FxHashMap, FxHashSet};
use serde::{Deserialize, Serialize};

use crate::{
    JourneyId, Result,
    models::{
        Attribute, BitField, Direction, ExchangeTimeAdministration, ExchangeTimeJourney,
        ExchangeTimeLine, Holiday, InformationText, Journey, JourneyPlatform, Line, Model,
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
    journeys: ResourceStorage<Journey>,
    journey_platform: ResourceStorage<JourneyPlatform>,
    platforms: ResourceStorage<Platform>,
    through_service: ResourceStorage<ThroughService>,

    // Exchange times
    exchange_times_administration: ResourceStorage<ExchangeTimeAdministration>,
    exchange_times_journey: ResourceStorage<ExchangeTimeJourney>,
    exchange_times_line: ResourceStorage<ExchangeTimeLine>,

    // Maps
    bit_fields_by_day: FxHashMap<NaiveDate, FxHashSet<i32>>,
    bit_fields_by_stop_id: FxHashMap<i32, FxHashSet<i32>>,
    journeys_by_stop_id_and_bit_field_id: FxHashMap<(i32, i32), Vec<i32>>,
    stop_connections_by_stop_id: FxHashMap<i32, FxHashSet<i32>>,
    bit_field_id_for_through_service_by_journey_id_stop_id:
        FxHashMap<(JourneyId, JourneyId, i32), i32>,
    exchange_times_administration_map: FxHashMap<(Option<i32>, String, String), i32>,
    exchange_times_journey_map: FxHashMap<(i32, JourneyId, JourneyId), FxHashSet<i32>>,

    // Additional global data
    default_exchange_time: (i16, i16), // (InterCity exchange time, Exchange time for all other journey types)
}

impl DataStorage {
    pub fn new(version: Version, path: &str) -> Result<Self> {
        // Time-relevant data
        let bit_fields = parsing::load_bit_fields(path)?;
        let holidays = parsing::load_holidays(path)?;
        let timetable_metadata = parsing::load_timetable_metadata(path)?;

        // Basic data
        let (attributes, attributes_pk_type_converter) = parsing::load_attributes(version, path)?;
        let (directions, directions_pk_type_converter) = parsing::load_directions(path)?;
        let information_texts = parsing::load_information_texts(path)?;
        let lines = parsing::load_lines(path)?;
        let transport_companies = parsing::load_transport_companies(path)?;
        let (transport_types, transport_types_pk_type_converter) =
            parsing::load_transport_types(version, path)?;

        // Stop data
        let stop_connections = parsing::load_stop_connections(path, &attributes_pk_type_converter)?;
        let (stops, default_exchange_time) = parsing::load_stops(version, path)?;

        // Timetable data
        let (journeys, journeys_pk_type_converter) = parsing::load_journeys(
            path,
            &transport_types_pk_type_converter,
            &attributes_pk_type_converter,
            &directions_pk_type_converter,
        )?;
        let (journey_platform, platforms) =
            parsing::load_platforms(version, path, &journeys_pk_type_converter)?;
        let through_service = parsing::load_through_service(path, &journeys_pk_type_converter)?;

        // Exchange times
        let exchange_times_administration = parsing::load_exchange_times_administration(path)?;
        let exchange_times_journey =
            parsing::load_exchange_times_journey(path, &journeys_pk_type_converter)?;
        let exchange_times_line =
            parsing::load_exchange_times_line(path, &transport_types_pk_type_converter)?;

        log::info!("Building bit_fields_by_day...");
        let bit_fields_by_day = create_bit_fields_by_day(&bit_fields, &timetable_metadata)?;
        log::info!("Building bit_fields_by_stop_id...");
        let bit_fields_by_stop_id = create_bit_fields_by_stop_id(&journeys);
        log::info!("Building journeys by stop id and bit field_id...");
        let journeys_by_stop_id_and_bit_field_id =
            create_journeys_by_stop_id_and_bit_field_id(&journeys);
        log::info!("Building stop connections by stop id...");
        let bit_field_id_for_through_service_by_journey_id_stop_id =
            create_bit_field_id_through_service_by_journey_id_stop_id(&through_service);
        log::info!("Building stop connections by stop id...");
        let stop_connections_by_stop_id = create_stop_connections_by_stop_id(&stop_connections);
        log::info!("Building exchange times administration map...");
        let exchange_times_administration_map =
            create_exchange_times_administration_map(&exchange_times_administration);
        log::info!("Building exchange times journey_map...");
        let exchange_times_journey_map = create_exchange_times_journey_map(&exchange_times_journey);
        log::info!("Building through service map...");

        let data_storage = Self {
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
            journeys,
            journey_platform,
            platforms,
            through_service,
            // Exchange times
            exchange_times_administration,
            exchange_times_journey,
            exchange_times_line,
            // Maps
            bit_fields_by_day,
            bit_fields_by_stop_id,
            journeys_by_stop_id_and_bit_field_id,
            stop_connections_by_stop_id,
            bit_field_id_for_through_service_by_journey_id_stop_id,
            exchange_times_administration_map,
            exchange_times_journey_map,
            // Additional global data
            default_exchange_time,
        };

        Ok(data_storage)
    }

    // Getters/Setters

    pub fn bit_fields(&self) -> &ResourceStorage<BitField> {
        &self.bit_fields
    }

    pub fn journeys(&self) -> &ResourceStorage<Journey> {
        &self.journeys
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

    pub fn through_service(&self) -> &ResourceStorage<ThroughService> {
        &self.through_service
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

    pub fn exchange_times_journey(&self) -> &ResourceStorage<ExchangeTimeJourney> {
        &self.exchange_times_journey
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

    pub fn journeys_by_stop_id_and_bit_field_id(&self) -> &FxHashMap<(i32, i32), Vec<i32>> {
        &self.journeys_by_stop_id_and_bit_field_id
    }

    pub fn stop_connections_by_stop_id(&self) -> &FxHashMap<i32, FxHashSet<i32>> {
        &self.stop_connections_by_stop_id
    }

    pub fn bit_field_id_for_through_service_by_journey_id_stop_id(
        &self,
    ) -> &FxHashMap<(JourneyId, JourneyId, i32), i32> {
        &self.bit_field_id_for_through_service_by_journey_id_stop_id
    }

    pub fn exchange_times_administration_map(
        &self,
    ) -> &FxHashMap<(Option<i32>, String, String), i32> {
        &self.exchange_times_administration_map
    }

    pub fn exchange_times_journey_map(
        &self,
    ) -> &FxHashMap<(i32, JourneyId, JourneyId), FxHashSet<i32>> {
        &self.exchange_times_journey_map
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

    pub fn find(&self, k: M::K) -> Option<&M> {
        // TODO: there might be a problem when k is not in data so we can't unwrap here
        self.data().get(&k)
    }

    pub fn entries(&self) -> Vec<&M> {
        self.data.values().collect()
    }

    pub fn resolve_ids(&self, ids: &FxHashSet<M::K>) -> Option<Vec<&M>> {
        ids.iter().map(|&id| self.find(id)).collect()
    }
}

// ------------------------------------------------------------------------------------------------
// --- Maps
// ------------------------------------------------------------------------------------------------

fn create_bit_fields_by_day(
    bit_fields: &ResourceStorage<BitField>,
    timetable_metadata: &ResourceStorage<TimetableMetadataEntry>,
) -> Result<FxHashMap<NaiveDate, FxHashSet<i32>>> {
    let start_date = timetable_start_date(timetable_metadata)?;
    let num_days =
        count_days_between_two_dates(start_date, timetable_end_date(timetable_metadata)?);

    let dates: Vec<NaiveDate> = (0..num_days)
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
        let bit_field = bit_fields
            .find(*bit_field_id)
            .unwrap_or_else(|| panic!("Bitfield id {:?} not found.", bit_field_id));
        let indexes: Vec<usize> = bit_field
            .bits()
            .iter()
            // The first two bits must be ignored.
            .skip(2)
            .enumerate()
            .filter(|&(ref i, &x)| *i < num_days && x == 1)
            .map(|(i, _)| i)
            .collect();

        indexes.iter().for_each(|&i| {
            acc.entry(dates[i]).or_default().insert(bit_field.id());
        });

        acc
    });
    Ok(result)
}

fn create_bit_fields_by_stop_id(
    journeys: &ResourceStorage<Journey>,
) -> FxHashMap<i32, FxHashSet<i32>> {
    journeys
        .entries()
        .into_iter()
        .fold(FxHashMap::default(), |mut acc, journey| {
            journey.route().iter().for_each(|route_entry| {
                acc.entry(route_entry.stop_id())
                    .or_default()
                    // If the journey has no bit_field_id, the default value is 0. A value of 0 means that the journey operates every day.
                    .insert(journey.bit_field_id().unwrap_or(0));
            });
            acc
        })
}

fn create_journeys_by_stop_id_and_bit_field_id(
    journeys: &ResourceStorage<Journey>,
) -> FxHashMap<(i32, i32), Vec<i32>> {
    journeys
        .entries()
        .into_iter()
        .fold(FxHashMap::default(), |mut acc, journey| {
            journey.route().iter().for_each(|route_entry| {
                // If the journey has no bit_field_id, the default value is 0. A value of 0 means that the journey operates every day.
                acc.entry((route_entry.stop_id(), journey.bit_field_id().unwrap_or(0)))
                    .or_default()
                    .push(journey.id());
            });
            acc
        })
}

/// Given journey_stop_id, and journey_id_1, journey_id_2, we obtain the bit_field_id of the ThroughService
fn create_bit_field_id_through_service_by_journey_id_stop_id(
    through_services: &ResourceStorage<ThroughService>,
) -> FxHashMap<(JourneyId, JourneyId, i32), i32> {
    through_services
        .entries()
        .into_iter()
        .fold(FxHashMap::default(), |mut acc, through_service| {
            let journey_1_id = through_service.journey_1_id();
            let journey_2_id = through_service.journey_2_id();
            let journey_stop_id = through_service.journey_1_stop_id();
            let bit_field_id = through_service.bit_field_id();

            acc.insert(
                (journey_1_id.clone(), journey_2_id.clone(), journey_stop_id),
                bit_field_id,
            );
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
                .or_default()
                .insert(stop_connection.id());
            acc
        })
}

fn create_exchange_times_journey_map(
    exchange_times_journey: &ResourceStorage<ExchangeTimeJourney>,
) -> FxHashMap<(i32, JourneyId, JourneyId), FxHashSet<i32>> {
    exchange_times_journey.entries().into_iter().fold(
        FxHashMap::default(),
        |mut acc, exchange_time| {
            let key = (
                exchange_time.stop_id(),
                (
                    exchange_time.journey_legacy_id_1(),
                    exchange_time.administration_1().to_string(),
                ),
                (
                    exchange_time.journey_legacy_id_2(),
                    exchange_time.administration_2().to_string(),
                ),
            );

            acc.entry(key).or_default().insert(exchange_time.id());
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
