use nom::{
    bytes::complete::tag, combinator::rest, sequence::preceded, Compare, IResult, Input, Parser,
};

pub(crate) fn bearer_token<I>(input: I) -> IResult<I, I>
where
    I: Input + Compare<&'static str>,
{
    preceded(tag("Bearer "), rest).parse(input)
}
