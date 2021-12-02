pub mod display;
pub mod literals;
pub mod ops;
pub mod util;

use log::info;
use nom::branch::alt;
use nom::bytes::complete::{tag, take_while1};
use nom::character::complete::{char, space0, space1};
use nom::combinator::{map, opt, rest};
use nom::multi::separated_list1;
use nom::sequence::{delimited, pair, tuple};
use nom::IResult;
use nom_locate::LocatedSpan;

use crate::parser::literals::{literal, Literal};
use crate::parser::ops::{assignment, ifstmt, method, method_def, method_unit, ops, Op};
use crate::parser::util::StrSpan;
use crate::{attempt, got};

#[derive(Debug, PartialEq, Clone)]
pub enum Token<'a> {
    /// An identifier such as: \
    /// `helloworld` \
    /// `variable2`
    Ident(StrSpan<'a>),

    /// An assignment to a variable: \
    /// `a = 10` \
    /// `mut b = 100` \
    /// `mut large population = 800l`
    Assignment {
        mutable: bool,
        type_annotation: Option<Box<Token<'a>>>,
        ident: Box<Token<'a>>,
        value: Box<Token<'a>>,
    },

    /// A function parameter: \
    /// `small x` \
    /// `mut medium y` \
    Param {
        mutable: bool,
        type_annotation: Box<Token<'a>>,
        ident: Box<Token<'a>>,
    },

    /// An if statement's condition: \
    /// `if x == 0` \
    /// `if fecal == "funny"`
    IfStmt {
        cond: Box<Token<'a>>,
    },

    /// A literal value: \
    /// `1` \
    /// `2l` \
    /// `"hello"`
    Literal(Literal<'a>),

    /// A dyadic operation: \
    /// `2 > 1` \
    /// `100 + 1` \
    /// `30 == 30`
    Op(Op<'a>),

    /// Connects two expressions together in a linked-list-ish format
    Pair(Box<Token<'a>>, Box<Token<'a>>),

    /// Similar to `Token::Pair` but it contains a middle token for conditions or blocks.
    CondPair(Box<Token<'a>>, Box<Token<'a>>, Box<Token<'a>>),

    /// Similar to `Token::Pair` but it contains a middle token for a block's contents, while the first contains a list of parameters.
    FnPair(TonsOfTokens<'a>, Box<Token<'a>>, Box<Token<'a>>),

    /// An operation on an object: \
    /// `"hello"; print` \
    /// `1; add_one`
    Method(Box<Token<'a>>, Box<Token<'a>>),

    /// Traditional function call: \
    /// `print "Hello"` \
    /// `add [1, 2]`
    MethodUnit(Box<Token<'a>>, Box<Token<'a>>),
    Span(LocatedSpan<&'a str>, Box<Token<'a>>),
    Array(Vec<Token<'a>>),
    None,
}

#[derive(Debug, PartialEq, Clone)]
pub struct TonsOfTokens<'a>(pub Vec<Token<'a>>);

pub fn parse(i: &str) -> Token {
    info!(
        "Parsing file {}",
        if i.len() <= 30 {
            String::from(
                i.lines()
                    .filter(|s| s.trim().len() > 0)
                    .collect::<Vec<&str>>()
                    .join(" -> "),
            )
        } else {
            format!("{}...", &i[0..29])
                .lines()
                .filter(|s| s.trim().len() > 0)
                .collect::<Vec<&str>>()
                .join(" -> ")
        }
    );

    token(StrSpan::from(i)).unwrap().1
}

pub(self) fn paren(i: StrSpan) -> IResult<StrSpan, Token> {
    attempt!("paren" from i);
    let (span, parsed) = delimited(pair(char('('), space0), token, pair(space0, char(')')))(i)?;
    got!("paren" from i);

    return Ok((span, parsed));
}

pub(self) fn singleton(i: StrSpan) -> IResult<StrSpan, Token> {
    attempt!("singleton" from i);
    let (span, parsed) = alt((
        paren,
        map(ident, Token::Ident),
        map(literal, Token::Literal),
    ))(i)?;

    got!("singleton" from i);

    return Ok((span, parsed));
}

pub(self) fn token(i: StrSpan) -> IResult<StrSpan, Token> {
    attempt!("token" from i);
    let (span, parsed) = alt((
        method_def,
        ifstmt,
        assignment,
        method,
        ops,
        method_unit,
        array,
        map(ident, Token::Ident),
        map(literal, Token::Literal),
        map(rest, |_| Token::None),
    ))(i)?;

    got!("token" from i);

    return Ok((span, parsed));
}

pub(self) fn param(i: StrSpan) -> IResult<StrSpan, Token> {
    attempt!("param" from i);
    let (span, parsed) = tuple((opt(pair(tag("mut"), space1)), ident, space1, ident))(i)?;

    got!("param" from i);

    return Ok((
        span,
        Token::Param {
            mutable: parsed.0.is_some(),
            ident: box Token::Ident(parsed.3),
            type_annotation: box Token::Ident(parsed.1),
        },
    ));
}

fn ident(i: StrSpan) -> IResult<StrSpan, StrSpan> {
    attempt!("ident" from i);
    let (span, parsed) = take_while1(|c: char| c.is_alphabetic())(i)?;

    got!("ident" from i);

    return Ok((span, parsed));
}

fn array(i: StrSpan) -> IResult<StrSpan, Token> {
    attempt!("array" from i);
    let (span, parsed) = delimited(
        pair(tag("["), space0),
        separated_list1(tuple((space0, tag(","), space0)), token),
        pair(space0, tag("]")),
    )(i)?;

    got!("array" from i);

    return Ok((span, Token::Array(parsed)));
}
