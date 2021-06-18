use std::fmt::Display;

use crate::parse::parse_token;

use super::{ast::Ast, peek_token, Parse, ParseError, ParseResult};

#[derive(Debug, Clone, PartialEq)]

pub struct Else<'i> {
    block: Ast<'i>,
}

impl<'i> Parse<'i> for Else<'i> {
    fn parse(input: &'i str) -> ParseResult<Self> {
        let (_token, input) = parse_token(input, "else")?;

        let (block, input) = Ast::parse(input)?;

        Ok((Self { block }, input))
    }

    fn parse_optional(input: &'i str) -> ParseResult<Option<Self>> {
        if peek_token(input, "else").unwrap_or((false, "")).0 {
            Self::parse(input).map(|(a, b)| (Some(a), b))
        } else {
            if input.is_empty() {
                return Err(ParseError::UnexpectedEndOfInput);
            }
            if input.len() < "else".len() {
                return Err(ParseError::UnexpectedToken(input.get(0..).unwrap()));
            } else {
                return Err(ParseError::UnexpectedToken(
                    input.get(0..input.len()).unwrap(),
                ));
            }
        }
    }
}

impl Display for Else<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("{% else %}")?;
        self.block.fmt(f)
    }
}
