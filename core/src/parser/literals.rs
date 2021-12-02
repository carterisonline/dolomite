use nom::branch::alt;
use nom::bytes::complete::{tag, take_while, take_while1};
use nom::character::complete::char;
use nom::combinator::{map, opt};
use nom::error::ErrorKind;
use nom::number::complete::{double, float};
use nom::sequence::{delimited, pair, preceded};
use nom::IResult;

use crate::parser::util::StrSpan;

#[derive(Debug, PartialEq, Clone)]
pub enum VagueLiteral<'a> {
    Integer(StrSpan<'a>),
    Float(f64),
    String(StrSpan<'a>),
}

#[derive(Debug, PartialEq, Clone)]
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

#[derive(Debug, PartialEq, Clone)]
pub enum Literal<'a> {
    Number(StrictNumber),
    Vague(VagueLiteral<'a>),
    Bool(bool),
    String(StrSpan<'a>),
}

macro_rules! def_strict_int {
    ($id: ident ($t: ident), $suffix: expr) => {
        map(pair(int, tag($suffix)), |b| {
            StrictNumber::$id(b.0.parse::<$t>().unwrap())
        })
    };
}

pub fn literal(i: StrSpan) -> IResult<StrSpan, Literal> {
    alt((
        map(string, |s| Literal::String(s)),
        map(strict_int, |n| Literal::Number(n)),
        map(int, |i| Literal::Vague(VagueLiteral::Integer(i))),
        map(strict_float, |n| Literal::Number(n)),
        map(double, |d| Literal::Vague(VagueLiteral::Float(d))),
    ))(i)
}

fn int(i: StrSpan) -> IResult<StrSpan, StrSpan> {
    if i.contains(".") || i.trim() == "" {
        Err(nom::Err::Error(nom::error::Error::new(i, ErrorKind::Fail)))
    } else if i.starts_with("-") {
        preceded(opt(tag("-")), numbers)(i)
    } else {
        numbers(i)
    }
}

fn numbers(i: StrSpan) -> IResult<StrSpan, StrSpan> {
    take_while1(|c: char| c.is_digit(10))(i)
}

fn strict_float(i: StrSpan) -> IResult<StrSpan, StrictNumber> {
    alt((
        map(pair(float, tag("m")), |b| StrictNumber::MediumFloat(b.0)),
        map(pair(double, tag("l")), |b| StrictNumber::LargeFloat(b.0)),
    ))(i)
}

fn strict_int(i: StrSpan) -> IResult<StrSpan, StrictNumber> {
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

//TODO: implement escaping
fn string(i: StrSpan) -> IResult<StrSpan, StrSpan> {
    let out = delimited(char('"'), take_while(|s| s != '"'), char('"'))(i);
    return out;
}
