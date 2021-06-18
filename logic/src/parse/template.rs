use std::{fmt::Display, path::PathBuf};

use super::{block::Block, Parse, ParseResult};

#[derive(Debug, Clone, PartialEq)]
pub struct Template<'i> {
    path: Option<PathBuf>,
    expressions: Vec<Block<'i>>,
}

impl<'i> Parse<'i> for Template<'i> {
    fn parse(mut input: &'i str) -> super::ParseResult<Self> {
        let (expressions, left_over) = {
            let mut output = vec![];
            while !input.is_empty() {
                let (out, left_over) = Block::parse(input)?;
                input = left_over;
                output.push(out);
            }
            (output, input)
        };
        Ok((
            Self {
                path: None,
                expressions,
            },
            left_over,
        ))
    }

    fn parse_optional(mut input: &'i str) -> ParseResult<Option<Self>> {
        let (expressions, left_over) = {
            let mut output = vec![];
            while !input.is_empty() {
                let (out, left_over) = Block::parse_optional(input)?;

                let out = match out {
                    Some(out) => out,
                    None => return Ok((None, left_over)),
                };

                input = left_over;
                output.push(out);
            }
            (output, input)
        };
        Ok((
            Some(Self {
                path: None,
                expressions,
            }),
            left_over,
        ))
    }
}

impl Display for Template<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for ast in &self.expressions {
            ast.fmt(f)?;
        }
        Ok(())
    }
}
