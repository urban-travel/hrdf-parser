// 1 file(s).
// File(s) read by the parser:
// ECKDATEN
use std::error::Error;

use chrono::NaiveDate;
use nom::{
    Parser,
    branch::alt,
    bytes::{
        complete::{take_till1, take_until},
        tag,
    },
    character::complete::{alphanumeric1, digit1, i32, space1, u32},
    combinator::{map, map_res},
    multi::{many0, separated_list1},
    sequence::preceded,
};
use rustc_hash::FxHashMap;

use crate::{
    models::{Model, TimetableMetadataEntry},
    parsing::{
        AdvancedRowMatcher, ColumnDefinition, ExpectedType, FastRowMatcher, FileParser,
        ParsedValue, RowDefinition, RowParser, helpers::read_lines,
    },
    storage::ResourceStorage,
    utils::AutoIncrement,
};

enum InfoLines {
    Date(NaiveDate),
    MetaData([String; 4]),
}

fn date_combinator<'a>()
-> impl Parser<&'a str, Output = InfoLines, Error = nom::error::Error<&'a str>> {
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
}

fn info_combinator<'a>()
-> impl Parser<&'a str, Output = InfoLines, Error = nom::error::Error<&'a str>> {
    map(
        (
            map(take_until("$"), String::from),
            separated_list1(tag("$"), map(alphanumeric1, String::from)),
        ),
        |(name, other_data)| {
            InfoLines::MetaData([
                name,
                other_data[0].to_owned(),
                other_data[1].to_owned(),
                other_data[2].to_owned(),
            ])
        },
    )
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
            let (_, res) = alt((date_combinator(), info_combinator()))
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

pub fn old_parse(path: &str) -> Result<ResourceStorage<TimetableMetadataEntry>, Box<dyn Error>> {
    log::info!("Parsing ECKDATEN...");
    const ROW_A: i32 = 1;
    const ROW_B: i32 = 2;

    #[rustfmt::skip]
    let row_parser = RowParser::new(vec![
        // This row contains the period start/end date in which timetables are effective.
        RowDefinition::new(ROW_A, Box::new(AdvancedRowMatcher::new(r"^[0-9]{2}.[0-9]{2}.[0-9]{4}$")?), vec![
            ColumnDefinition::new(1, 10, ExpectedType::String),
        ]),
        // This row contains the name, the creation date, the version and the provider of the timetable.
        RowDefinition::new(ROW_B, Box::new(FastRowMatcher::new(1, 0, "", true)), vec![
            ColumnDefinition::new(1, -1, ExpectedType::String),
        ]),
    ]);
    let parser = FileParser::new(&format!("{path}/ECKDATEN"), row_parser)?;

    let mut data: Vec<ParsedValue> = parser
        .parse()
        .map(|x| x.map(|(_, _, mut values)| values.remove(0)))
        .collect::<Result<Vec<_>, _>>()?;

    let start_date: String = data.remove(0).into();
    let end_date: String = data.remove(0).into();
    let other_data: String = data.remove(0).into();

    let start_date = NaiveDate::parse_from_str(&start_date, "%d.%m.%Y")?;
    let end_date = NaiveDate::parse_from_str(&end_date, "%d.%m.%Y")?;
    let other_data: Vec<String> = other_data.split('$').map(String::from).collect();

    let rows = vec![
        ("start_date", start_date.to_string()),
        ("end_date", end_date.to_string()),
        ("name", other_data[0].to_owned()),
        ("created_at", other_data[1].to_owned()),
        ("version", other_data[2].to_owned()),
        ("provider", other_data[3].to_owned()),
    ];

    let auto_increment = AutoIncrement::new();

    let data: Vec<TimetableMetadataEntry> = rows
        .iter()
        .map(|(key, value)| {
            TimetableMetadataEntry::new(auto_increment.next(), key.to_string(), value.to_owned())
        })
        .collect();
    let data = TimetableMetadataEntry::vec_to_map(data);

    Ok(ResourceStorage::new(data))
}
