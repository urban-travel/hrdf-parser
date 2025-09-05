/// Here we will define all the parsing Helper functions
/// Such as primitive parsers
use nom::{
    Parser,
    branch::alt,
    character::{anychar, one_of},
    combinator::{map, map_res, opt},
    multi::count,
};

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

pub(crate) fn i32_from_n_digits_parser<'a>(
    n_digits: usize,
) -> impl Parser<&'a str, Output = i32, Error = nom::error::Error<&'a str>> {
    map_res(
        count(one_of("0123456789"), n_digits),
        |digits: Vec<char>| to_string(digits).parse(),
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
