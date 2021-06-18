mod op;

use std::fmt::{Display, Write};

use crate::parse::{expr::op::Op, ignore_whitespace, parse_token, ParseError};

use self::op::{BinOpExpr, UnaryOpExpr};

use super::{
    ident::Ident, literal::Literal, parse_multiple, peek_multiple_bool, peek_token_bool, Parse,
    ParseResult,
};

#[derive(Debug, Clone, PartialEq)]

pub enum Expr<'i> {
    UnaryOp(Box<UnaryOpExpr<'i>>),
    BinOpExpr(Box<BinOpExpr<'i>>),
    Literal(Literal<'i>),
    Ident(Ident<'i>),
    FunctionCall(Ident<'i>, Vec<Expr<'i>>),
}

#[derive(Debug, Clone)]
enum ExprOpSum<'i> {
    Expr(Expr<'i>),
    Op(Op),
}

impl<'i> ExprOpSum<'i> {
    fn try_into_expr(self) -> Result<Expr<'i>, Self> {
        if let Self::Expr(v) = self {
            Ok(v)
        } else {
            Err(self)
        }
    }
}

impl<'i> Parse<'i> for Expr<'i> {
    fn parse(input: &'i str) -> super::ParseResult<Self> {
        if input.len() <= 2 {
            return Err(ParseError::UnexpectedEndOfInput);
        }
        let (res, input) = Self::parse_bp(input.get(2..).unwrap(), 0)?;
        if input.len() < 2 {
            return Err(ParseError::UnexpectedEndOfInput);
        }
        if input.get(0..2).unwrap() != "}}" {
            return Err(ParseError::UnexpectedToken(input.get(0..2).unwrap()));
        }
        Ok((res, input.get(2..).unwrap_or("")))
    }
}

impl<'i> Expr<'i> {
    fn parse_bp(input: &'i str, min_bp: u8) -> ParseResult<Self> {
        ignore_whitespace(input, |mut input| {
            let mut lhs = {
                if let Ok((ident, rest)) = Ident::parse(input) {
                    input = rest;

                    if peek_token_bool(input, "(") {
                        let mut args = vec![];
                        loop {
                            let (expr, rest) = Expr::parse(input)?;
                            args.push(expr);
                            input = rest;

                            if peek_token_bool(input, ")") {
                                let (_, rest) = parse_token(input, "(")?;
                                input = rest;
                                break;
                            } else if peek_multiple_bool(input, &[",", ")"]) {
                                let (_, rest) = parse_multiple(input, &[",", ")"])?;
                                input = rest;
                                break;
                            }
                        }
                        Some(ExprOpSum::Expr(Self::FunctionCall(ident, args)))
                    } else {
                        Some(ExprOpSum::Expr(Self::Ident(ident)))
                    }
                } else if let Ok((operator, rest)) = Op::parse(input) {
                    input = rest;
                    Some(ExprOpSum::Op(operator))
                } else if let Ok((literal, rest)) = Literal::parse(input) {
                    input = rest;
                    Some(ExprOpSum::Expr(Expr::Literal(literal)))
                } else if let Ok((_, rest)) = parse_token(input, "(") {
                    input = rest;

                    let (lhs, rest) = Expr::parse_bp(input, 0)?;

                    input = rest;

                    let (_, rest) = parse_token(input, ")")?;

                    input = rest;

                    Some(ExprOpSum::Expr(lhs))
                } else {
                    return Err(ParseError::UnexpectedToken(input.get(0..).unwrap()));
                }
            };

            loop {
                let (op, rest) = Op::parse(input)?;

                if !op.is_bin_op() {
                    return Err(ParseError::UnexpectedToken(input.get(0..=1).unwrap_or("")));
                }

                if let Some((l_bp, r_bp)) = op.binding_power(lhs.is_some()) {
                    if l_bp < min_bp {
                        break;
                    }

                    input = rest;

                    let (rhs, rest) = Expr::parse_bp(input, r_bp)?;

                    input = rest;

                    if let Some(old_lhs) = lhs {
                        let old_lhs = match old_lhs.try_into_expr() {
                            Ok(t) => t,
                            Err(_) => return Err(ParseError::OperatorUsedInExpressionPosition),
                        };

                        lhs = Some(ExprOpSum::Expr(Expr::BinOpExpr(Box::new(BinOpExpr::new(
                            op.try_into_bin_op().unwrap(),
                            old_lhs,
                            rhs,
                        )))));

                        continue;
                    } else {
                        lhs = Some(ExprOpSum::Expr(Expr::UnaryOp(
                            box (UnaryOpExpr::new(op.try_into_unary_op().unwrap(), rhs)),
                        )));
                    }
                }

                break;
            }

            lhs.ok_or(ParseError::UndiagnosedError)
                .and_then(|expr| match expr.try_into_expr() {
                    Ok(t) => Ok((t, input)),
                    Err(_) => Err(ParseError::UndiagnosedError),
                })
        })
    }
}

impl<'i> Display for Expr<'i> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Expr::UnaryOp(u) => u.fmt(f),
            Expr::BinOpExpr(b) => b.fmt(f),
            Expr::Literal(l) => l.fmt(f),
            Expr::Ident(i) => i.fmt(f),
            Expr::FunctionCall(name, args) => {
                name.fmt(f)?;
                f.write_char('(')?;
                for arg in args {
                    arg.fmt(f)?;
                    f.write_char(',')?;
                }
                f.write_char(')')
            }
        }
    }
}
