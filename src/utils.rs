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

/// Number of days from `date_1` to `date_2`, both included.
/// Returns `HrdfError::OutOfRangeDate(date_2)` if `date_2` is before `date_1`.
pub fn count_days_between_two_dates(date_1: NaiveDate, date_2: NaiveDate) -> HResult<usize> {
    usize::try_from((date_2 - date_1).num_days())
        .map(|days| days + 1)
        .map_err(|_| HrdfError::OutOfRangeDate(date_2))
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
    timetable_metadata
        .data()
        .values()
        .find(|val| val.key() == "start_date")
        .ok_or(HrdfError::MissingStartDate)?
        .value_as_naive_date()
}

pub fn timetable_end_date(
    timetable_metadata: &ResourceStorage<TimetableMetadataEntry>,
) -> HResult<NaiveDate> {
    timetable_metadata
        .data()
        .values()
        .find(|val| val.key() == "end_date")
        .ok_or(HrdfError::MissingEndDate)?
        .value_as_naive_date()
}

#[cfg(test)]
mod tests {
    use super::*;
    use rustc_hash::FxHashMap;

    #[test]
    fn auto_increment_starts_at_zero() {
        let counter = AutoIncrement::new();
        assert_eq!(counter.get(), 0);
    }

    #[test]
    fn auto_increment_next_increments_and_returns_new_value() {
        let counter = AutoIncrement::new();
        assert_eq!(counter.next(), 1);
        assert_eq!(counter.next(), 2);
        assert_eq!(counter.next(), 3);
    }

    #[test]
    fn auto_increment_get_does_not_advance() {
        let counter = AutoIncrement::new();
        counter.next();
        assert_eq!(counter.get(), 1);
        assert_eq!(counter.get(), 1);
    }

    #[test]
    fn add_1_day_rolls_over_month_and_year() {
        assert_eq!(
            add_1_day(NaiveDate::from_ymd_opt(2026, 1, 15).unwrap()).unwrap(),
            NaiveDate::from_ymd_opt(2026, 1, 16).unwrap()
        );
        assert_eq!(
            add_1_day(NaiveDate::from_ymd_opt(2026, 1, 31).unwrap()).unwrap(),
            NaiveDate::from_ymd_opt(2026, 2, 1).unwrap()
        );
        assert_eq!(
            add_1_day(NaiveDate::from_ymd_opt(2026, 12, 31).unwrap()).unwrap(),
            NaiveDate::from_ymd_opt(2027, 1, 1).unwrap()
        );
    }

    #[test]
    fn add_1_day_handles_leap_day() {
        assert_eq!(
            add_1_day(NaiveDate::from_ymd_opt(2024, 2, 28).unwrap()).unwrap(),
            NaiveDate::from_ymd_opt(2024, 2, 29).unwrap()
        );
        assert_eq!(
            add_1_day(NaiveDate::from_ymd_opt(2024, 2, 29).unwrap()).unwrap(),
            NaiveDate::from_ymd_opt(2024, 3, 1).unwrap()
        );
    }

    #[test]
    fn add_1_day_errors_at_max_date() {
        let date = NaiveDate::MAX;
        let err = add_1_day(date).unwrap_err();
        assert!(matches!(err, HrdfError::FailedToAddDays(d, 1) if d == date));
    }

    #[test]
    fn sub_1_day_rolls_back_month_and_year() {
        assert_eq!(
            sub_1_day(NaiveDate::from_ymd_opt(2026, 3, 1).unwrap()).unwrap(),
            NaiveDate::from_ymd_opt(2026, 2, 28).unwrap()
        );
        assert_eq!(
            sub_1_day(NaiveDate::from_ymd_opt(2026, 1, 1).unwrap()).unwrap(),
            NaiveDate::from_ymd_opt(2025, 12, 31).unwrap()
        );
    }

    #[test]
    fn sub_1_day_errors_at_min_date() {
        let date = NaiveDate::MIN;
        let err = sub_1_day(date).unwrap_err();
        assert!(matches!(err, HrdfError::FailedToSubDays(d, 1) if d == date));
    }

    #[test]
    fn count_days_between_two_dates_counts_inclusive() {
        let date = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        assert_eq!(count_days_between_two_dates(date, date).unwrap(), 1);

        let end = NaiveDate::from_ymd_opt(2024, 1, 10).unwrap();
        assert_eq!(count_days_between_two_dates(date, end).unwrap(), 10);
    }

    #[test]
    fn count_days_between_two_dates_spans_leap_day() {
        let start = NaiveDate::from_ymd_opt(2024, 2, 27).unwrap();
        let end = NaiveDate::from_ymd_opt(2024, 3, 1).unwrap();
        assert_eq!(count_days_between_two_dates(start, end).unwrap(), 4);
    }

    #[test]
    fn count_days_between_two_dates_errors_when_end_is_before_start() {
        let start = NaiveDate::from_ymd_opt(2024, 1, 10).unwrap();
        let end = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        assert!(matches!(
            count_days_between_two_dates(start, end),
            Err(HrdfError::OutOfRangeDate(date)) if date == end
        ));
    }

    #[test]
    fn create_time_builds_valid_time() {
        assert_eq!(
            create_time(8, 30).unwrap(),
            NaiveTime::from_hms_opt(8, 30, 0).unwrap()
        );
        assert_eq!(
            create_time(23, 59).unwrap(),
            NaiveTime::from_hms_opt(23, 59, 0).unwrap()
        );
    }

    #[test]
    fn create_time_rejects_invalid_input() {
        assert!(matches!(
            create_time(24, 0).unwrap_err(),
            ParsingError::UnableToBuildTime(24, 0, 0)
        ));
        assert!(matches!(
            create_time(0, 60).unwrap_err(),
            ParsingError::UnableToBuildTime(0, 60, 0)
        ));
    }

    #[test]
    fn create_time_from_value_splits_hhmm() {
        assert_eq!(
            create_time_from_value(830).unwrap(),
            NaiveTime::from_hms_opt(8, 30, 0).unwrap()
        );
        assert_eq!(
            create_time_from_value(2359).unwrap(),
            NaiveTime::from_hms_opt(23, 59, 0).unwrap()
        );
    }

    #[test]
    fn timetable_start_date_and_end_date_read_metadata() {
        let mut data = FxHashMap::default();
        data.insert(
            1,
            TimetableMetadataEntry::new(1, "start_date".to_string(), "2024-12-15".to_string()),
        );
        data.insert(
            2,
            TimetableMetadataEntry::new(2, "end_date".to_string(), "2025-12-13".to_string()),
        );
        let storage = ResourceStorage::new(data);

        assert_eq!(
            timetable_start_date(&storage).unwrap(),
            NaiveDate::from_ymd_opt(2024, 12, 15).unwrap()
        );
        assert_eq!(
            timetable_end_date(&storage).unwrap(),
            NaiveDate::from_ymd_opt(2025, 12, 13).unwrap()
        );
    }

    #[test]
    fn timetable_start_date_and_end_date_error_when_missing() {
        let storage = ResourceStorage::<TimetableMetadataEntry>::new(FxHashMap::default());

        assert!(matches!(
            timetable_start_date(&storage).unwrap_err(),
            HrdfError::MissingStartDate
        ));
        assert!(matches!(
            timetable_end_date(&storage).unwrap_err(),
            HrdfError::MissingEndDate
        ));
    }
}
