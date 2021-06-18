use std::fmt::Display;

use crate::parse::{ignore_whitespace, up_to};

use super::{ast::Ast, expr::Expr, ident::Ident, parse_token, Parse};

#[derive(Debug, Clone, PartialEq)]

pub struct ForStmt<'i> {
    idents_of_iter: Vec<Ident<'i>>,
    in_expr: Expr<'i>,
    block: Ast<'i>,
}

impl<'i> Parse<'i> for ForStmt<'i> {
    fn parse(input: &'i str) -> super::ParseResult<Self> {
        let (_, input) = ignore_whitespace(input, |input| parse_token(input, "{%"))?;

        let (_, mut input) = parse_token(input, "for")?;

        let mut idents_of_iter = vec![];

        loop {
            let (_, leftover) = up_to(input, &[",", "in"])?;

            let (ident, rest) = Ident::parse(input)?;

            input = rest;

            idents_of_iter.push(ident);

            let input_is_empty =
                ignore_whitespace(leftover, |input| Ok((input.is_empty(), input)))?.0;

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

        let (in_expr, input) = Expr::parse(input)?;

        let (_, input) = parse_token(input, "%}")?;

        let (block, input) = Ast::parse(input)?;

        // todo: make parsing more forgiving
        let (_, input) = parse_token(input, "{% endfor %}")?;

        Ok((
            Self {
                idents_of_iter,
                in_expr,
                block,
            },
            input,
        ))
    }
}

impl Display for ForStmt<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("{% for ")?;
        for ident in &self.idents_of_iter {
            ident.fmt(f)?;
            f.write_str(",")?;
        }
        f.write_str(" in ")?;
        self.in_expr.fmt(f)?;
        f.write_str("%}")?;
        self.block.fmt(f)?;
        f.write_str("{% endfor %}")
    }
}
