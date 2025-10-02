use std::{
    fs::File,
    io::{self, Read, Seek},
};

/// Here we will define all the parsing Helper functions
/// Such as primitive parsers
use nom::{
    IResult, Parser,
    branch::alt,
    bytes::{complete::take_till, tag},
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

pub(crate) fn string_from_n_chars_parser(
    n_chars: usize,
) -> impl FnMut(&str) -> IResult<&str, String> {
    move |input: &str| {
        map(count(anychar, n_chars), |chars| {
            to_string(chars).trim().to_string()
        })
        .parse(input)
    }
}

pub(crate) fn string_till_eol_parser(input: &str) -> IResult<&str, String> {
    map(take_till(is_newline), |c: &str| c.trim().to_string()).parse(input)
}

pub(crate) fn i16_from_n_digits_parser(n_digits: usize) -> impl FnMut(&str) -> IResult<&str, i16> {
    move |input: &str| {
        map_res(
            // Take exactly n_digits bytes
            nom::bytes::take(n_digits),
            |n_chars: &str| {
                // Trim spaces and parse to i32
                n_chars.trim().parse::<i16>()
            },
        )
        .parse(input)
    }
}

pub(crate) fn i32_from_n_digits_parser(n_digits: usize) -> impl FnMut(&str) -> IResult<&str, i32> {
    move |input: &str| {
        map_res(
            // Take exactly n_digits bytes
            nom::bytes::take(n_digits),
            |n_chars: &str| {
                // Trim spaces and parse to i32
                n_chars.trim().parse::<i32>()
            },
        )
        .parse(input)
    }
}

fn exactly_n_spaces_or_at_parser<T>(
    n_digits: usize,
) -> impl FnMut(&str) -> IResult<&str, Option<T>> {
    move |input: &str| map(count(one_of(" @"), n_digits), |_| None).parse(input)
}

pub(crate) fn optional_i32_from_n_digits_parser(
    n_digits: usize,
) -> impl FnMut(&str) -> IResult<&str, Option<i32>> {
    move |input: &str| {
        alt((
            exactly_n_spaces_or_at_parser(n_digits),
            opt(i32_from_n_digits_parser(n_digits)),
        ))
        .parse(input)
    }
}

pub(crate) fn direction_parser(input: &str) -> IResult<&str, (String, i32)> {
    (map(tag("R"), String::from), i32_from_n_digits_parser(6)).parse(input)
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

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_is_newline() {
        assert!(is_newline('\n'));
        assert!(is_newline('\r'));
        assert!(!is_newline(' '));
        assert!(!is_newline('a'));
    }

    #[test]
    fn test_to_string() {
        let chars = vec!['h', 'e', 'l', 'l', 'o'];
        assert_eq!(to_string(chars), "hello");

        let empty: Vec<char> = vec![];
        assert_eq!(to_string(empty), "");
    }

    #[test]
    fn test_string_from_n_chars_parser_basic() {
        let input = "ABC123";
        let result = string_from_n_chars_parser(3)(input);
        assert!(result.is_ok());
        let (remaining, parsed) = result.unwrap();
        assert_eq!(parsed, "ABC");
        assert_eq!(remaining, "123");
    }

    #[test]
    fn test_string_from_n_chars_parser_with_spaces() {
        let input = "AB  456";
        let result = string_from_n_chars_parser(4)(input);
        assert!(result.is_ok());
        let (remaining, parsed) = result.unwrap();
        // Should trim the result
        assert_eq!(parsed, "AB");
        assert_eq!(remaining, "456");
    }

    #[test]
    fn test_string_from_n_chars_parser_exact_length() {
        let input = "HELLO";
        let result = string_from_n_chars_parser(5)(input);
        assert!(result.is_ok());
        let (remaining, parsed) = result.unwrap();
        assert_eq!(parsed, "HELLO");
        assert_eq!(remaining, "");
    }

    #[test]
    fn test_string_from_n_chars_parser_insufficient_input() {
        let input = "AB";
        let result = string_from_n_chars_parser(5)(input);
        assert!(result.is_err());
    }

    #[test]
    fn test_string_till_eol_parser_basic() {
        let input = "hello world\nmore text";
        let result = string_till_eol_parser(input);
        assert!(result.is_ok());
        let (remaining, parsed) = result.unwrap();
        assert_eq!(parsed, "hello world");
        assert_eq!(remaining, "\nmore text");
    }

    #[test]
    fn test_string_till_eol_parser_with_cr() {
        let input = "hello\rworld";
        let result = string_till_eol_parser(input);
        assert!(result.is_ok());
        let (remaining, parsed) = result.unwrap();
        assert_eq!(parsed, "hello");
        assert_eq!(remaining, "\rworld");
    }

    #[test]
    fn test_string_till_eol_parser_no_newline() {
        let input = "single line";
        let result = string_till_eol_parser(input);
        assert!(result.is_ok());
        let (remaining, parsed) = result.unwrap();
        assert_eq!(parsed, "single line");
        assert_eq!(remaining, "");
    }

    #[test]
    fn test_string_till_eol_parser_trims_spaces() {
        let input = "  spaces  \nmore";
        let result = string_till_eol_parser(input);
        assert!(result.is_ok());
        let (_, parsed) = result.unwrap();
        assert_eq!(parsed, "spaces");
    }

    #[test]
    fn test_i16_from_n_digits_parser_positive() {
        let input = "123456";
        let result = i16_from_n_digits_parser(3)(input);
        assert!(result.is_ok());
        let (remaining, parsed) = result.unwrap();
        assert_eq!(parsed, 123);
        assert_eq!(remaining, "456");
    }

    #[test]
    fn test_i16_from_n_digits_parser_with_spaces() {
        let input = " 42rest";
        let result = i16_from_n_digits_parser(3)(input);
        assert!(result.is_ok());
        let (remaining, parsed) = result.unwrap();
        assert_eq!(parsed, 42);
        assert_eq!(remaining, "rest");
    }

    #[test]
    fn test_i16_from_n_digits_parser_zero() {
        let input = "000text";
        let result = i16_from_n_digits_parser(3)(input);
        assert!(result.is_ok());
        let (_, parsed) = result.unwrap();
        assert_eq!(parsed, 0);
    }

    #[test]
    fn test_i16_from_n_digits_parser_invalid() {
        let input = "ABCdef";
        let result = i16_from_n_digits_parser(3)(input);
        assert!(result.is_err());
    }

    #[test]
    fn test_i32_from_n_digits_parser_basic() {
        let input = "1234567890";
        let result = i32_from_n_digits_parser(7)(input);
        assert!(result.is_ok());
        let (remaining, parsed) = result.unwrap();
        assert_eq!(parsed, 1234567);
        assert_eq!(remaining, "890");
    }

    #[test]
    fn test_i32_from_n_digits_parser_with_leading_spaces() {
        let input = "  12345rest";
        let result = i32_from_n_digits_parser(7)(input);
        assert!(result.is_ok());
        let (remaining, parsed) = result.unwrap();
        assert_eq!(parsed, 12345);
        assert_eq!(remaining, "rest");
    }

    #[test]
    fn test_i32_from_n_digits_parser_large_number() {
        let input = "8507000end";
        let result = i32_from_n_digits_parser(7)(input);
        assert!(result.is_ok());
        let (remaining, parsed) = result.unwrap();
        assert_eq!(parsed, 8507000);
        assert_eq!(remaining, "end");
    }

    #[test]
    fn test_optional_i32_from_n_digits_parser_with_number() {
        let input = "123456rest";
        let result = optional_i32_from_n_digits_parser(6)(input);
        assert!(result.is_ok());
        let (remaining, parsed) = result.unwrap();
        assert_eq!(parsed, Some(123456));
        assert_eq!(remaining, "rest");
    }

    #[test]
    fn test_optional_i32_from_n_digits_parser_with_spaces() {
        let input = "      rest";
        let result = optional_i32_from_n_digits_parser(6)(input);
        assert!(result.is_ok());
        let (remaining, parsed) = result.unwrap();
        assert_eq!(parsed, None);
        assert_eq!(remaining, "rest");
    }

    #[test]
    fn test_optional_i32_from_n_digits_parser_with_at_signs() {
        let input = "@@@@@@rest";
        let result = optional_i32_from_n_digits_parser(6)(input);
        assert!(result.is_ok());
        let (remaining, parsed) = result.unwrap();
        assert_eq!(parsed, None);
        assert_eq!(remaining, "rest");
    }

    #[test]
    fn test_optional_i32_from_n_digits_parser_mixed_spaces_and_at() {
        let input = " @ @ @rest";
        let result = optional_i32_from_n_digits_parser(6)(input);
        assert!(result.is_ok());
        let (remaining, parsed) = result.unwrap();
        assert_eq!(parsed, None);
        assert_eq!(remaining, "rest");
    }

    #[test]
    fn test_direction_parser_basic() {
        let input = "R123456more";
        let result = direction_parser(input);
        assert!(result.is_ok());
        let (remaining, (direction, id)) = result.unwrap();
        assert_eq!(direction, "R");
        assert_eq!(id, 123456);
        assert_eq!(remaining, "more");
    }

    #[test]
    fn test_direction_parser_with_spaces() {
        let input = "R  8500rest";
        let result = direction_parser(input);
        assert!(result.is_ok());
        let (remaining, (direction, id)) = result.unwrap();
        assert_eq!(direction, "R");
        assert_eq!(id, 8500);
        assert_eq!(remaining, "rest");
    }

    #[test]
    fn test_direction_parser_invalid_prefix() {
        let input = "H123456";
        let result = direction_parser(input);
        assert!(result.is_err());
    }

    #[test]
    fn test_direction_parser_insufficient_digits() {
        let input = "R123";
        let result = direction_parser(input);
        assert!(result.is_err());
    }
}
