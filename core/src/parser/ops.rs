use std::fmt;

use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::character::complete::{space0, space1};
use nom::combinator::opt;
use nom::error::ErrorKind;
use nom::sequence::{delimited, pair, tuple};
use nom::IResult;

use crate::parser::util::line_feed_whitespace;
use crate::parser::{rest_of_file, singleton};

use super::{ident, token, Token};

#[derive(Debug, PartialEq)]
pub enum Op {
    Add(Box<Token>, Box<Token>),
    Subtract(Box<Token>, Box<Token>),
    Eq(Box<Token>, Box<Token>),
    Neq(Box<Token>, Box<Token>),
    Gt(Box<Token>, Box<Token>),
    Lt(Box<Token>, Box<Token>),
    Gte(Box<Token>, Box<Token>),
    Lte(Box<Token>, Box<Token>),
}

impl fmt::Display for Op {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self {
            &Op::Add(t1, t2) => {
                write!(f, "({t1} + {t2})")
            }

            &Op::Subtract(t1, t2) => {
                write!(f, "({t1} - {t2})")
            }

            &Op::Eq(t1, t2) => {
                write!(f, "({t1} == {t2})")
            }

            &Op::Neq(t1, t2) => {
                write!(f, "({t1} != {t2})")
            }
            &Op::Gt(t1, t2) => {
                write!(f, "({t1} > {t2})")
            }
            &Op::Lt(t1, t2) => {
                write!(f, "({t1} < {t2})")
            }
            &Op::Gte(t1, t2) => {
                write!(f, "({t1} >= {t2})")
            }
            &Op::Lte(t1, t2) => {
                write!(f, "({t1} <= {t2})")
            }
        }
    }
}

pub fn method(i: &str) -> IResult<&str, Token> {
    let parsed = tuple((singleton, space0, tag(";"), space0, token))(i);

    if let Ok(p) = parsed {
        return Ok((p.0, Token::Method(box p.1 .0, box p.1 .4)));
    } else {
        return Err(parsed.err().unwrap());
    }
}

pub fn bracket_group(i: &str) -> IResult<&str, Token> {
    let mut level = 1;

    if i.chars().nth(0) != Some('{') {
        return Err(nom::Err::Error(nom::error::Error::new(i, ErrorKind::Fail)));
    }
    for (u, c) in i.chars().enumerate() {
        if u == 0 {
            continue;
        }

        match c {
            '{' => level += 1,
            '}' => {
                if level == 1 {
                    match delimited(line_feed_whitespace, token, line_feed_whitespace)(&i[1..u]) {
                        Ok((_, t)) => {
                            return Ok((&i[(u + 1)..i.len()], t));
                        }

                        Err(e) => return Err(e),
                    }
                } else {
                    level -= 1;
                }
            }
            _ => (),
        }
    }

    return Err(nom::Err::Error(nom::error::Error::new(i, ErrorKind::Fail)));
}

pub fn ifstmt(i: &str) -> IResult<&str, Token> {
    let parsed = tuple((
        tag("if"),
        space0,
        comparison,
        space0,
        bracket_group,
        line_feed_whitespace,
        rest_of_file,
    ))(i);

    if let Ok(p) = parsed {
        return Ok((
            p.0,
            Token::CondPair(
                box Token::IfStmt { cond: box p.1 .2 },
                box p.1 .4,
                box p.1 .6,
            ),
        ));
    } else {
        return Err(parsed.err().unwrap());
    }
}

pub fn assignment(i: &str) -> IResult<&str, Token> {
    let parsed = tuple((
        opt(pair(tag("mut"), space1)),
        ident,
        space0,
        opt(pair(ident, space0)),
        tag("="),
        space0,
        token,
        line_feed_whitespace,
        rest_of_file,
    ))(i);

    if let Ok(p) = parsed {
        if let Some(t) = p.1 .3 {
            return Ok((
                p.0,
                Token::Pair(
                    box Token::Assignment {
                        mutable: p.1 .0.is_some(),
                        type_annotation: Some(box Token::Ident(ident(&p.1 .1).unwrap().1)),
                        ident: box Token::Ident(ident(&t.0).unwrap().1),
                        value: box p.1 .6,
                    },
                    box p.1 .8,
                ),
            ));
        } else {
            return Ok((
                p.0,
                Token::Pair(
                    box Token::Assignment {
                        mutable: p.1 .0.is_some(),
                        type_annotation: None,
                        ident: box Token::Ident(ident(&p.1 .1).unwrap().1),
                        value: box p.1 .6,
                    },
                    box p.1 .8,
                ),
            ));
        }
    } else {
        return Err(parsed.err().unwrap());
    }
}

macro_rules! interop {
    ($i: expr, $c: expr, $op: ident) => {{
        let parsed = nom::sequence::tuple((
            //nom::bytes::complete::take_till(|c: char| c.is_whitespace() || c == $c),
            crate::parser::singleton,
            nom::character::complete::space0,
            nom::bytes::complete::tag($c),
            nom::character::complete::space0,
            //nom::bytes::complete::take_till(|c: char| c.is_whitespace() || c == $c),
            crate::parser::singleton,
        ))($i);

        if let Ok(p) = parsed {
            return Ok((
                p.0,
                crate::parser::Token::Op(crate::parser::ops::Op::$op(
                    //box crate::parser::singleton(&p.1 .0).unwrap().1,
                    box p.1 .0, //box crate::parser::singleton(&p.1 .4).unwrap().1,
                    box p.1 .4,
                )),
            ));
        } else {
            return Err(parsed.err().unwrap());
        }
    }};
}

pub fn addition(i: &str) -> IResult<&str, Token> {
    interop!(i, "+", Add)
}

pub fn subtraction(i: &str) -> IResult<&str, Token> {
    interop!(i, "-", Subtract)
}

pub fn comparison(i: &str) -> IResult<&str, Token> {
    alt((eq, neq, gte, lte, gt, lt))(i)
}

pub fn eq(i: &str) -> IResult<&str, Token> {
    interop!(i, "==", Eq)
}

pub fn neq(i: &str) -> IResult<&str, Token> {
    interop!(i, "!=", Neq)
}
pub fn gt(i: &str) -> IResult<&str, Token> {
    interop!(i, ">", Gt)
}
pub fn lt(i: &str) -> IResult<&str, Token> {
    interop!(i, "<", Lt)
}
pub fn gte(i: &str) -> IResult<&str, Token> {
    interop!(i, ">=", Gte)
}
pub fn lte(i: &str) -> IResult<&str, Token> {
    interop!(i, "<=", Lte)
}

pub fn ops(i: &str) -> IResult<&str, Token> {
    alt((subtraction, addition, comparison))(i)
}
