use std::fmt::{Display, Formatter, Write};

use crate::parse::{bracketed::parse_bracketed, ident::Ident, parse_token, peek_token_bool};

use super::{ast::Ast, expr::Expr, Parse};

pub struct Call<'i> {
    fn_name: Ident<'i>,
    fn_args: Vec<Expr<'i>>,
    args: Vec<Expr<'i>>,
    block: Ast<'i>,
}

impl<'i> Parse<'i> for Call<'i> {
    fn parse(input: &'i str) -> super::ParseResult<Self> {
        let (_, input) = parse_token(input, "{%")?;
        let (_, mut input) = parse_token(input, "call")?;

        let args = if peek_token_bool(input, "(") {
            let (arg_list, rest) = parse_bracketed(input, ",")?;
            input = rest;
            arg_list
        } else {
            vec![]
        };

        let (fn_name, input) = Ident::parse(input)?;

        let (fn_args, input) = parse_bracketed(input, ",")?;

        let (_, input) = parse_token(input, "%}")?;

        let (block, input) = Ast::parse(input)?;

        let (_, input) = parse_token(input, "{%")?;
        let (_, input) = parse_token(input, "endcall")?;
        let (_, input) = parse_token(input, "%}")?;

        Ok((
            Self {
                fn_name,
                fn_args,
                args,
                block,
            },
            input,
        ))
    }
}

impl Display for Call<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("{% call")?;

        if self.args.is_empty() {
            f.write_char(' ')?;
        } else {
            write_args_bracketed(self.args.iter(), f)?;
        }

        self.fn_name.fmt(f)?;

        if self.fn_args.is_empty() {
            f.write_str("()")?;
        } else {
            write_args_bracketed(self.fn_args.iter(), f)?;
        }

        f.write_str("%}")?;

        self.block.fmt(f)?;

        f.write_str("{% endcall %}")
    }
}

fn write_args_bracketed(
    args: impl Iterator<Item = impl Display>,
    f: &mut Formatter,
) -> std::fmt::Result {
    f.write_char('(')?;
    for arg in args {
        arg.fmt(f)?;
        f.write_char(',')?;
    }
    f.write_char(')')
}
