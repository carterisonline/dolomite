use std::fmt;

use nom::branch::alt;
use nom::bytes::complete::{tag, take_while};
use nom::character::complete::char;
use nom::character::is_digit;
use nom::combinator::{map, opt};
use nom::error::ErrorKind;
use nom::number::complete::{double, float};
use nom::sequence::{delimited, pair};
use nom::IResult;

#[derive(Debug, PartialEq)]
pub enum VagueLiteral {
    Integer(String),
    Float(f64),
    String(String),
}

impl fmt::Display for VagueLiteral {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self {
            &VagueLiteral::Float(i) => write!(f, "{i}vf"),
            &VagueLiteral::Integer(i) => write!(f, "{i}v"),
            &VagueLiteral::String(s) => write!(f, "{s}"),
        }
    }
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

impl fmt::Display for StrictNumber {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self {
            &StrictNumber::Byte(i) => write!(f, "{i}b"),
            &StrictNumber::ByteSigned(i) => write!(f, "{i}bi"),
            &StrictNumber::Small(i) => write!(f, "{i}s"),
            &StrictNumber::SmallSigned(i) => write!(f, "{i}si"),
            &StrictNumber::Medium(i) => write!(f, "{i}m"),
            &StrictNumber::MediumSigned(i) => write!(f, "{i}mi"),
            &StrictNumber::MediumFloat(i) => write!(f, "{i}m"),
            &StrictNumber::Large(i) => write!(f, "{i}l"),
            &StrictNumber::LargeSigned(i) => write!(f, "{i}li"),
            &StrictNumber::LargeFloat(i) => write!(f, "{i}l"),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum Literal {
    Number(StrictNumber),
    Vague(VagueLiteral),
    Bool(bool),
    String(String),
}

impl fmt::Display for Literal {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self {
            &Literal::Number(num) => write!(f, "{num}"),
            &Literal::Vague(num) => write!(f, "{num}"),
            &Literal::Bool(b) => write!(f, "{b}"),
            &Literal::String(s) => write!(f, "{s}"),
        }
    }
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

pub fn is_char_digit(i: char) -> bool {
    is_digit(i as u8)
}

fn numbers(i: &'a str) -> IResult<&'a str, &'a str> {
    take_while(is_char_digit)(i)
}

pub fn int(i: &'a str) -> IResult<&'a str, String> {
    if i.contains(".") || i.trim() == "" {
        Err(nom::Err::Error(nom::error::Error::new(i, ErrorKind::Fail)))
    } else {
        append_prefix(pair(opt(char('-')), numbers)(i))
    }
}

pub fn strict_int(i: &'a str) -> IResult<&'a str, StrictNumber> {
    alt((
        map(pair(int, tag("bi")), |b| {
            StrictNumber::ByteSigned(b.0.parse::<i8>().unwrap())
        }),
        map(pair(int, tag("b")), |b| {
            StrictNumber::Byte(b.0.parse::<u8>().unwrap())
        }),
        map(pair(int, tag("si")), |b| {
            StrictNumber::SmallSigned(b.0.parse::<i16>().unwrap())
        }),
        map(pair(int, tag("s")), |b| {
            StrictNumber::Small(b.0.parse::<u16>().unwrap())
        }),
        map(pair(int, tag("mi")), |b| {
            StrictNumber::MediumSigned(b.0.parse::<i32>().unwrap())
        }),
        map(pair(int, tag("m")), |b| {
            StrictNumber::Medium(b.0.parse::<u32>().unwrap())
        }),
        map(pair(int, tag("li")), |b| {
            StrictNumber::LargeSigned(b.0.parse::<i64>().unwrap())
        }),
        map(pair(int, tag("l")), |b| {
            StrictNumber::Large(b.0.parse::<u64>().unwrap())
        }),
    ))(i)
}

pub fn strict_float(i: &'a str) -> IResult<&'a str, StrictNumber> {
    alt((
        map(pair(float, tag("m")), |b| StrictNumber::MediumFloat(b.0)),
        map(pair(double, tag("l")), |b| StrictNumber::LargeFloat(b.0)),
    ))(i)
}

pub fn string(i: &'a str) -> IResult<&'a str, &'a str> {
    let out = delimited(char('"'), take_while(|s| s != '"'), char('"'))(i);
    return out;
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
