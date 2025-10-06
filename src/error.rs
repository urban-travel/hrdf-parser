use crate::parsing::error::ParsingError;
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
}

pub type HResult<T> = Result<T, HrdfError>;
