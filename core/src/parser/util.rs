use nom::branch::alt;
use nom::bytes::complete::take_while;
use nom::character::{is_alphanumeric, is_digit};
use nom::combinator::{map, rest};
use nom::IResult;

use crate::parser::{token, Token};

pub(super) fn line_feed_whitespace(i: &str) -> IResult<&str, &str> {
    take_while(|c| c == '\n' || c == ' ' || c == '\t')(i)
}

pub(super) fn concat<'a, T: ToString, U: ToString>(
    i: IResult<&'a str, (T, U)>,
) -> IResult<&'a str, String> {
    if i.is_err() {
        return Err(i.err().unwrap());
    } else {
        let i_ok = i.unwrap();
        return Ok((
            i_ok.0,
            format!("{}{}", i_ok.1 .0.to_string(), i_ok.1 .1.to_string()),
        ));
    }
}

pub(super) fn is_char_digit(i: char) -> bool {
    is_digit(i as u8)
}

pub(super) fn is_char_alphanumeric(i: char) -> bool {
    is_alphanumeric(i as u8)
}

pub(super) fn is_char_valid_ident_mid(i: char) -> bool {
    is_char_alphanumeric(i.clone()) || i == '_'
}

pub(super) fn rest_of_file(i: &str) -> IResult<&str, Token> {
    alt((token, map(rest, |_| Token::None)))(i)
}
