use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::character::complete::{space0, space1};
use nom::combinator::opt;
use nom::error::ErrorKind;
use nom::multi::separated_list0;
use nom::sequence::{delimited, pair, tuple};
use nom::IResult;
use nom_locate::LocatedSpan;

use crate::parser::util::{line_feed_whitespace, rest_of_file, StrSpan};
use crate::parser::{param, singleton, TonsOfTokens};

use super::{attempt, got, ident, token, Token};

#[derive(Debug, PartialEq, Clone)]
pub enum Op<'a> {
    Add(Box<Token<'a>>, Box<Token<'a>>),
    Subtract(Box<Token<'a>>, Box<Token<'a>>),
    Eq(Box<Token<'a>>, Box<Token<'a>>),
    Neq(Box<Token<'a>>, Box<Token<'a>>),
    Gt(Box<Token<'a>>, Box<Token<'a>>),
    Lt(Box<Token<'a>>, Box<Token<'a>>),
    Gte(Box<Token<'a>>, Box<Token<'a>>),
    Lte(Box<Token<'a>>, Box<Token<'a>>),
}

macro_rules! interop {
    ($name: expr; $f: ident ($c: literal) -> $op: ident) => {
        fn $f(i: StrSpan) -> IResult<StrSpan, Token> {
            attempt!($name from i);
            let (span, parsed) = nom::sequence::tuple((
                crate::parser::singleton,
                nom::character::complete::space0,
                nom::bytes::complete::tag($c),
                nom::character::complete::space0,
                crate::parser::singleton,
            ))(i)?;

            got!($name from i);

            return Ok((
                span,
                crate::parser::Token::Op(crate::parser::ops::Op::$op(box parsed.0, box parsed.4)),
            ));
        }
    };
}

pub(super) fn method_def(i: StrSpan) -> IResult<StrSpan, Token> {
    attempt!("method_def" from i);
    let (span, parsed) = tuple((
        delimited(
            pair(tag("|"), space0),
            separated_list0(tuple((space0, tag(","), space0)), param),
            pair(space0, tag("|")),
        ),
        space0,
        bracket_group,
        line_feed_whitespace,
        rest_of_file,
    ))(i)?;

    got!("method_def" from i);

    return Ok((
        span,
        Token::FnPair(TonsOfTokens(parsed.0), box parsed.2, box parsed.4),
    ));
}

pub(super) fn method_unit(i: StrSpan) -> IResult<StrSpan, Token> {
    attempt!("method_unit" from i);
    let (span, parsed) = tuple((ident, space1, token, line_feed_whitespace, rest_of_file))(i)?;

    got!("method_unit" from i);

    return Ok((
        span,
        Token::Pair(
            box Token::MethodUnit(box Token::Ident(parsed.0), box parsed.2),
            box parsed.4,
        ),
    ));
}

pub(super) fn method(i: StrSpan) -> IResult<StrSpan, Token> {
    attempt!("method" from i);
    let (span, parsed) = tuple((singleton, space0, tag(";"), space0, token))(i)?;

    got!("method" from i);

    return Ok((span, Token::Method(box parsed.0, box parsed.4)));
}

pub(super) fn ifstmt(i: StrSpan) -> IResult<StrSpan, Token> {
    attempt!("ifstmt" from i);
    let (span, parsed) = tuple((
        tag("if"),
        space0,
        comparison,
        space0,
        bracket_group,
        line_feed_whitespace,
        rest_of_file,
    ))(i)?;

    got!("ifstmt" from i);

    return Ok((
        span,
        Token::CondPair(
            box Token::IfStmt { cond: box parsed.2 },
            box parsed.4,
            box parsed.6,
        ),
    ));
}

pub(super) fn assignment(i: StrSpan) -> IResult<StrSpan, Token> {
    attempt!("assignment" from i);
    let (span, parsed) = tuple((
        opt(pair(tag("mut"), space1)),
        ident,
        space0,
        opt(pair(ident, space0)),
        tag("="),
        space0,
        token,
        line_feed_whitespace,
        rest_of_file,
    ))(i)?;

    got!("assignment" from i);

    if let Some(t) = parsed.3 {
        return Ok((
            span,
            Token::Pair(
                box Token::Assignment {
                    mutable: parsed.0.is_some(),
                    type_annotation: Some(box Token::Ident(ident(parsed.1).unwrap().1)),
                    ident: box Token::Ident(ident(t.0).unwrap().1),
                    value: box parsed.6,
                },
                box parsed.8,
            ),
        ));
    } else {
        return Ok((
            span,
            Token::Pair(
                box Token::Assignment {
                    mutable: parsed.0.is_some(),
                    type_annotation: None,
                    ident: box Token::Ident(ident(parsed.1).unwrap().1),
                    value: box parsed.6,
                },
                box parsed.8,
            ),
        ));
    }
}

//TODO: `LocatedSpan::from` using a slice is questionable... maybe??
//      this function is also generally garbage
fn bracket_group(i: StrSpan) -> IResult<StrSpan, Token> {
    attempt!("bracket_group" from i);
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
                    match delimited(line_feed_whitespace, token, line_feed_whitespace)(
                        LocatedSpan::from(&i[1..u]),
                    ) {
                        Ok((_, t)) => {
                            got!("bracket_group" from i);
                            return Ok((LocatedSpan::from(&i[(u + 1)..i.len()]), t));
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

    // Simple version that doesn't work :(

    /*
    let (span, parsed) = delimited(
        pair(tag("{"), line_feed_whitespace),
        token,
        pair(line_feed_whitespace, tag("}")),
    )(i)?;

    return Ok((span, parsed));*/
}

interop!("eq"; eq("==") -> Eq);
interop!("neq"; neq("!=") -> Neq);
interop!("gt"; gt(">") -> Gt);
interop!("lt"; lt("<") -> Lt);
interop!("gte"; gte(">=") -> Gte);
interop!("lte"; lte("<=") -> Lte);

pub fn comparison(i: StrSpan) -> IResult<StrSpan, Token> {
    attempt!("comparison" from i);
    let (span, parsed) = alt((eq, neq, gte, lte, gt, lt))(i)?;

    got!("comparison" from i);

    return Ok((span, parsed));
}

interop!("addition"; addition("+") -> Add);
interop!("subtraction"; subtraction("-") -> Subtract);

pub fn ops(i: StrSpan) -> IResult<StrSpan, Token> {
    attempt!("ops" from i);
    let (span, parsed) = alt((subtraction, addition, comparison))(i)?;

    got!("ops" from i);
    return Ok((span, parsed));
}
