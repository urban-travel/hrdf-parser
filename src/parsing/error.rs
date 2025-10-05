use thiserror::Error;

#[derive(Debug, Error)]
pub enum ParsingError {
    #[error("Nom parsing error: {0}")]
    ParseError(#[from] nom::Err<nom::error::Error<String>>),
    #[error("Language error: {0}")]
    Language(#[from] strum::ParseError),
    #[error("Unkown id: {0}")]
    UnknownId(String),
    #[error("Io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Unkown error: {0}")]
    Unknown(String),
    #[error("Invalid hex digit {0}")]
    InvalidHexDigit(char),
    #[error("Missing line type")]
    MissingLineType,
    #[error("Error default exchange time not defined")]
    MissingDefaultExchangeTime,
}

impl From<nom::Err<nom::error::Error<&str>>> for ParsingError {
    fn from(value: nom::Err<nom::error::Error<&str>>) -> Self {
        ParsingError::ParseError(value.map_input(String::from))
    }
}

impl From<&str> for ParsingError {
    fn from(value: &str) -> Self {
        ParsingError::Unknown(value.to_string())
    }
}

pub type PResult<T> = Result<T, ParsingError>;
