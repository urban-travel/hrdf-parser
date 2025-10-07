use crate::{JourneyError, parsing::error::ParsingError};
use chrono::NaiveDate;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum HrdfError {
    #[error("File {file}, at line {line_number}: {line}. Parsing error: {error}")]
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
    #[error("BitFieldId {0} not found")]
    BitFieldIdNotFound(i32),
}

pub type HResult<T> = Result<T, HrdfError>;
