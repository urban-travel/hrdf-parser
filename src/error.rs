use std::{
    io,
    num::{ParseFloatError, ParseIntError},
};

use zip::result::ZipError;

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
#[error(transparent)]
pub struct Error(#[from] ErrorKind);

macro_rules! impl_from_error {
    ($( $type:ty ),* $(,)? ) => {
        $(
        impl From<$type> for Error {
            fn from(error: $type) -> Self {
                Self(error.into())
            }
        }
        )*
    };
}

impl_from_error!(
    io::Error,
    reqwest::Error,
    ZipError,
    bincode::error::EncodeError,
    bincode::error::DecodeError,
    regex::Error,
    ParseIntError,
    ParseFloatError,
    strum::ParseError,
    chrono::ParseError,
);

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub(crate) enum ErrorKind {
    #[error("read/write error")]
    Io(#[from] io::Error),
    #[error("network error")]
    Network(#[from] reqwest::Error),
    #[error("malformed zip archive")]
    Zip(#[from] ZipError),
    #[error("cache creation error")]
    BincodeEcode(#[from] bincode::error::EncodeError),
    #[error("cache parsing error")]
    BincodeDecode(#[from] bincode::error::DecodeError),
    #[error("malformed regex")]
    Regex(#[from] regex::Error),

    #[error("invalid interger")]
    InvalidInt(#[from] ParseIntError),
    #[error("invalid float")]
    InvalidFloat(#[from] ParseFloatError),
    #[error("enum value invalid")]
    Strum(#[from] strum::ParseError),
    #[error("date or time value invalid")]
    DateTime(#[from] chrono::ParseError),

    #[error("invalid hexadecimal digit")]
    InvalidHexaDigit,

    #[error("Unknown legacy {0} ID: {1:?}")]
    UnknownLegacyId(&'static str, String),
    #[error("Unknown legacy {name} ID: {id} #{index}")]
    UnknownLegacyIdIndex {
        name: &'static str,
        id: i32,
        index: i32,
    },
    #[error("Unknown legacy {name} ID: {id} admin={admin:?}")]
    UnknownLegacyIdAdmin {
        name: &'static str,
        id: i32,
        admin: String,
    },

    #[error("Unknown ID {0}")]
    UnknownId(i32),

    #[error("Missing value part")]
    MissingValuePart,
    #[error("Missing stop name (standard name is mandatory)")]
    MissingStopName,
    #[error("Missing designation")]
    MissingDesignation,

    #[error("Type {typ} row missing.")]
    RowMissing { typ: &'static str },
    #[error("Entry of type {typ} missing.")]
    EntryMissing { typ: &'static str },

    #[error("Key {name:?} missing.")]
    KeyMissing { name: &'static str },

    #[error("The start column is out of range.")]
    TheStartColumnIsOutOfRange,
    #[error("This type of row is unknown: {row:?}")]
    UnknownRowType { row: String },
}
