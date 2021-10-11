use nom::branch::alt;
use nom::bytes::complete::{tag, take_while};
use nom::character::complete::char;
use nom::combinator::{map, opt};
use nom::error::ErrorKind;
use nom::number::complete::{double, float};
use nom::sequence::{delimited, pair};
use nom::IResult;

use crate::parser::util::is_char_digit;

#[derive(Debug, PartialEq)]
pub enum VagueLiteral {
    Integer(String),
    Float(f64),
    String(String),
}

#[derive(Debug, PartialEq)]
pub enum StrictNumber {
    Byte(u8),
    ByteSigned(i8),
    Small(u16),
    SmallSigned(i16),
    Medium(u32),
    MediumSigned(i32),
    MediumFloat(f32),
    Large(u64),
    LargeSigned(i64),
    LargeFloat(f64),
}

#[derive(Debug, PartialEq)]
pub enum Literal {
    Number(StrictNumber),
    Vague(VagueLiteral),
    Bool(bool),
    String(String),
}

macro_rules! def_strict_int {
    ($id: ident ($t: ident), $suffix: expr) => {
        map(pair(int, tag($suffix)), |b| {
            StrictNumber::$id(b.0.parse::<$t>().unwrap())
        })
    };
}

pub fn literal(i: &'a str) -> IResult<&'a str, Literal> {
    alt((
        map(string, |s| Literal::String(s.to_string())),
        map(strict_int, |n| Literal::Number(n)),
        map(int, |i| {
            Literal::Vague(VagueLiteral::Integer(i.to_string()))
        }),
        map(strict_float, |n| Literal::Number(n)),
        map(double, |d| Literal::Vague(VagueLiteral::Float(d))),
    ))(i)
}

fn append_prefix<T: ToString, U: ToString>(
    i: IResult<&'a str, (Option<T>, U)>,
) -> IResult<&'a str, String> {
    if let Ok(res) = i {
        if let Some(c) = res.1 .0 {
            return Ok((res.0, format!("{}{}", c.to_string(), res.1 .1.to_string())));
        } else {
            return Ok((res.0, res.1 .1.to_string()));
        }
    } else {
        return Err(i.err().unwrap());
    }
}

fn int(i: &'a str) -> IResult<&'a str, String> {
    if i.contains(".") || i.trim() == "" {
        Err(nom::Err::Error(nom::error::Error::new(i, ErrorKind::Fail)))
    } else {
        append_prefix(pair(opt(char('-')), numbers)(i))
    }
}

fn numbers(i: &'a str) -> IResult<&'a str, &'a str> {
    take_while(is_char_digit)(i)
}

fn strict_float(i: &'a str) -> IResult<&'a str, StrictNumber> {
    alt((
        map(pair(float, tag("m")), |b| StrictNumber::MediumFloat(b.0)),
        map(pair(double, tag("l")), |b| StrictNumber::LargeFloat(b.0)),
    ))(i)
}

fn strict_int(i: &'a str) -> IResult<&'a str, StrictNumber> {
    alt((
        def_strict_int!(ByteSigned(i8), "bi"),
        def_strict_int!(Byte(u8), "b"),
        def_strict_int!(SmallSigned(i16), "si"),
        def_strict_int!(Small(u16), "s"),
        def_strict_int!(MediumSigned(i32), "mi"),
        def_strict_int!(Medium(u32), "m"),
        def_strict_int!(LargeSigned(i64), "li"),
        def_strict_int!(Large(u64), "l"),
    ))(i)
}

fn string(i: &'a str) -> IResult<&'a str, &'a str> {
    let out = delimited(char('"'), take_while(|s| s != '"'), char('"'))(i);
    return out;
}
