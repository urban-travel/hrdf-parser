/// # ECKDATEN file
///
/// Life of the timetable
///
/// The timetable data is valid for the defined period. The duration usually corresponds to that of the timetable period
///
/// Can be read in decoupled from other data.
///
///
/// 1 file(s).
/// File(s) read by the parser:
/// ECKDATEN
use std::error::Error;

use chrono::NaiveDate;
use nom::{
    IResult, Parser,
    branch::alt,
    bytes::{complete::is_not, tag},
    character::complete::{char, i32, u32},
    combinator::{map, map_res},
    multi::separated_list1,
    sequence::preceded,
};
use rustc_hash::FxHashMap;

use crate::{
    models::{Model, TimetableMetadataEntry},
    parsing::helpers::read_lines,
    storage::ResourceStorage,
    utils::AutoIncrement,
};

enum InfoLines {
    Date(NaiveDate),
    MetaData(Vec<String>),
}

fn date_combinator(input: &str) -> IResult<&str, InfoLines> {
    map(
        map_res(
            (u32, preceded(tag("."), u32), preceded(tag("."), i32)),
            |(day, month, year)| {
                NaiveDate::from_ymd_opt(year, month, day)
                    .ok_or("Unable to parse date {day}, {month}, {year}")
            },
        ),
        InfoLines::Date,
    )
    .parse(input)
}

fn info_combinator(input: &str) -> IResult<&str, InfoLines> {
    map(
        separated_list1(char('$'), map(is_not("$"), String::from)),
        InfoLines::MetaData,
    )
    .parse(input)
}

pub fn parse(path: &str) -> Result<ResourceStorage<TimetableMetadataEntry>, Box<dyn Error>> {
    log::info!("Parsing ECKDATEN...");
    let auto_increment = AutoIncrement::new();
    let keys = [
        "start_date",
        "end_date",
        "name",
        "created_at",
        "version",
        "provider",
    ];
    let mut index = 0;
    let mut data = FxHashMap::default();
    read_lines(&format!("{path}/ECKDATEN"), 0)?
        .into_iter()
        .filter(|line| !line.trim().is_empty())
        .try_for_each(|line| {
            let (_, res) = alt((date_combinator, info_combinator))
                .parse(&line)
                .map_err(|e| format!("Error: {e}, for line: {line}"))?;
            match res {
                InfoLines::Date(d) => {
                    let tt = TimetableMetadataEntry::new(
                        auto_increment.next(),
                        keys[index].to_owned(),
                        d.to_string(),
                    );
                    data.insert(tt.id(), tt);
                    index += 1;
                }
                InfoLines::MetaData(mt) => {
                    for t in mt {
                        let tt = TimetableMetadataEntry::new(
                            auto_increment.next(),
                            keys[index].to_owned(),
                            t,
                        );
                        data.insert(tt.id(), tt);
                        index += 1;
                    }
                }
            }
            Ok::<(), Box<dyn Error>>(())
        })?;

    Ok(ResourceStorage::new(data))
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Datelike;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_date_combinator_valid() {
        let input = "11.12.2023";
        let result = date_combinator(input);
        assert!(result.is_ok());
        let (_, info_line) = result.unwrap();
        match info_line {
            InfoLines::Date(date) => {
                assert_eq!(date.day(), 11);
                assert_eq!(date.month(), 12);
                assert_eq!(date.year(), 2023);
            }
            _ => panic!("Expected Date variant"),
        }
    }

    #[test]
    fn test_date_combinator_start_of_year() {
        let input = "1.1.2024";
        let result = date_combinator(input);
        assert!(result.is_ok());
        let (_, info_line) = result.unwrap();
        match info_line {
            InfoLines::Date(date) => {
                assert_eq!(date.day(), 1);
                assert_eq!(date.month(), 1);
                assert_eq!(date.year(), 2024);
            }
            _ => panic!("Expected Date variant"),
        }
    }

    #[test]
    fn test_date_combinator_end_of_year() {
        let input = "31.12.2024";
        let result = date_combinator(input);
        assert!(result.is_ok());
        let (_, info_line) = result.unwrap();
        match info_line {
            InfoLines::Date(date) => {
                assert_eq!(date.day(), 31);
                assert_eq!(date.month(), 12);
                assert_eq!(date.year(), 2024);
            }
            _ => panic!("Expected Date variant"),
        }
    }

    #[test]
    fn test_info_combinator_single_value() {
        let input = "Timetable 2024";
        let result = info_combinator(input);
        assert!(result.is_ok());
        let (_, info_line) = result.unwrap();
        match info_line {
            InfoLines::MetaData(metadata) => {
                assert_eq!(metadata.len(), 1);
                assert_eq!(metadata[0], "Timetable 2024");
            }
            _ => panic!("Expected MetaData variant"),
        }
    }

    #[test]
    fn test_info_combinator_multiple_values() {
        let input = "Value1$Value2$Value3";
        let result = info_combinator(input);
        assert!(result.is_ok());
        let (_, info_line) = result.unwrap();
        match info_line {
            InfoLines::MetaData(metadata) => {
                assert_eq!(metadata.len(), 3);
                assert_eq!(metadata[0], "Value1");
                assert_eq!(metadata[1], "Value2");
                assert_eq!(metadata[2], "Value3");
            }
            _ => panic!("Expected MetaData variant"),
        }
    }

    #[test]
    fn test_info_combinator_with_spaces() {
        let input = "SBB CFF FFS$OpenTransport";
        let result = info_combinator(input);
        assert!(result.is_ok());
        let (_, info_line) = result.unwrap();
        match info_line {
            InfoLines::MetaData(metadata) => {
                assert_eq!(metadata.len(), 2);
                assert_eq!(metadata[0], "SBB CFF FFS");
                assert_eq!(metadata[1], "OpenTransport");
            }
            _ => panic!("Expected MetaData variant"),
        }
    }

    #[test]
    fn test_info_combinator_consecutive_delimiters() {
        let input = "Start$$End";
        let result = info_combinator(input);
        assert!(result.is_ok());
        let (_, info_line) = result.unwrap();
        match info_line {
            InfoLines::MetaData(metadata) => {
                // Parser behavior with consecutive delimiters
                assert!(!metadata.is_empty());
                assert_eq!(metadata[0], "Start");
            }
            _ => panic!("Expected MetaData variant"),
        }
    }

    #[test]
    fn test_date_combinator_single_digit_day() {
        let input = "5.6.2024";
        let result = date_combinator(input);
        assert!(result.is_ok());
        let (_, info_line) = result.unwrap();
        match info_line {
            InfoLines::Date(date) => {
                assert_eq!(date.day(), 5);
                assert_eq!(date.month(), 6);
                assert_eq!(date.year(), 2024);
            }
            _ => panic!("Expected Date variant"),
        }
    }

    #[test]
    fn test_date_combinator_leap_year() {
        let input = "29.2.2024";
        let result = date_combinator(input);
        assert!(result.is_ok());
        let (_, info_line) = result.unwrap();
        match info_line {
            InfoLines::Date(date) => {
                assert_eq!(date.day(), 29);
                assert_eq!(date.month(), 2);
                assert_eq!(date.year(), 2024);
            }
            _ => panic!("Expected Date variant"),
        }
    }

    #[test]
    fn test_date_combinator_invalid_date() {
        let input = "32.13.2024"; // Invalid day and month
        let result = date_combinator(input);
        // Parser should fail for invalid dates
        assert!(result.is_err());
    }

    #[test]
    fn test_info_combinator_numeric_values() {
        let input = "5.40.41";
        let result = info_combinator(input);
        assert!(result.is_ok());
        let (_, info_line) = result.unwrap();
        match info_line {
            InfoLines::MetaData(metadata) => {
                assert_eq!(metadata.len(), 1);
                assert_eq!(metadata[0], "5.40.41");
            }
            _ => panic!("Expected MetaData variant"),
        }
    }
}
