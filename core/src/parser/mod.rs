pub mod display;
pub mod literals;
pub mod ops;
pub mod util;

use nom::branch::alt;
use nom::bytes::complete::take_while;
use nom::character::complete::{char, satisfy, space0};
use nom::combinator::{map, rest};
use nom::sequence::{delimited, pair};
use nom::IResult;

use crate::parser::literals::{literal, Literal};
use crate::parser::ops::{assignment, ifstmt, method, ops, Op};
use crate::parser::util::{concat, is_char_alphanumeric, is_char_digit, is_char_valid_ident_mid};

#[derive(Debug, PartialEq)]
pub enum Token {
    Ident(String),
    Assignment {
        mutable: bool,
        type_annotation: Option<Box<Token>>,
        ident: Box<Token>,
        value: Box<Token>,
    },
    Param {
        mutable: bool,
        type_annotation: Box<Token>,
        ident: Box<Token>,
    },
    IfStmt {
        cond: Box<Token>,
    },
    Literal(Literal),
    Op(Op),
    Pair(Box<Token>, Box<Token>),
    CondPair(Box<Token>, Box<Token>, Box<Token>),
    FnPair(Box<Token>, Box<Token>, Box<Token>),
    Method(Box<Token>, Box<Token>),
    None,
}

pub fn parse(i: &str) -> Token {
    token(i).unwrap().1
}

pub(self) fn paren(i: &str) -> IResult<&str, Token> {
    delimited(pair(char('('), space0), token, pair(space0, char(')')))(i)
}

pub(self) fn singleton(i: &str) -> IResult<&str, Token> {
    alt((
        paren,
        map(ident, Token::Ident),
        map(literal, Token::Literal),
    ))(i)
}

pub(self) fn token(i: &str) -> IResult<&str, Token> {
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

fn ident(i: &'a str) -> IResult<&'a str, String> {
    concat(pair(
        alt((
            char('_'),
            satisfy(|c| is_char_alphanumeric(c) && !is_char_digit(c)),
        )),
        take_while(is_char_valid_ident_mid),
    )(i))
}
