use crate::{JourneyError, Version, parsing::error::ParsingError};
use bincode::error::{DecodeError, EncodeError};
use chrono::NaiveDate;
use thiserror::Error;
use zip::result::ZipError;

#[derive(Debug, Error)]
pub enum HrdfError {
    #[error("File {file}, at line {line_number}: {line}. Parsing error: {error:?}")]
    Parsing {
        error: ParsingError,
        file: String,
        line: String,
        line_number: usize,
    },
    #[error("Io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Missing key \"start_date\"")]
    MissingStartDate,
    #[error("Missing key \"end_date\"")]
    MissingEndDate,
    #[error("JourneyError {0}")]
    Journey(#[from] JourneyError),
    #[error("Failed to add {1} days to {0}")]
    FailedToAddDays(NaiveDate, u64),
    #[error("Failed to subtract {1} days to {0}")]
    FailedToSubDays(NaiveDate, u64),
    #[error("BitFieldId {0} not found")]
    BitFieldIdNotFound(i32),
    #[error("Failed to read cache: {0}")]
    ReadCache(#[from] DecodeError),
    #[error("Failed to write cache: {0}")]
    WriteCacher(#[from] EncodeError),
    #[error("Failed decompress data: {0}")]
    Decompress(#[from] ZipError),
    #[error("Failed to download data: {0}")]
    Download(#[from] reqwest::Error),
    #[error("Missing stop id: {0}")]
    MissingStopId(i32),
    #[error("Missing departure time at index: {0}")]
    MissingDepartureTime(usize),
    #[error("Missing arrival time at index: {0}")]
    MissingArrivalTime(usize),
    #[error("Missing route")]
    MissingRoute,
    #[error("Out of rage date: {0}")]
    OutOfRangeDate(NaiveDate),
    #[error("Invalid year provided")]
    InvalidYear,
    #[error("Version not supported: {0}")]
    SupportedVersion(Version),
}

pub type HResult<T> = Result<T, HrdfError>;
