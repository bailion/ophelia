//! Assignments (not set theory)

use std::fmt::{Display, Write};

use crate::parse::{ident::ident_list, parse_token};

use super::{ast::Ast, expr::Expr, ident::Ident, peek_token_bool, Parse};

#[derive(Debug, Clone, PartialEq)]
pub struct Set<'i> {
    idents: Vec<Ident<'i>>,
    data: SetData<'i>,
}

impl<'i> Parse<'i> for Set<'i> {
    fn parse(input: &'i str) -> super::ParseResult<Self> {
        let (_, input) = parse_token(input, "{%")?;
        let (_, input) = parse_token(input, "set")?;

        let (idents, input) = ident_list(input)?;

        if peek_token_bool(input, "%}") {
            let (_, input) = parse_token(input, "%}")?;

            let (ast, input) = Ast::parse(input)?;

            let (_, input) = parse_token(input, "{% endset %}")?;

            return Ok((
                Self {
                    idents,
                    data: SetData::Block(box (ast)),
                },
                input,
            ));
        }

        let (expr, input) = Expr::parse(input)?;

        let (_, input) = parse_token(input, "%}")?;

        Ok((
            Self {
                idents,
                data: SetData::Expr(expr),
            },
            input,
        ))
    }
}

impl Display for Set<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("{% set ")?;
        for (index, ident) in self.idents.iter().enumerate() {
            ident.fmt(f)?;
            if index > 0 && index != self.idents.len() - 1 {
                f.write_char(',')?;
            }
        }
        f.write_char(' ')?;
        self.data.fmt(f)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum SetData<'i> {
    Expr(Expr<'i>),
    Block(Box<Ast<'i>>),
}

impl Display for SetData<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SetData::Expr(expr) => {
                expr.fmt(f)?;
                f.write_str("%}")
            }
            SetData::Block(ast) => {
                f.write_str("%}")?;
                ast.fmt(f)?;
                f.write_str("{% endset %}")
            }
        }
    }
}
