use crate::parse::{parse_token, peek_token_bool};

use super::{Parse, ParseResult};

pub fn parse_bracketed<'i, P: Parse<'i>>(
    input: &'i str,
    delimiter: &str,
) -> ParseResult<'i, Vec<P>> {
    let (_, input) = parse_token(input, "(")?;

    let (parsed, input) = {
        let mut input = input;

        let mut parsed = vec![];

        loop {
            parsed.push({
                let (parsed, rest) = P::parse(input)?;
                input = rest;
                parsed
            });

            if peek_token_bool(input, delimiter) {
                let (_, rest) = parse_token(input, delimiter)?;
                input = rest;
            } else {
                break;
            }
        }

        (parsed, input)
    };

    let (_, input) = parse_token(input, ")")?;
    Ok((parsed, input))
}
