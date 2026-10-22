use crate::hrdf::{AddableTypes, ModifiableTypes, RemovableTypes};
use crate::{error::{HResult, HrdfError}, models::{
    Attribute, BitField, Direction, ExchangeTimeAdministration, ExchangeTimeJourney,
    ExchangeTimeLine, Holiday, InformationText, Journey, JourneyPlatform, Line, Model,
    Platform, Stop, StopConnection, ThroughService, TimetableMetadataEntry, TransportCompany,
    TransportType, Version,
}, parsing, utils::{count_days_between_two_dates, timetable_end_date, timetable_start_date}, JourneyError, JourneyId, JourneyMetadataType, JourneyRouteEntry};
use chrono::{Days, NaiveDate, NaiveTime};
use nom::sequence::Tuple;
use rustc_hash::{FxHashMap, FxHashSet};
use serde::{Deserialize, Serialize};
use sha2::digest::consts::True;
use std::collections::HashMap;
use std::hash::Hasher;
use std::{path::Path, time::Instant};
use std::fs::File;
use std::time::Duration;
use csv::Writer;
// ------------------------------------------------------------------------------------------------
// --- DataStorage
// ------------------------------------------------------------------------------------------------
//

#[derive(Debug, Serialize, Deserialize, Clone)]
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
    journeys_by_stop_id_and_bit_field_id: FxHashMap<(i32, i32), FxHashSet<i32>>,
    stop_connections_by_stop_id: FxHashMap<i32, FxHashSet<i32>>,
    bit_field_id_for_through_service_by_journey_id_stop_id:
        FxHashMap<(JourneyId, JourneyId, i32), i32>,
    exchange_times_administration_map: FxHashMap<(Option<i32>, String, String), i32>,
    exchange_times_journey_map: FxHashMap<(i32, JourneyId, JourneyId), FxHashSet<i32>>,

    // Additional global data
    default_exchange_time: (i16, i16), // (InterCity exchange time, Exchange time for all other journey types)
}

impl DataStorage {
    pub fn new(version: Version, path: &Path) -> HResult<Self> {
        // Time-relevant data
        let complete = Instant::now();
        let now = Instant::now();
        let bit_fields = parsing::load_bit_fields(path)?;
        log::info!("Time elapsed for bitfields parsing: {:?}", now.elapsed());
        let now = Instant::now();
        let holidays = parsing::load_holidays(path)?;
        log::info!("Time elapsed for holidays parsing: {:?}", now.elapsed());

        let now = Instant::now();
        let timetable_metadata = parsing::load_timetable_metadata(path)?;
        log::info!(
            "Time elapsed for timetable_metadata parsing: {:?}",
            now.elapsed()
        );

        // Basic data
        let now = Instant::now();
        let (attributes, attributes_pk_type_converter) = parsing::load_attributes(path)?;
        log::info!("Time elapsed for attributes parsing: {:?}", now.elapsed());
        let now = Instant::now();
        let (directions, directions_pk_type_converter) = parsing::load_directions(path)?;
        log::info!("Time elapsed for directions parsing: {:?}", now.elapsed());
        let now = Instant::now();
        let information_texts = parsing::load_information_texts(path)?;
        log::info!(
            "Time elapsed for information_texts parsing: {:?}",
            now.elapsed()
        );
        let now = Instant::now();
        let lines = parsing::load_lines(path)?;
        log::info!("Time elapsed for line parsing: {:?}", now.elapsed());
        let now = Instant::now();
        let transport_companies = parsing::load_transport_companies(path)?;
        log::info!(
            "Time elapsed for transport_companies parsing: {:?}",
            now.elapsed()
        );
        let now = Instant::now();
        let (transport_types, transport_types_pk_type_converter) =
            parsing::load_transport_types(path)?;
        log::info!(
            "Time elapsed for transport_types parsing: {:?}",
            now.elapsed()
        );

        // Stop data
        let now = Instant::now();
        let stop_connections = parsing::load_stop_connections(path, &attributes_pk_type_converter)?;
        log::info!(
            "Time elapsed for stop_connections parsing: {:?}",
            now.elapsed()
        );
        let now = Instant::now();
        let (stops, default_exchange_time) = parsing::load_stops(version, path)?;
        log::info!("Time elapsed for stops parsing: {:?}", now.elapsed());

        // Timetable data
        let now = Instant::now();
        let (journeys, journeys_pk_type_converter) = parsing::load_journeys(
            path,
            &transport_types_pk_type_converter,
            &attributes_pk_type_converter,
            &directions_pk_type_converter,
        )?;
        log::info!("Time elapsed for journeys parsing: {:?}", now.elapsed());

        let now = Instant::now();
        let (journey_platform, platforms) =
            parsing::load_platforms(version, path, &journeys_pk_type_converter)?;
        log::info!("Time elapsed for platforms parsing: {:?}", now.elapsed());
        let now = Instant::now();
        let through_service = parsing::load_through_service(path, &journeys_pk_type_converter)?;
        log::info!(
            "Time elapsed for through_service parsing: {:?}",
            now.elapsed()
        );

        // Exchange times
        let now = Instant::now();
        let exchange_times_administration = parsing::load_exchange_times_administration(path)?;
        log::info!(
            "Time elapsed for exchange_times_administration parsing: {:?}",
            now.elapsed()
        );
        let now = Instant::now();
        let exchange_times_journey =
            parsing::load_exchange_times_journey(path, &journeys_pk_type_converter)?;
        log::info!(
            "Time elapsed for exchange_times_journey parsing: {:?}",
            now.elapsed()
        );
        let now = Instant::now();
        let exchange_times_line =
            parsing::load_exchange_times_line(path, &transport_types_pk_type_converter)?;
        log::info!(
            "Time elapsed for exchange_times_line parsing: {:?}",
            now.elapsed()
        );

        log::info!("Parsing of all HRDF files in {:?}", complete.elapsed());

        log::info!("Building bit_fields_by_day...");
        let bit_fields_by_day = create_bit_fields_by_day(&bit_fields, &timetable_metadata)?;
        log::info!("Building bit_fields_by_stop_id...");
        let bit_fields_by_stop_id = create_bit_fields_by_stop_id(&journeys)?;
        log::info!("Building journeys by stop id and bit field_id...");
        let journeys_by_stop_id_and_bit_field_id =
            create_journeys_by_stop_id_and_bit_field_id(&journeys)?;
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

    pub fn journeys_by_stop_id_and_bit_field_id(&self) -> &FxHashMap<(i32, i32), FxHashSet<i32>> {
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


    pub fn filter(self, elements_to_remove: &HashMap<RemovableTypes, Vec<String>>) -> Self {
        let mut filtered = self;
        let administration = String::from("000881");
        for (rem_type, elements) in elements_to_remove.iter() {
            match rem_type {
                RemovableTypes::Line => {
                    let now = Instant::now();
                    // First translate line names to ids
                    let removed_line_ids = elements
                        .into_iter()
                        .flat_map(|value| {
                            filtered
                                .lines
                                .data
                                .iter()
                                .filter(move |(_key, line)| value == line.get_name())
                                .map(|(key, _value)| *key)
                        })
                        .collect::<Vec<i32>>();
                    log::info!("Done finding line ids, time elapsed: {:?}", now.elapsed());

                    let now = Instant::now();
                    // Then remove lines we don't want to keep
                    filtered.lines = filtered
                        .lines
                        .filter(|_, l| !elements.contains(l.get_name()));
                    log::info!("Done filtering lines by name, time elapsed: {:?}", now.elapsed());

                    let now = Instant::now();
                    // Finally remove journeys using removed lines
                    let removed_journeys_ids: FxHashSet<_> = filtered
                        .journeys
                        .data()
                        .iter()
                        .filter_map(|(id, journey)| {
                            if journey.administration() == administration {
                                match journey.metadata().get(&JourneyMetadataType::Line) {
                                    Some(entry) => entry
                                        .iter()
                                        .find(|entry| match entry.resource_id {
                                            Some(id) => !removed_line_ids.contains(&id),
                                            None => match &entry.extra_field_1 {
                                                Some(id) => !elements.contains(id),
                                                None => panic!("journey with wrong format"),
                                            },
                                        })
                                        .map(|_| *id),
                                    None => None,
                                }
                            } else { None }
                        })
                        .collect();
                    log::info!("Done finding journeys to remove by name, time elapsed: {:?}", now.elapsed());
                    let now = Instant::now();
                    filtered.journeys = filtered.journeys.filter(|_key, journey: &mut Journey| {
                        removed_journeys_ids.contains(&journey.id())
                    });
                    log::info!("Done filtering journeys by id, time elapsed: {:?}", now.elapsed());
                    let now = Instant::now();
                    filtered.journeys_by_stop_id_and_bit_field_id = filtered
                        .journeys_by_stop_id_and_bit_field_id
                        .iter()
                        .map(|(index, journeys)| {
                            (
                                *index,
                                journeys
                                    .iter()
                                    .filter_map(|journey_id| {
                                        if removed_journeys_ids.contains(journey_id) {
                                            Some(*journey_id)
                                        } else {
                                            None
                                        }
                                    })
                                    .collect(),
                            )
                        })
                        .collect();
                    log::info!("Done filtering journeys by name, time elapsed: {:?}", now.elapsed());
                }
                RemovableTypes::Stop => {
                    let removed_stop_ids : FxHashSet<_> = filtered.stops.data.iter().filter_map(
                        |(_, s)| if elements.contains(&String::from(s.name())) {Some(s.id())} else { None }
                    ).collect();
                    filtered.stops = filtered.stops.filter(|_, s| !elements.contains(&String::from(s.name())));
                    filtered.journeys = filtered.journeys.map(
                        |(ix, journey)|
                            (*ix, journey.filter_route(&removed_stop_ids))
                    )
                }
                RemovableTypes::TransportType => { unimplemented!() }
                RemovableTypes::TransportCompany => { unimplemented!() }
            }
        }
        filtered
    }

    pub fn modify(self, elements_to_modify: &ModifiableTypes) -> Self {
        let mut modified = self;
        match elements_to_modify {
            // todo: Check if copy is avoidable
            ModifiableTypes::Line { modifications } => {
                modified.lines = modified.lines.map(|(key, value)| {
                    if modifications.contains_key(key) {
                        (*key, modifications[key].clone())
                    } else {
                        (*key, value.clone())
                    }
                })
            }
            ModifiableTypes::Stop { modifications } => {
                modified.stops = modified.stops.map(|(key, value)| {
                    if modifications.contains_key(key) {
                        (*key, modifications[key].clone())
                    } else {
                        (*key, value.clone())
                    }
                })
            }
            ModifiableTypes::TransportType => { unimplemented!() }
            ModifiableTypes::TransportCompany => { unimplemented!() }
        }
        modified
    }

    pub fn add(self, elements_to_add: &AddableTypes) -> Self {
        let mut completed = self;
        match elements_to_add {
            AddableTypes::Line { add_list } => {
                // todo: fix hashmap new indices
                completed.lines = completed.lines.extend(
                    add_list
                        .iter()
                        .enumerate()
                        .map(|(i, line)| ((1000000 + i) as i32, line.clone()))
                        .collect(),
                )
            }
            AddableTypes::Stop { add_list } => {
                completed.stops = completed.stops.extend(
                    add_list
                        .iter()
                        .enumerate()
                        .map(|(i, line)| ((1001000 + i) as i32, line.clone()))
                        .collect(),
                )
            }
            AddableTypes::TransportType => {}
            AddableTypes::TransportCompany => {}
        }
        completed
    }

    pub fn export_to_csv_time_graph(&self, wf: &mut Writer<File>, filter_administration: Option<String>, start_date: &NaiveDate, duration: &Duration) {
        self.journeys.data.iter().for_each(
            |(_, journeys)| {
                if let Some(metadata) = journeys.metadata().get(&JourneyMetadataType::Line) {
                    let metadata = metadata[0].clone();
                    let tag = String::from(journeys.administration()) + "#" + metadata.extra_field_1.or(Some(String::from(""))).unwrap().trim();
                    if let Some(filter_administration) = filter_administration.clone() && filter_administration != String::from(journeys.administration()){
                        return
                    }
                    journeys.route().iter().fold((&None, 0),
                                                 |(prev_time, old_stop_id), journey: &JourneyRouteEntry| -> (&Option<NaiveTime>, i32) {
                                                     if let Some(prev_time) = prev_time {
                                                         let cost = journey.arrival_time().unwrap() - *prev_time;
                                                         let active_time = prev_time;
                                                         let start_node = old_stop_id;
                                                         let destination_node = journey.stop_id();
                                                         wf.serialize((start_node, destination_node, active_time, cost.num_minutes(), tag.clone())).expect("Something went wrong serializing data");
                                                     }
                                                     (journey.departure_time(), journey.stop_id())
                                                 },
                    );
                }
            }
        )
    }
}

// ------------------------------------------------------------------------------------------------
// --- ResourceStorage
// ------------------------------------------------------------------------------------------------

#[derive(Debug, Serialize, Deserialize, Clone)]
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
        self.data().get(&k)
    }

    pub fn entries(&self) -> Vec<&M> {
        self.data.values().collect()
    }

    pub fn resolve_ids(&self, ids: &FxHashSet<M::K>) -> Option<Vec<&M>> {
        ids.iter().map(|&id| self.find(id)).collect()
    }

    pub fn filter<F>(mut self, predicate: F) -> Self
    where
        F: FnMut(&M::K, &mut M) -> bool,
    {
        self.data.retain(predicate);
        self
    }

    pub fn map<F>(mut self, predicate: F) -> Self
    where
        F: FnMut((&M::K, &M)) -> (M::K, M),
    {
        self.data = self.data.iter().map(predicate).collect();
        self
    }

    pub fn extend(mut self, add_list: Vec<(<M as Model<M>>::K, M)>) -> Self {
        self.data.extend(add_list);
        self
    }
}

// ------------------------------------------------------------------------------------------------
// --- Maps
// ------------------------------------------------------------------------------------------------

fn create_bit_fields_by_day(
    bit_fields: &ResourceStorage<BitField>,
    timetable_metadata: &ResourceStorage<TimetableMetadataEntry>,
) -> HResult<FxHashMap<NaiveDate, FxHashSet<i32>>> {
    let start_date = timetable_start_date(timetable_metadata)?;
    let num_days =
        count_days_between_two_dates(start_date, timetable_end_date(timetable_metadata)?);

    let dates = (0..num_days)
        .map(|i| {
            let i = i.try_into().unwrap();
            start_date
                // unwrap: Converting i from usize to u64 will never fail.
                .checked_add_days(Days::new(i))
                .ok_or(HrdfError::FailedToAddDays(start_date, i))
        })
        .collect::<HResult<Vec<NaiveDate>>>()?;

    let mut map = FxHashMap::default();
    dates.iter().for_each(|date| {
        map.entry(*date).or_insert(FxHashSet::default()).insert(0);
    });

    bit_fields
        .data()
        .keys()
        .try_fold(map, |mut acc, bit_field_id| {
            let bit_field = bit_fields
                .find(*bit_field_id)
                .ok_or(HrdfError::BitFieldIdNotFound(*bit_field_id))?;
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

            Ok(acc)
        })
}

fn create_bit_fields_by_stop_id(
    journeys: &ResourceStorage<Journey>,
) -> HResult<FxHashMap<i32, FxHashSet<i32>>> {
    journeys.entries().into_iter().try_fold(
        FxHashMap::default(),
        |mut acc: FxHashMap<i32, FxHashSet<i32>>, journey| {
            journey.route().iter().try_for_each(|route_entry| {
                acc.entry(route_entry.stop_id())
                    .or_default()
                    // If the journey has no bit_field_id, the default value is 0. A value of 0 means that the journey operates every day.
                    .insert(journey.bit_field_id()?.unwrap_or(0));
                Ok::<(), JourneyError>(())
            })?;
            Ok(acc)
        },
    )
}

fn create_journeys_by_stop_id_and_bit_field_id(
    journeys: &ResourceStorage<Journey>,
) -> HResult<FxHashMap<(i32, i32), FxHashSet<i32>>> {
    journeys.entries().into_iter().try_fold(
        FxHashMap::default(),
        |mut acc: FxHashMap<(i32, i32), FxHashSet<i32>>, journey| {
            journey.route().iter().try_for_each(|route_entry| {
                // If the journey has no bit_field_id, the default value is 0. A value of 0 means that the journey operates every day.
                acc.entry((route_entry.stop_id(), journey.bit_field_id()?.unwrap_or(0)))
                    .or_default()
                    .insert(journey.id());
                Ok::<(), JourneyError>(())
            })?;
            Ok(acc)
        },
    )
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

#[cfg(test)]
mod tests {
    use crate::{JourneyMetadataEntry, JourneyMetadataType, JourneyRouteEntry};

    use super::*;
    use chrono::{NaiveDate, NaiveTime};
    use rustc_hash::FxHashMap;

    fn build_timetable_metadata(start: &str, end: &str) -> ResourceStorage<TimetableMetadataEntry> {
        let mut data = FxHashMap::default();
        data.insert(
            1,
            TimetableMetadataEntry::new(1, "start_date".to_string(), start.to_string()),
        );
        data.insert(
            2,
            TimetableMetadataEntry::new(2, "end_date".to_string(), end.to_string()),
        );
        ResourceStorage::new(data)
    }

    fn build_bit_field(bits: Vec<u8>) -> ResourceStorage<BitField> {
        let mut data = FxHashMap::default();
        data.insert(1, BitField::new(1, bits));
        ResourceStorage::new(data)
    }

    fn build_journey_with_bitfield(
        id: i32,
        legacy_id: i32,
        bit_field_id: Option<i32>,
        route_stops: &[i32],
    ) -> Journey {
        let mut journey = Journey::new(id, legacy_id, "CH".to_string());
        journey.add_metadata_entry(
            JourneyMetadataType::BitField,
            JourneyMetadataEntry::new(
                None,
                None,
                None,
                bit_field_id,
                Some(NaiveTime::from_hms_opt(8, 0, 0).unwrap()),
                None,
                None,
                None,
            ),
        );

        for (index, stop_id) in route_stops.iter().enumerate() {
            let departure = if index + 1 == route_stops.len() {
                None
            } else {
                Some(NaiveTime::from_hms_opt(8, (index as u32) * 10, 0).unwrap())
            };
            let arrival = if index == 0 {
                None
            } else {
                Some(NaiveTime::from_hms_opt(8, (index as u32) * 10 - 5, 0).unwrap())
            };
            journey.add_route_entry(JourneyRouteEntry::new(*stop_id, arrival, departure));
        }

        journey
    }

    #[test]
    fn bit_fields_by_day_include_defaults_and_active_days() {
        let metadata = build_timetable_metadata("2024-01-01", "2024-01-03");
        let bit_fields = build_bit_field(vec![0, 0, 1, 0, 1]);

        let map = create_bit_fields_by_day(&bit_fields, &metadata).unwrap();
        let day1 = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        let day2 = NaiveDate::from_ymd_opt(2024, 1, 2).unwrap();
        let day3 = NaiveDate::from_ymd_opt(2024, 1, 3).unwrap();

        assert_eq!(map.len(), 3);
        assert!(map.get(&day1).unwrap().contains(&0));
        assert!(map.get(&day2).unwrap().contains(&0));
        assert!(map.get(&day3).unwrap().contains(&0));
        assert!(map.get(&day1).unwrap().contains(&1));
        assert!(!map.get(&day2).unwrap().contains(&1));
        assert!(map.get(&day3).unwrap().contains(&1));
    }

    #[test]
    fn journey_maps_group_by_stop_and_bitfield() {
        let journey_a = build_journey_with_bitfield(1, 100, Some(7), &[10, 20]);
        let journey_b = build_journey_with_bitfield(2, 200, None, &[10]);

        let mut journeys_data = FxHashMap::default();
        journeys_data.insert(1, journey_a);
        journeys_data.insert(2, journey_b);
        let journeys = ResourceStorage::new(journeys_data);

        let by_stop = create_bit_fields_by_stop_id(&journeys).unwrap();
        assert!(by_stop.get(&10).unwrap().contains(&7));
        assert!(by_stop.get(&10).unwrap().contains(&0));
        assert!(by_stop.get(&20).unwrap().contains(&7));

        let by_stop_and_bit = create_journeys_by_stop_id_and_bit_field_id(&journeys).unwrap();
        assert_eq!(by_stop_and_bit.get(&(10, 7)).unwrap(), &FxHashSet::from_iter([1]));
        assert_eq!(by_stop_and_bit.get(&(10, 0)).unwrap(), &FxHashSet::from_iter([2]));
        assert_eq!(by_stop_and_bit.get(&(20, 7)).unwrap(), &FxHashSet::from_iter([1]));
    }

    #[test]
    fn stop_connection_map_collects_ids() {
        let mut data = FxHashMap::default();
        data.insert(1, StopConnection::new(1, 10, 11, 5));
        data.insert(2, StopConnection::new(2, 10, 12, 7));
        let storage = ResourceStorage::new(data);

        let map = create_stop_connections_by_stop_id(&storage);
        let ids = map.get(&10).unwrap();
        assert_eq!(ids.len(), 2);
        assert!(ids.contains(&1));
        assert!(ids.contains(&2));
    }

    #[test]
    fn exchange_time_maps_resolve_expected_keys() {
        let mut admin_data = FxHashMap::default();
        admin_data.insert(
            1,
            ExchangeTimeAdministration::new(1, Some(10), "A".to_string(), "B".to_string(), 5),
        );
        let admin_storage = ResourceStorage::new(admin_data);
        let admin_map = create_exchange_times_administration_map(&admin_storage);
        assert_eq!(
            *admin_map
                .get(&(Some(10), "A".to_string(), "B".to_string()))
                .unwrap(),
            1
        );

        let mut journey_data = FxHashMap::default();
        journey_data.insert(
            1,
            ExchangeTimeJourney::new(
                1,
                10,
                (100, "A".to_string()),
                (200, "B".to_string()),
                6,
                false,
                None,
            ),
        );
        journey_data.insert(
            2,
            ExchangeTimeJourney::new(
                2,
                10,
                (100, "A".to_string()),
                (200, "B".to_string()),
                8,
                true,
                Some(3),
            ),
        );
        let journey_storage = ResourceStorage::new(journey_data);
        let journey_map = create_exchange_times_journey_map(&journey_storage);

        let key = (10, (100, "A".to_string()), (200, "B".to_string()));
        let ids = journey_map.get(&key).unwrap();
        assert_eq!(ids.len(), 2);
        assert!(ids.contains(&1));
        assert!(ids.contains(&2));
    }

    #[test]
    fn through_service_map_keys_by_journeys_and_stop() {
        let mut data = FxHashMap::default();
        data.insert(
            1,
            ThroughService::new(1, (100, "A".to_string()), 10, (200, "B".to_string()), 20, 3),
        );
        let storage = ResourceStorage::new(data);
        let map = create_bit_field_id_through_service_by_journey_id_stop_id(&storage);

        let key = ((100, "A".to_string()), (200, "B".to_string()), 10);
        assert_eq!(*map.get(&key).unwrap(), 3);
    }
}
