mod attribute_parser;
mod bit_field_parser;
mod direction_parser;
mod exchange_administration_parser;
mod exchange_line_parser;
mod exchange_trip_parser;
mod holiday_parser;
mod information_text_parser;
mod line_parser;
mod platform_parser;
mod stop_connection_parser;
mod stop_parser;
mod through_service_parser;
mod timetable_metadata_parser;
mod transport_company_parser;
mod transport_type_parser;
mod trip_parser;

pub use attribute_parser::parse as load_attributes;
pub use bit_field_parser::parse as load_bit_fields;
pub use direction_parser::parse as load_directions;
pub use exchange_administration_parser::parse as load_exchange_times_administration;
pub use exchange_line_parser::parse as load_exchange_times_line;
pub use exchange_trip_parser::parse as load_exchange_times_trip;
pub use holiday_parser::parse as load_holidays;
pub use information_text_parser::parse as load_information_texts;
pub use line_parser::parse as load_lines;
pub use platform_parser::parse as load_platforms;
pub use stop_connection_parser::parse as load_stop_connections;
pub use stop_parser::parse as load_stops;
pub use through_service_parser::parse as load_through_service;
pub use timetable_metadata_parser::parse as load_timetable_metadata;
pub use transport_company_parser::parse as load_transport_companies;
pub use transport_type_parser::parse as load_transport_types;
pub use trip_parser::parse as load_trips;

use std::{
    error::Error,
    fs::File,
    io::{self, Read, Seek},
};

use regex::Regex;

pub enum ExpectedType {
    Float,
    Integer16,
    Integer32,
    String,
    OptionInteger32,
}

#[derive(Debug)]
pub enum ParsedValue {
    Float(f64),
    Integer16(i16),
    Integer32(i32),
    String(String),
    OptionInteger16(Option<i16>),
    OptionInteger32(Option<i32>),
}

impl From<ParsedValue> for f64 {
    fn from(value: ParsedValue) -> Self {
        match value {
            ParsedValue::Float(x) => x,
            // If this error occurs, it's due to a typing error and it's the developer's fault.
            _ => panic!("Failed to convert ParsedValue to f64."),
        }
    }
}

impl From<ParsedValue> for i16 {
    fn from(value: ParsedValue) -> Self {
        match value {
            ParsedValue::Integer16(x) => x,
            // If this error occurs, it's due to a typing error and it's the developer's fault.
            _ => panic!("Failed to convert ParsedValue to i16."),
        }
    }
}

impl From<ParsedValue> for i32 {
    fn from(value: ParsedValue) -> Self {
        match value {
            ParsedValue::Integer32(x) => x,
            // If this error occurs, it's due to a typing error and it's the developer's fault.
            _ => panic!("Failed to convert ParsedValue to i32."),
        }
    }
}

impl From<ParsedValue> for String {
    fn from(value: ParsedValue) -> Self {
        match value {
            ParsedValue::String(x) => x,
            // If this error occurs, it's due to a typing error and it's the developer's fault.
            _ => panic!("Failed to convert ParsedValue to String."),
        }
    }
}

impl From<ParsedValue> for Option<i32> {
    fn from(value: ParsedValue) -> Self {
        match value {
            ParsedValue::OptionInteger32(x) => x,
            // If this error occurs, it's due to a typing error and it's the developer's fault.
            _ => panic!("Failed to convert ParsedValue to Option<i32>."),
        }
    }
}

// ------------------------------------------------------------------------------------------------
// --- RowMatcher
// ------------------------------------------------------------------------------------------------

pub trait RowMatcher {
    fn match_row(&self, row: &str) -> bool;
}

// ------------------------------------------------------------------------------------------------
// --- FastRowMatcher
// ------------------------------------------------------------------------------------------------

pub struct FastRowMatcher {
    // 1-based indexing
    start: usize,
    length: usize,
    value: String,
    should_equal_value: bool,
}

impl FastRowMatcher {
    pub fn new(start: usize, length: usize, value: &str, should_equal_value: bool) -> Self {
        Self {
            start,
            length,
            value: value.to_string(),
            should_equal_value,
        }
    }
}

impl RowMatcher for FastRowMatcher {
    fn match_row(&self, row: &str) -> bool {
        // Info: if the start index is after characters longer than 1 byte, the code will not function correctly.
        // This is not a problem, as the start index is always at the beginning of the
        // string, where there is no character requiring more than 1 byte.
        let start = self.start - 1;
        let target_value = &row[start..(start + self.length)];
        self.should_equal_value == (target_value == self.value)
    }
}

// ------------------------------------------------------------------------------------------------
// --- AdvancedRowMatcher
// ------------------------------------------------------------------------------------------------

pub struct AdvancedRowMatcher {
    re: Regex,
}

impl AdvancedRowMatcher {
    pub fn new(re: &str) -> Result<Self, Box<dyn Error>> {
        Ok(Self {
            re: Regex::new(re)?,
        })
    }
}

impl RowMatcher for AdvancedRowMatcher {
    fn match_row(&self, row: &str) -> bool {
        self.re.is_match(row)
    }
}

// ------------------------------------------------------------------------------------------------
// --- ColumnDefinition
// ------------------------------------------------------------------------------------------------

pub struct ColumnDefinition {
    // 1-based indexing
    start: usize,
    // 1-based indexing
    stop: isize,
    expected_type: ExpectedType,
}

impl ColumnDefinition {
    pub fn new(start: usize, stop: isize, expected_type: ExpectedType) -> Self {
        Self {
            start,
            stop,
            expected_type,
        }
    }
}

// ------------------------------------------------------------------------------------------------
// --- RowDefinition
// ------------------------------------------------------------------------------------------------

type RowConfiguration = Vec<ColumnDefinition>;

pub struct RowDefinition {
    id: i32,
    row_matcher: Option<Box<dyn RowMatcher>>,
    row_configuration: RowConfiguration,
}

impl RowDefinition {
    pub fn new(
        id: i32,
        row_matcher: Box<dyn RowMatcher>,
        row_configuration: RowConfiguration,
    ) -> Self {
        Self {
            id,
            row_matcher: Some(row_matcher),
            row_configuration,
        }
    }
}

impl From<RowConfiguration> for RowDefinition {
    fn from(row_configuration: RowConfiguration) -> Self {
        Self {
            id: 1,
            row_matcher: None,
            row_configuration,
        }
    }
}

// ------------------------------------------------------------------------------------------------
// --- RowParser
// ------------------------------------------------------------------------------------------------

// (RowDefinition.id, number of bytes read, values parsed from the row)
type ParsedRow = (i32, u64, Vec<ParsedValue>);

pub struct RowParser {
    row_definitions: Vec<RowDefinition>,
}

impl RowParser {
    pub fn new(row_definitions: Vec<RowDefinition>) -> Self {
        Self { row_definitions }
    }

    fn parse(&self, row: &str) -> Result<ParsedRow, Box<dyn Error>> {
        let row_definition = self.row_definition(row)?;
        // 2 bytes for \r\n
        let bytes_read = row.len() as u64 + 2;
        let values = row_definition
            .row_configuration
            .iter()
            .map(|column_definition| {
                let start = column_definition.start - 1;
                let stop = if column_definition.stop == -1 {
                    row.chars().count()
                } else {
                    column_definition.stop as usize
                };

                // Converts start/stop columns into real indexes.
                let start = row
                    .char_indices()
                    .map(|(i, _)| i)
                    .nth(start)
                    .ok_or("The start column is out of range.")?;
                let stop = if let Some(i) = row.char_indices().map(|(i, _)| i).nth(stop) {
                    i
                } else {
                    row.len()
                };

                let value = row[start..stop].trim();

                let result = match column_definition.expected_type {
                    ExpectedType::Float => ParsedValue::Float(value.parse()?),
                    ExpectedType::Integer16 => ParsedValue::Integer16(value.parse()?),
                    ExpectedType::Integer32 => ParsedValue::Integer32(value.parse()?),
                    // The "value" variable is a &str, so it's impossible to fail by converting it to a String.
                    ExpectedType::String => ParsedValue::String(value.to_owned()),
                    ExpectedType::OptionInteger32 => {
                        ParsedValue::OptionInteger32(value.parse().ok())
                    }
                };
                Ok::<ParsedValue, Box<dyn Error>>(result)
            })
            .collect::<Result<Vec<_>, Box<dyn Error>>>()?;
        Ok((row_definition.id, bytes_read, values))
    }

    fn row_definition(&self, row: &str) -> Result<&RowDefinition, Box<dyn Error>> {
        if self.row_definitions.len() == 1 {
            return Ok(&self.row_definitions[0]);
        }

        let matched_row_definition = self
            .row_definitions
            .iter()
            // unwrap: "row_matcher" is guaranteed to always have a value when there are multiple row definitions.
            .find(|row_definition| row_definition.row_matcher.as_ref().unwrap().match_row(row));

        return matched_row_definition
            .ok_or(format!("This type of row is unknown:\n{}", row).into());
    }
}

// ------------------------------------------------------------------------------------------------
// --- FileParser
// ------------------------------------------------------------------------------------------------

pub struct FileParser {
    rows: Vec<String>,
    row_parser: RowParser,
}

impl FileParser {
    pub fn new(path: &str, row_parser: RowParser) -> io::Result<Self> {
        Self::new_with_bytes_offset(path, row_parser, 0)
    }

    pub fn new_with_bytes_offset(
        path: &str,
        row_parser: RowParser,
        bytes_offset: u64,
    ) -> io::Result<Self> {
        let rows = Self::read_lines(path, bytes_offset)?;
        Ok(Self { rows, row_parser })
    }

    fn read_lines(path: &str, bytes_offset: u64) -> io::Result<Vec<String>> {
        let mut file = File::open(path)?;
        file.seek(io::SeekFrom::Start(bytes_offset))?;
        let mut reader = io::BufReader::new(file);
        let mut contents = String::new();
        reader.read_to_string(&mut contents)?;
        let lines = contents.lines().map(String::from).collect();
        Ok(lines)
    }

    pub fn parse(&self) -> ParsedRowIterator {
        ParsedRowIterator {
            rows_iter: self.rows.iter(),
            row_parser: &self.row_parser,
        }
    }
}

// ------------------------------------------------------------------------------------------------
// --- ParsedRowIterator
// ------------------------------------------------------------------------------------------------

pub struct ParsedRowIterator<'a> {
    rows_iter: std::slice::Iter<'a, String>,
    row_parser: &'a RowParser,
}

impl Iterator for ParsedRowIterator<'_> {
    type Item = Result<ParsedRow, Box<dyn Error>>;

    fn next(&mut self) -> Option<Self::Item> {
        self.rows_iter
            .by_ref()
            .skip_while(|row| row.trim().is_empty())
            .next()
            .map(|row| self.row_parser.parse(row))
    }
}
