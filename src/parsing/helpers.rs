use std::{
    fs::File,
    io::{self, Read, Seek},
};

/// Here we will define all the parsing Helper functions
/// Such as primitive parsers
use nom::{
    Parser,
    branch::alt,
    bytes::complete::take_till,
    character::{anychar, one_of},
    combinator::{map, map_res, opt},
    multi::count,
};

pub(crate) fn is_newline(c: char) -> bool {
    c == '\n' || c == '\r'
}

pub(crate) fn to_string(v: Vec<char>) -> String {
    v.into_iter().collect::<String>()
}

pub(crate) fn string_from_n_chars_parser<'a>(
    n_chars: usize,
) -> impl Parser<&'a str, Output = String, Error = nom::error::Error<&'a str>> {
    map(count(anychar, n_chars), |chars| {
        to_string(chars).trim().to_string()
    })
}

pub(crate) fn string_till_eol_parser<'a>()
-> impl Parser<&'a str, Output = String, Error = nom::error::Error<&'a str>> {
    map(take_till(is_newline), |c: &str| c.trim().to_string())
}

pub(crate) fn i32_from_n_digits_parser<'a>(
    n_digits: usize,
) -> impl Parser<&'a str, Output = i32, Error = nom::error::Error<&'a str>> {
    map_res(
        // Take exactly n_digits bytes
        nom::bytes::take(n_digits),
        |n_chars: &str| {
            // Trim spaces and parse to i32
            n_chars.trim().parse::<i32>()
        },
    )
}

fn exaclty_n_spaces_parser<'a, T>(
    n_digits: usize,
) -> impl Parser<&'a str, Output = Option<T>, Error = nom::error::Error<&'a str>> {
    map(count(one_of(" "), n_digits), |_| None)
}

pub(crate) fn optional_i32_from_n_digits_parser<'a>(
    n_digits: usize,
) -> impl Parser<&'a str, Output = Option<i32>, Error = nom::error::Error<&'a str>> {
    alt((
        exaclty_n_spaces_parser(n_digits),
        opt(i32_from_n_digits_parser(n_digits)),
    ))
}

pub(crate) fn read_lines(path: &str, bytes_offset: u64) -> io::Result<Vec<String>> {
    let mut file = File::open(path)?;
    file.seek(io::SeekFrom::Start(bytes_offset))?;
    let mut reader = io::BufReader::new(file);
    let mut contents = String::new();
    reader.read_to_string(&mut contents)?;
    let lines = contents.lines().map(String::from).collect();
    Ok(lines)
}
