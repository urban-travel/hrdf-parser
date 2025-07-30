// ------------------------------------------------------------------------------------------------
// --- AutoIncrement
// ------------------------------------------------------------------------------------------------

use std::cell::RefCell;

use chrono::{Days, NaiveDate, NaiveTime};

use crate::{Result, error::ErrorKind, models::TimetableMetadataEntry, storage::ResourceStorage};

pub struct AutoIncrement {
    value: RefCell<i32>,
}

impl AutoIncrement {
    pub fn new() -> Self {
        Self {
            value: RefCell::new(0),
        }
    }

    pub fn next(&self) -> i32 {
        *self.value.borrow_mut() += 1;
        *self.value.borrow()
    }
}

pub fn add_1_day(date: NaiveDate) -> NaiveDate {
    date.checked_add_days(Days::new(1))
        .expect("Error adding 1 day to the date.")
}

pub fn sub_1_day(date: NaiveDate) -> NaiveDate {
    date.checked_sub_days(Days::new(1))
        .expect("Error substracting 1 day to the date.")
}

pub fn count_days_between_two_dates(date_1: NaiveDate, date_2: NaiveDate) -> usize {
    usize::try_from((date_2 - date_1).num_days()).expect("The number of days should be positive.")
        + 1
}

pub fn create_time(hour: u32, minute: u32) -> NaiveTime {
    NaiveTime::from_hms_opt(hour, minute, 0)
        .expect("Impossible to create a NaiveTime from hour and minute.")
}

pub fn create_time_from_value(value: u32) -> NaiveTime {
    create_time(value / 100, value % 100)
}

pub fn timetable_start_date(
    timetable_metadata: &ResourceStorage<TimetableMetadataEntry>,
) -> Result<NaiveDate> {
    let result = timetable_metadata
        .data()
        .values()
        .find(|val| val.key() == "start_date")
        .ok_or(ErrorKind::KeyMissing { name: "start_date" })?
        .value_as_NaiveDate();
    Ok(result)
}

pub fn timetable_end_date(
    timetable_metadata: &ResourceStorage<TimetableMetadataEntry>,
) -> Result<NaiveDate> {
    let result = timetable_metadata
        .data()
        .values()
        .find(|val| val.key() == "end_date")
        .ok_or(ErrorKind::KeyMissing { name: "end_date" })?
        .value_as_NaiveDate();
    Ok(result)
}
