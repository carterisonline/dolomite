use nom::branch::alt;
use nom::bytes::complete::take_while;
use nom::combinator::{map, rest};
use nom::IResult;
use nom_locate::LocatedSpan;

use crate::parser::{token, Token};
use crate::{attempt, got};

pub(super) type StrSpan<'a> = LocatedSpan<&'a str, ()>;

pub(super) fn line_feed_whitespace(i: StrSpan) -> IResult<StrSpan, StrSpan> {
    attempt!("line_feed_whitespace" from i);
    let (span, parsed) = take_while(|c| c == '\n' || c == ' ' || c == '\t')(i)?;

    got!("line_feed_whitespace" from i);

    return Ok((span, parsed));
}

pub(super) fn rest_of_file(i: StrSpan) -> IResult<StrSpan, Token> {
    attempt!("rest_of_file" from i);
    let (span, parsed) = alt((token, map(rest, |_| Token::None)))(i)?;

    got!("rest_of_file" from i);

    return Ok((span, parsed));
}
