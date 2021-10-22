use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::character::complete::{space0, space1};
use nom::combinator::opt;
use nom::error::ErrorKind;
use nom::multi::separated_list0;
use nom::sequence::{delimited, pair, tuple};
use nom::IResult;

use crate::parser::util::{line_feed_whitespace, rest_of_file};
use crate::parser::{param, singleton, TonsOfTokens};

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

macro_rules! interop {
    ($f: ident ($c: literal) -> $op: ident) => {
        fn $f(i: &str) -> IResult<&str, Token> {
            let parsed = nom::sequence::tuple((
                //nom::bytes::complete::take_till(|c: char| c.is_whitespace() || c == $c),
                crate::parser::singleton,
                nom::character::complete::space0,
                nom::bytes::complete::tag($c),
                nom::character::complete::space0,
                //nom::bytes::complete::take_till(|c: char| c.is_whitespace() || c == $c),
                crate::parser::singleton,
            ))(i);

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
        }
    };
}

pub(super) fn method_def(i: &str) -> IResult<&str, Token> {
    let parsed = tuple((
        delimited(
            pair(tag("|"), space0),
            separated_list0(tuple((space0, tag(","), space0)), param),
            pair(space0, tag("|")),
        ),
        space0,
        bracket_group,
        line_feed_whitespace,
        rest_of_file,
    ))(i);

    if let Ok(p) = parsed {
        return Ok((
            p.0,
            Token::FnPair(TonsOfTokens(p.1 .0), box p.1 .2, box p.1 .4),
        ));
    } else {
        return Err(parsed.err().unwrap());
    }
}

pub(super) fn method(i: &str) -> IResult<&str, Token> {
    let parsed = tuple((singleton, space0, tag(";"), space0, token))(i);

    if let Ok(p) = parsed {
        return Ok((p.0, Token::Method(box p.1 .0, box p.1 .4)));
    } else {
        return Err(parsed.err().unwrap());
    }
}

pub(super) fn ifstmt(i: &str) -> IResult<&str, Token> {
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

pub(super) fn assignment(i: &str) -> IResult<&str, Token> {
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

fn bracket_group(i: &str) -> IResult<&str, Token> {
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

interop!(eq("==") -> Eq);
interop!(neq("!=") -> Neq);
interop!(gt(">") -> Gt);
interop!(lt("<") -> Lt);
interop!(gte(">=") -> Gte);
interop!(lte("<=") -> Lte);

pub fn comparison(i: &str) -> IResult<&str, Token> {
    alt((eq, neq, gte, lte, gt, lt))(i)
}

interop!(addition("+") -> Add);
interop!(subtraction("-") -> Subtract);

pub fn ops(i: &str) -> IResult<&str, Token> {
    alt((subtraction, addition, comparison))(i)
}
