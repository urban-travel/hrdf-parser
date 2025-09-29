// 1 file(s).
// File(s) read by the parser:
// ECKDATEN
use std::error::Error;

use chrono::NaiveDate;
use nom::{
    Parser,
    branch::alt,
    bytes::{complete::is_not, tag},
    character::{
        complete::char,
        complete::{i32, u32},
    },
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
        separated_list1(char('$'), map(is_not("$"), String::from)),
        InfoLines::MetaData,
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
