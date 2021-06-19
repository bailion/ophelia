use std::fmt::Display;

use crate::parse::{skip, up_to_optional, ParseError};

use super::{expr::Expr, ignore_whitespace, stmt::Stmt, Parse};

#[derive(Debug, Clone, PartialEq)]

pub enum Block<'i> {
    /// Raw text, to be output as-is
    RawText(&'i str),
    /// An expression
    Expr(Expr<'i>),
    /// A statement
    Stmt(Stmt<'i>),
    /// A comment
    Comment(&'i str),
}

impl<'i> Parse<'i> for Block<'i> {
    fn parse_optional(input: &'i str) -> super::ParseResult<Option<Self>> {
        ignore_whitespace(input, |input| {
            if let Some(indicator) = input.get(0..2) {
                match indicator {
                    "{%" => Stmt::parse(input).map(|(a, b)| (Some(Self::Stmt(a)), b)),
                    "{{" => Expr::parse(input).map(|(a, b)| (Some(Self::Expr(a)), b)),
                    "{#" => skip(input, 2, |input| {
                        let (comment, rest) = up_to_optional(input, &["#}"])?;

                        let comment = match comment {
                            Some(t) => t,
                            None => return Ok((None, rest)),
                        };

                        if rest.len() < 2 {
                            return Err(ParseError::UnexpectedEndOfInput);
                        }

                        if rest.get(0..2).unwrap() != "#}" {
                            return Err(ParseError::UnexpectedToken(rest.get(0..2).unwrap()));
                        }

                        skip(rest, 2, |input| Ok((Some(Self::Comment(comment)), input)))
                    }),
                    _ => {
                        let (raw_string, rest) = up_to_optional(input, &["{%", "{{", "{#"])?;

                        let raw_string = match raw_string {
                            Some(t) => t,
                            None => return Ok((Some(Self::RawText(input)), "")),
                        };

                        Ok((Some(Self::RawText(raw_string)), rest))
                    }
                }
            } else {
                let (raw_string, rest) = up_to_optional(input, &["{%", "{{", "{#"])?;

                match raw_string {
                    Some(s) => Ok((Some(Self::RawText(s)), rest)),
                    None => Ok((Some(Self::RawText(input)), "")),
                }
            }
        })
    }

    fn parse(input: &'i str) -> super::ParseResult<Self> {
        let (ast, rest) = <Self as Parse>::parse_optional(input)?;
        Ok((
            match ast {
                Some(ast) => ast,
                None => return Err(ParseError::UnexpectedEndOfInput),
            },
            rest,
        ))
    }
}

impl<'i> Display for Block<'i> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Block::RawText(raw) => raw.fmt(f),
            Block::Expr(e) => e.fmt(f),
            Block::Stmt(s) => s.fmt(f),
            Block::Comment(c) => {
                f.write_str("{#")?;
                c.fmt(f)?;
                f.write_str("#}")
            }
        }
    }
}
