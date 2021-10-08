pub mod literals;
pub mod ops;
pub mod util;

use core::fmt;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

use nom::branch::alt;
use nom::bytes::complete::take_while;
use nom::character::complete::{char, satisfy, space0};
use nom::character::is_alphanumeric;
use nom::combinator::{map, rest};
use nom::sequence::{delimited, pair};
use nom::IResult;

use crate::parser::ops::{ifstmt, method};

use self::literals::{is_char_digit, literal, Literal};
use self::ops::{assignment, ops, Op};

#[derive(Debug, PartialEq)]
pub enum Token {
    Ident(String),
    Assignment {
        mutable: bool,
        type_annotation: Option<Box<Token>>,
        ident: Box<Token>,
        value: Box<Token>,
    },
    IfStmt {
        cond: Box<Token>,
    },
    Literal(Literal),
    Op(Op),
    Pair(Box<Token>, Box<Token>),
    CondPair(Box<Token>, Box<Token>, Box<Token>),
    Method(Box<Token>, Box<Token>),
    None,
}

static INDENT: AtomicUsize = AtomicUsize::new(0);
static DO_INDENT: AtomicBool = AtomicBool::new(false);

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut out = Vec::new();
        let indent = INDENT.load(Ordering::Relaxed);
        if DO_INDENT.load(Ordering::Relaxed) {
            out.push((0..indent).into_iter().map(|_| '\t').collect::<String>());
        }

        match &self {
            &Token::Ident(ident) => out.push(format!("{ident}")),

            &Token::Assignment {
                mutable,
                type_annotation,
                ident,
                value,
            } => {
                DO_INDENT.store(false, Ordering::Relaxed);
                out.push(format!(
                    "Assignment{mutable_fmt}: {ident} {type_annotation_fmt} = {value}",
                    mutable_fmt = if *mutable { " (mut)" } else { "" },
                    type_annotation_fmt = match type_annotation {
                        None => "[unknown]".to_string(),
                        Some(token) => format!("[{token}]"),
                    }
                ))
            }

            &Token::Literal(literal) => out.push(format!("{literal}")),

            &Token::Op(op) => out.push(format!("{op}")),

            &Token::IfStmt { cond } => out.push(format!("Cond: {cond}")),

            &Token::Pair(p1, p2) => {
                DO_INDENT.store(false, Ordering::Relaxed);
                out.push(format!("<{p1}"));
                DO_INDENT.store(true, Ordering::Relaxed);
                if format!("{p2}").trim() == "" {
                    out.push(format!(">"));
                } else {
                    out.push(format!(";\n{p2}>"));
                }
            }

            &Token::CondPair(p1, p2, p3) => {
                out.push((0..indent).into_iter().map(|_| '\t').collect::<String>());
                DO_INDENT.store(false, Ordering::Relaxed);
                out.push(format!("<<?{p1};\n"));
                DO_INDENT.store(true, Ordering::Relaxed);
                INDENT.store(INDENT.load(Ordering::Relaxed) + 1, Ordering::Relaxed);
                out.push(format!("{p2}\n"));
                if format!("{p3}").trim() == "" {
                    out.push((0..indent).into_iter().map(|_| '\t').collect::<String>());
                    out.push(format!("?>"));
                    DO_INDENT.store(false, Ordering::Relaxed);
                } else {
                    out.push(format!("?>\n"));
                }

                INDENT.store(INDENT.load(Ordering::Relaxed) - 1, Ordering::Relaxed);
                out.push(format!("{p3}>"));
            }

            &Token::Method(operator, method) => {
                DO_INDENT.store(false, Ordering::Relaxed);
                out.push(format!("<{operator} THEN \n"));
                DO_INDENT.store(true, Ordering::Relaxed);
                out.push(format!("{method}>"));
            }

            &Token::None => (),
        }

        write!(f, "{}", out.join(""))
    }
}

/// Tests if char (not `u8`) is ASCII alphanumeric
fn is_char_alphanumeric(i: char) -> bool {
    is_alphanumeric(i as u8)
}

fn is_char_valid_ident_mid(i: char) -> bool {
    is_char_alphanumeric(i.clone()) || i == '_'
}

/// Magic function that concatenates the result of a parsed pair into a single String.
fn concat<'a, T: ToString, U: ToString>(i: IResult<&'a str, (T, U)>) -> IResult<&'a str, String> {
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

fn ident(i: &'a str) -> IResult<&'a str, String> {
    concat(pair(
        alt((
            char('_'),
            satisfy(|c| is_char_alphanumeric(c) && !is_char_digit(c)),
        )),
        take_while(is_char_valid_ident_mid),
    )(i))
}

pub fn token(i: &str) -> IResult<&str, Token> {
    alt((
        ifstmt,
        assignment,
        method,
        ops,
        map(ident, Token::Ident),
        map(literal, Token::Literal),
        map(rest, |_| Token::None),
    ))(i)
}

fn rest_of_file(i: &str) -> IResult<&str, Token> {
    alt((token, map(rest, |_| Token::None)))(i)
}

pub fn parse(i: &str) -> Token {
    token(i).unwrap().1
}

pub fn singleton(i: &str) -> IResult<&str, Token> {
    alt((
        paren,
        map(ident, Token::Ident),
        map(literal, Token::Literal),
    ))(i)
}

pub fn paren(i: &str) -> IResult<&str, Token> {
    delimited(pair(char('('), space0), token, pair(space0, char(')')))(i)
}
