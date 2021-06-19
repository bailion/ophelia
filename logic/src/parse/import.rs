use std::fmt::{Display, Formatter, Write};

use crate::parse::{bracketed::parse_delimited, parse_multiple, parse_token, peek_token_bool};

use super::{expr::Expr, ident::Ident, Parse, ParseError, ParseResult};

#[derive(Debug, Clone, PartialEq)]
pub struct Import<'i> {
    file: Expr<'i>,
    r#as: Option<Vec<Ident<'i>>>,
    items: Items<'i>,
    with_context: bool,
}

impl<'i> Parse<'i> for Import<'i> {
    fn parse(input: &'i str) -> super::ParseResult<Self> {
        let (_, input) = parse_token(input, "{%")?;
        if peek_token_bool(input, "from") {
            Self::parse_from(input)
        } else if peek_token_bool(input, "import") {
            Self::parse_vanilla(input)
        } else {
            Err(ParseError::UnexpectedToken(input))
        }
    }
}

impl<'i> Import<'i> {
    fn parse_from(input: &'i str) -> ParseResult<Self> {
        let (_, input) = parse_token(input, "from")?;

        let (file, _) = Expr::parse(input)?;

        let (_, input) = parse_token(input, "import")?;

        let (items, input) = parse_delimited::<Ident>(input, ",")?;

        if items.is_empty() {
            // todo: return a proper error message
            return Err(ParseError::UnexpectedEndOfInput);
        }

        let (r#as, input) = if peek_token_bool(input, "as") {
            let (_, input) = parse_token(input, "as")?;

            let (r#as, input) = parse_delimited::<Ident>(input, ",")?;
            (Some(r#as), input)
        } else {
            (None, input)
        };

        let (with_context, input) = Self::with_context(input)?;

        Ok((
            Self {
                file,
                r#as,
                items: Items::List(items),
                with_context,
            },
            input,
        ))
    }

    fn parse_vanilla(input: &'i str) -> ParseResult<Self> {
        let (_, input) = parse_token(input, "import")?;

        let (file, input) = Expr::parse(input)?;

        let (_, input) = parse_token(input, "as")?;

        let (r#as, input) = parse_delimited::<Ident>(input, ",")?;

        if r#as.is_empty() {
            return Err(ParseError::UnexpectedEndOfInput);
        }

        let (with_context, input) = Self::with_context(input)?;

        Ok((
            Self {
                file,
                r#as: Some(r#as),
                items: Items::All,
                with_context,
            },
            input,
        ))
    }

    fn with_context(input: &'i str) -> ParseResult<bool> {
        Ok(if peek_token_bool(input, "without") {
            let (_, input) = parse_multiple(input, &["without", "context"])?;
            (false, input)
        } else if peek_token_bool(input, "with") {
            let (_, input) = parse_multiple(input, &["with", "context"])?;
            (true, input)
        } else {
            (true, input)
        })
    }

    fn write_as(&self, f: &mut Formatter) -> std::fmt::Result {
        if let Some(ref r#as) = self.r#as {
            f.write_str(" as ")?;
            for (i, ident) in r#as.iter().enumerate() {
                ident.fmt(f)?;
                if i != 0 && i != (r#as.len() - 1) {
                    f.write_char(',')?;
                }
            }
        }
        Ok(())
    }

    fn write_context(&self, f: &mut Formatter) -> std::fmt::Result {
        if !self.with_context {
            f.write_str(" without context ")
        } else {
            Ok(())
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Items<'i> {
    All,
    List(Vec<Ident<'i>>),
}

impl<'i> Display for Import<'i> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.items {
            Items::All => {
                f.write_str("{% import ")?;
                self.file.fmt(f)?;
                self.write_as(f)?;
                self.write_context(f)?;
                f.write_str("%}")
            }
            Items::List(items) => {
                f.write_str("{% from ")?;

                self.file.fmt(f)?;

                f.write_str(" import ")?;

                for (i, item) in items.iter().enumerate() {
                    item.fmt(f)?;
                    if i != 0 && i != (items.len() - 1) {
                        f.write_char(',')?;
                    }
                }

                self.write_as(f)?;
                self.write_context(f)?;
                f.write_str("%}")
            }
        }
    }
}
