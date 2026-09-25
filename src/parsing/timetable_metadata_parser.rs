use std::path::Path;

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
    error::{HResult, HrdfError},
    models::{Model, TimetableMetadataEntry},
    parsing::{
        error::{PResult, ParsingError},
        helpers::read_lines,
    },
    storage::ResourceStorage,
    utils::{AutoIncrement, timetable_end_date, timetable_start_date},
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
                // `parse_line` reports the offending line; nom drops any message given here.
                NaiveDate::from_ymd_opt(year, month, day).ok_or(())
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

/// The first two values of ECKDATEN (`start_date` and `end_date`) are dates.
const NUM_DATE_KEYS: usize = 2;

fn key_at(keys: &[&str], index: usize, value: &str) -> PResult<String> {
    keys.get(index).map(|key| (*key).to_owned()).ok_or_else(|| {
        ParsingError::Unknown(format!(
            "ECKDATEN has more than {} values, unexpected value {value:?}",
            keys.len()
        ))
    })
}

fn parse_line(
    line: &str,
    data: &mut FxHashMap<i32, TimetableMetadataEntry>,
    keys: &[&str],
    index: &mut usize,
    auto_increment: &AutoIncrement,
) -> PResult<()> {
    let (_, res) = alt((date_combinator, info_combinator)).parse(line)?;
    match res {
        InfoLines::Date(d) => {
            let key = key_at(keys, *index, line)?;
            let tt = TimetableMetadataEntry::new(auto_increment.next(), key, d.to_string());
            data.insert(tt.id(), tt);
            *index += 1;
        }
        InfoLines::MetaData(mt) => {
            if *index < NUM_DATE_KEYS {
                // The start and end dates must be valid dates; reject anything else here, where the
                // offending line can still be reported.
                return Err(ParsingError::Unknown(format!(
                    "Expected a date (DD.MM.YYYY) for {}, got {line:?}",
                    keys[*index]
                )));
            }
            for t in mt {
                let key = key_at(keys, *index, &t)?;
                let tt = TimetableMetadataEntry::new(auto_increment.next(), key, t);
                data.insert(tt.id(), tt);
                *index += 1;
            }
        }
    }
    Ok(())
}

pub fn parse(path: &Path) -> HResult<ResourceStorage<TimetableMetadataEntry>> {
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
    let file = path.join("ECKDATEN");
    let time_table = read_lines(&file, 0)?;
    time_table
        .into_iter()
        .enumerate()
        .filter(|(_, line)| !line.trim().is_empty())
        .try_for_each(|(line_number, line)| {
            parse_line(&line, &mut data, &keys, &mut index, &auto_increment).map_err(|e| {
                HrdfError::Parsing {
                    error: e,
                    file: String::from(file.to_string_lossy()),
                    line,
                    line_number,
                }
            })
        })?;

    let storage = ResourceStorage::new(data);
    // `storage.rs` counts the days between the two dates; an end date before the start date is invalid.
    if let (Ok(start), Ok(end)) = (timetable_start_date(&storage), timetable_end_date(&storage))
        && end < start
    {
        return Err(HrdfError::OutOfRangeDate(end));
    }
    Ok(storage)
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
    #[should_panic]
    fn test_date_combinator_invalid_date() {
        let input = "32.13.2024"; // Invalid day and month
        date_combinator(input).unwrap();
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

    const KEYS: [&str; 6] = [
        "start_date",
        "end_date",
        "name",
        "created_at",
        "version",
        "provider",
    ];

    fn parse_lines(lines: &[&str]) -> PResult<FxHashMap<i32, TimetableMetadataEntry>> {
        let mut data = FxHashMap::default();
        let mut index = 0;
        let auto_increment = AutoIncrement::new();
        for line in lines {
            parse_line(line, &mut data, &KEYS, &mut index, &auto_increment)?;
        }
        Ok(data)
    }

    fn value_of(data: &FxHashMap<i32, TimetableMetadataEntry>, key: &str) -> String {
        data.values()
            .find(|entry| entry.key() == key)
            .unwrap_or_else(|| panic!("missing key {key}"))
            .value()
            .to_string()
    }

    #[test]
    fn parse_lines_reads_a_real_eckdaten() {
        let data = parse_lines(&[
            "14.12.2025",
            "12.12.2026",
            "Fahrplan 2026$22.09.2026 21:49:55$5.40.41$INFO+",
        ])
        .unwrap();
        assert_eq!(data.len(), 6);
        assert_eq!(value_of(&data, "start_date"), "2025-12-14");
        assert_eq!(value_of(&data, "end_date"), "2026-12-12");
        assert_eq!(value_of(&data, "provider"), "INFO+");
    }

    #[test]
    fn invalid_start_date_is_an_error_naming_the_value() {
        let err = parse_lines(&["31.02.2024"]).unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("start_date"), "unexpected message: {msg}");
        assert!(msg.contains("31.02.2024"), "unexpected message: {msg}");
    }

    #[test]
    fn invalid_end_date_is_an_error_naming_the_value() {
        let err = parse_lines(&["14.12.2025", "not a date"]).unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("end_date"), "unexpected message: {msg}");
        assert!(msg.contains("not a date"), "unexpected message: {msg}");
    }

    #[test]
    fn more_values_than_keys_is_an_error_not_a_panic() {
        let result = std::panic::catch_unwind(|| {
            parse_lines(&[
                "14.12.2025",
                "12.12.2026",
                "name$created_at$version$provider",
                "extra",
            ])
        });
        let err = result
            .expect("parse_line must not panic")
            .unwrap_err()
            .to_string();
        assert!(err.contains("extra"), "unexpected message: {err}");
    }

    fn write_eckdaten(name: &str, content: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("hrdf_{name}_{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("ECKDATEN"), content).unwrap();
        dir
    }

    #[test]
    fn parse_accepts_a_valid_file() {
        let dir = write_eckdaten(
            "valid",
            "14.12.2025\n12.12.2026\nFahrplan 2026$22.09.2026 21:49:55$5.40.41$INFO+\n",
        );
        let data = parse(&dir).unwrap();
        std::fs::remove_dir_all(&dir).ok();
        assert_eq!(data.data().len(), 6);
    }

    #[test]
    fn parse_accepts_windows_line_endings() {
        // The 2025 archive's ECKDATEN uses CRLF.
        let dir = write_eckdaten(
            "crlf",
            "15.12.2024\r\n13.12.2025\r\nFahrplan 2025$05.12.2025 20:05:36$5.40.41$INFO+\r\n",
        );
        let data = parse(&dir).unwrap();
        std::fs::remove_dir_all(&dir).ok();
        assert_eq!(data.data().len(), 6);
        let provider = data
            .data()
            .values()
            .find(|e| e.key() == "provider")
            .unwrap();
        assert_eq!(provider.value(), "INFO+");
    }

    #[test]
    fn parse_rejects_an_end_date_before_the_start_date() {
        let dir = write_eckdaten(
            "reversed",
            "12.12.2026\n14.12.2025\nFahrplan 2026$22.09.2026 21:49:55$5.40.41$INFO+\n",
        );
        let result = parse(&dir);
        std::fs::remove_dir_all(&dir).ok();
        assert!(matches!(result, Err(HrdfError::OutOfRangeDate(_))));
    }
}
