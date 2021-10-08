use nom::bytes::complete::take_while;
use nom::IResult;

pub fn line_feed_whitespace(i: &str) -> IResult<&str, &str> {
    take_while(|c| c == '\n' || c == ' ' || c == '\t')(i)
}
