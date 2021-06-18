use std::fmt::{Display, Write};

use crate::parse::{peek_multiple, peek_token};

use super::{block::Block, expr::Expr, ident::Ident, parse_token, Parse, ParseResult};

#[derive(Clone, PartialEq, Debug)]

pub struct Macro<'i> {
    name: Ident<'i>,
    args: Vec<Ident<'i>>,
    kwargs: Vec<(Ident<'i>, Expr<'i>)>,
    ast: Box<Block<'i>>,
}

impl<'i> Parse<'i> for Macro<'i> {
    fn parse(input: &'i str) -> super::ParseResult<Self> {
        // todo: make parsing more forgiving
        let (_, input) = parse_token(input, "{% macro")?;

        let (name, input) = Ident::parse(input)?;

        let (args, input) = ArgParser::parse(input)?;

        let (args, kwargs) = args.take();

        let (_, input) = parse_token(input, "-%}")?;

        let (ast, input) = Block::parse(input)?;
        let ast = box (ast);

        let (_, input) = parse_token(input, "{%-")?;

        let (_, input) = parse_token(input, "endmacro")?;

        let (_, input) = parse_token(input, "%}")?;

        Ok((
            Self {
                name,
                args,
                kwargs,
                ast,
            },
            input,
        ))
    }
}

impl Display for Macro<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("{% macro ")?;

        self.name.fmt(f)?;

        f.write_str("{%- endmacro %}")
    }
}

struct ArgFmt<'i> {
    args: Vec<Ident<'i>>,
    kwargs: Vec<(Ident<'i>, Expr<'i>)>,
}

impl Display for ArgFmt<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for arg in &self.args {
            arg.fmt(f)?;
            f.write_char(',')?;
        }
        for (ident, expr) in &self.kwargs {
            ident.fmt(f)?;
            f.write_char('=')?;
            expr.fmt(f)?;
            f.write_char(',')?;
        }
        Ok(())
    }
}

struct ArgParser<'i> {
    state: ArgParserState,
    pub(crate) args: Vec<Ident<'i>>,
    pub(crate) kwargs: Vec<(Ident<'i>, Expr<'i>)>,
}

impl<'i> ArgParser<'i> {
    fn new() -> Self {
        Self {
            state: ArgParserState::Args,
            args: Default::default(),
            kwargs: Default::default(),
        }
    }

    fn take(self) -> (Vec<Ident<'i>>, Vec<(Ident<'i>, Expr<'i>)>) {
        (self.args, self.kwargs)
    }
}

impl<'i> Parse<'i> for ArgParser<'i> {
    fn parse(input: &'i str) -> super::ParseResult<Self> {
        let mut myself = ArgParser::new();

        let (_, mut input) = parse_token(input, "(")?;

        'parse_loop: loop {
            match myself.state {
                ArgParserState::Args => {
                    let (ident, rest) = Ident::parse(input)?;

                    input = rest;

                    if peek_token(input, "=").unwrap_or((false, "")).0 {
                        let (expr, rest) = Expr::parse(input)?;
                        input = rest;

                        myself.kwargs.push((ident, expr));

                        myself.state = ArgParserState::VarArgs;
                        continue 'parse_loop;
                    }

                    myself.args.push(ident);

                    let (should_exit, _) =
                        match (peek_token(input, ")"), peek_multiple(input, &[",", ")"])) {
                            (_, Ok(m)) => m,
                            (Ok(m), Err(_)) => m,
                            (Err(_), Err(e)) => return Err(e),
                        };

                    if should_exit {
                        break;
                    }
                }
                ArgParserState::VarArgs => {
                    let (arg, rest) = parse_kwarg(input)?;
                    myself.kwargs.push(arg);
                    input = rest;

                    let (should_exit, _) =
                        match (peek_token(input, ")"), peek_multiple(input, &[",", ")"])) {
                            (_, Ok(m)) => m,
                            (Ok(m), Err(_)) => m,
                            (Err(_), Err(e)) => return Err(e),
                        };

                    if should_exit {
                        break;
                    }
                }
            }
        }

        Ok((myself, input))
    }
}

enum ArgParserState {
    Args,
    VarArgs,
}

fn parse_kwarg<'i>(input: &'i str) -> ParseResult<(Ident<'i>, Expr<'i>)> {
    let (ident, input) = Ident::parse(input)?;

    let (expr, input) = Expr::parse(input)?;

    Ok(((ident, expr), input))
}
