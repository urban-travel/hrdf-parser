/// Here we will define all the parsing Helper functions
/// Such as primitive parsers
use nom::{
    IResult, Parser,
    character::{anychar, one_of},
    combinator::{map, map_res},
    multi::count,
};

pub(crate) fn to_string(v: Vec<char>) -> String {
    v.into_iter().collect::<String>()
}

pub(crate) fn string_from_n_chars_parser<'a>(
    n_chars: usize,
) -> impl Parser<&'a str, Output = String, Error = nom::error::Error<&'a str>> {
    map(count(anychar, n_chars), to_string)
}

pub(crate) fn i32_from_n_digits_parser<'a>(
    n_digits: usize,
) -> impl Parser<&'a str, Output = i32, Error = nom::error::Error<&'a str>> {
    map_res(
        count(one_of("0123456789"), n_digits),
        |digits: Vec<char>| to_string(digits).parse(),
    )
}
