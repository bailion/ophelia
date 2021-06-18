use std::fmt::Display;

use crate::parse::{ignore_whitespace, up_to, ParseError};

use super::{Parse, ParseResult};

#[derive(Debug, Clone, PartialEq)]
pub struct Ident<'i> {
    name: &'i str,
}

impl<'i> Parse<'i> for Ident<'i> {
    fn parse(input: &'i str) -> super::ParseResult<Self> {
        ignore_whitespace(input, |input| {
            let mut index = 0;

            let next = input
                .chars()
                .next()
                .ok_or(ParseError::UnexpectedEndOfInput)?;

            if !next.is_ascii_alphabetic() {
                return Err(ParseError::UnexpectedToken(input.get(0..0).unwrap()));
            }

            index += 1;

            loop {
                let next = input
                    .chars()
                    .next()
                    .ok_or(ParseError::UnexpectedEndOfInput)?;

                if next.is_alphanumeric() {
                    index += 1;
                } else {
                    break;
                }
            }

            Ok((
                Self {
                    name: input.get(0..index).unwrap(),
                },
                input.get(index..).unwrap_or(""),
            ))
        })
    }
}

impl Display for Ident<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.name.fmt(f)
    }
}

pub fn ident_list(mut input: &str) -> ParseResult<Vec<Ident<'_>>> {
    let mut idents_of_iter = vec![];

    loop {
        let (_, leftover) = up_to(input, &[",", "in"])?;

        let (ident, rest) = Ident::parse(input)?;

        input = rest;

        idents_of_iter.push(ident);

        let input_is_empty = ignore_whitespace(leftover, |input| Ok((input.is_empty(), input)))?.0;

        if input_is_empty {
            break;
        }

        let next_token_is_in = ignore_whitespace(leftover, |whitespace_vanquished| {
            let cond = input.get(0..=1) == Some("in");

            if cond {
                input = whitespace_vanquished;
            }

            Ok((cond, whitespace_vanquished))
        })?
        .0;

        if next_token_is_in {
            break;
        }
    }

    Ok((idents_of_iter, input))
}
