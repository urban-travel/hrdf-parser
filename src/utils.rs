// ------------------------------------------------------------------------------------------------
// --- AutoIncrement
// ------------------------------------------------------------------------------------------------

use std::cell::RefCell;

use chrono::{Days, NaiveDate, NaiveTime};

use crate::{
    error::{HResult, HrdfError},
    models::TimetableMetadataEntry,
    parsing::error::{PResult, ParsingError},
    storage::ResourceStorage,
};

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

    pub fn get(&self) -> i32 {
        *self.value.borrow()
    }
}

pub fn add_1_day(date: NaiveDate) -> HResult<NaiveDate> {
    date.checked_add_days(Days::new(1))
        .ok_or(HrdfError::FailedToAddDays(date, 1))
}

pub fn sub_1_day(date: NaiveDate) -> HResult<NaiveDate> {
    date.checked_sub_days(Days::new(1))
        .ok_or(HrdfError::FailedToSubDays(date, 1))
}

pub fn count_days_between_two_dates(date_1: NaiveDate, date_2: NaiveDate) -> usize {
    usize::try_from((date_2 - date_1).num_days()).expect("The number of days should be positive.")
        + 1
}

pub fn create_time(hour: u32, minute: u32) -> PResult<NaiveTime> {
    NaiveTime::from_hms_opt(hour, minute, 0).ok_or(ParsingError::UnableToBuildTime(hour, minute, 0))
}

pub fn create_time_from_value(value: u32) -> PResult<NaiveTime> {
    create_time(value / 100, value % 100)
}

pub fn timetable_start_date(
    timetable_metadata: &ResourceStorage<TimetableMetadataEntry>,
) -> HResult<NaiveDate> {
    let result = timetable_metadata
        .data()
        .values()
        .find(|val| val.key() == "start_date")
        .ok_or(HrdfError::MissingStartDate)?
        .value_as_naive_date();
    Ok(result)
}

pub fn timetable_end_date(
    timetable_metadata: &ResourceStorage<TimetableMetadataEntry>,
) -> HResult<NaiveDate> {
    let result = timetable_metadata
        .data()
        .values()
        .find(|val| val.key() == "end_date")
        .ok_or(HrdfError::MissingEndDate)?
        .value_as_naive_date();
    Ok(result)
}
