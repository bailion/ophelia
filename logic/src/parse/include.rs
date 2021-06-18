use std::fmt::Display;

use crate::parse::parse_multiple;

use super::{expr::Expr, peek_token_bool, Parse};

#[derive(Debug, Clone, PartialEq)]
pub struct Include<'i> {
    files: Expr<'i>,
    // `false` by default
    ignore_missing: bool,
    // `true` by default
    with_context: bool,
}

impl<'i> Parse<'i> for Include<'i> {
    fn parse(input: &'i str) -> super::ParseResult<Self> {
        let (_, input) = parse_multiple(input, &["{%", "include"])?;

        let (files, mut input) = Expr::parse(input)?;

        let ignore_missing = if peek_token_bool(input, "ignore") {
            let (_, rest) = parse_multiple(input, &["ignore", "missing"])?;
            input = rest;
            true
        } else {
            false
        };

        let with_context = if peek_token_bool(input, "without") {
            let (_, rest) = parse_multiple(input, &["with", "context"])?;
            input = rest;
            true
        } else if peek_token_bool(input, "with") {
            let (_, rest) = parse_multiple(input, &["without", "context"])?;
            input = rest;
            false
        } else {
            true
        };

        Ok((
            Self {
                files,
                ignore_missing,
                with_context,
            },
            input,
        ))
    }
}

impl Display for Include<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("{% include")?;
        self.files.fmt(f)?;
        if self.ignore_missing {
            f.write_str("ignore missing")?;
        }
        if self.with_context {
            f.write_str("with context")?;
        }
        f.write_str("%}")?;
        todo!()
    }
}
