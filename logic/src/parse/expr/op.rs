use std::fmt::Display;

use crate::parse::{parse_token, Parse};

use super::Expr;

#[derive(Debug, Clone, PartialEq)]

pub struct UnaryOpExpr<'i> {
    operator: UnaryOp,
    arg: Expr<'i>,
}

impl<'i> UnaryOpExpr<'i> {
    pub fn new(operator: UnaryOp, arg: Expr<'i>) -> Self {
        Self { operator, arg }
    }
}

impl Display for UnaryOpExpr<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.operator.fmt(f)?;
        self.arg.fmt(f)
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]

pub enum UnaryOp {
    Not,
}

impl Display for UnaryOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UnaryOp::Not => f.write_str("not "),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]

pub struct BinOpExpr<'i> {
    operator: BinOp,
    arg1: Expr<'i>,
    arg2: Expr<'i>,
}

impl Display for BinOpExpr<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.arg1.fmt(f)?;
        self.operator.fmt(f)?;
        self.arg2.fmt(f)
    }
}

impl<'i> BinOpExpr<'i> {
    pub fn new(operator: BinOp, arg1: Expr<'i>, arg2: Expr<'i>) -> Self {
        Self {
            operator,
            arg1,
            arg2,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]

pub enum BinOp {
    /// Addition (+)
    Add,
    /// Subtraction (-)
    Sub,
    /// Division (/)
    Div,
    /// Integer division (//)
    IntDiv,
    /// Integer division (%)
    Mod,
    /// Multiplication (*)
    Mul,
    /// The exponent (`a^b` or `a**b` as you would write in Python notation)
    Exp,
    /// ==
    Eq,
    /// !=
    NotEq,
    /// Greater than
    Gt,
    /// Less than
    Lt,
    /// Greater than or equal to
    GtEq,
    /// Less than or equal to,
    LtEq,
    /// Boolean AND
    And,
    /// Boolean OR
    Or,
    /// `a IN b`
    In,
    /// `a IS b`
    Is,
    /// Applies a filter (source token: `|`)
    Pipe,
    /// Concatenates arguments `~`
    Tilde,
    /// Dot
    Dot,
}

impl Display for BinOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            BinOp::Add => "+",
            BinOp::Sub => "-",
            BinOp::Div => "/",
            BinOp::IntDiv => "//",
            BinOp::Mod => "%",
            BinOp::Mul => "*",
            BinOp::Exp => "**",
            BinOp::Eq => "==",
            BinOp::NotEq => "!=",
            BinOp::Gt => ">",
            BinOp::Lt => "<",
            BinOp::GtEq => ">=",
            BinOp::LtEq => "<=",
            BinOp::And => "and",
            BinOp::Or => "or",
            BinOp::In => "in",
            BinOp::Is => "is",
            BinOp::Pipe => "|",
            BinOp::Tilde => "~",
            BinOp::Dot => ".",
        })
    }
}

#[derive(Copy, Clone, Debug)]
pub(crate) enum Op {
    BinOp(BinOp),
    UnaryOp(UnaryOp),
}

impl Op {
    /// The "binding power" of an operator is a measure of operator precedence.
    ///
    /// See <https://matklad.github.io/2020/04/13/simple-but-powerful-pratt-parsing.html> for more
    /// details.
    pub(crate) fn binding_power(&self, prefix: bool) -> Option<(u8, u8)> {
        Some(match *self {
            Op::BinOp(BinOp::Add | BinOp::Sub) if prefix => (99, 9),
            Op::BinOp(BinOp::Add | BinOp::Sub) => (5, 6),
            Op::BinOp(
                BinOp::Mul | BinOp::Div | BinOp::IntDiv | BinOp::Mod | BinOp::Pipe | BinOp::Tilde,
            ) => (7, 8),
            Op::BinOp(
                BinOp::And
                | BinOp::Eq
                | BinOp::Or
                | BinOp::GtEq
                | BinOp::Lt
                | BinOp::Gt
                | BinOp::LtEq
                | BinOp::Is
                | BinOp::NotEq
                | BinOp::In,
            ) => (9, 10),
            Op::BinOp(BinOp::Exp) => (11, 12),
            Op::BinOp(BinOp::Dot) => (14, 13),
            Op::UnaryOp(UnaryOp::Not) => (11, 100),
        })
    }

    /// Returns `true` if the op is [`BinOp`].
    pub(crate) fn is_bin_op(&self) -> bool {
        matches!(self, Self::BinOp(..))
    }

    pub(crate) fn try_into_bin_op(self) -> Result<BinOp, Self> {
        if let Self::BinOp(v) = self {
            Ok(v)
        } else {
            Err(self)
        }
    }

    pub(crate) fn try_into_unary_op(self) -> Result<UnaryOp, Self> {
        if let Self::UnaryOp(v) = self {
            Ok(v)
        } else {
            Err(self)
        }
    }
}

macro_rules! parse_operators {
    ($input:ident: $($op:expr => $item:expr),+) => {
        {
            $(
                if $crate::parse::utils::peek_token_bool($input, $op) {
                    let (_, rest) = parse_token($input, $op)?;

                    return Ok(($item, rest));
                }
            )*
            Err($crate::parse::ParseError::UnexpectedToken(
                    $input
                        .get(0..2)
                        .unwrap_or_else(|| $input.get(0..1).unwrap_or("")),
            ))
        }
    };
}

impl<'i> Parse<'i> for Op {
    fn parse(input: &'i str) -> crate::parse::ParseResult<Self> {
        // **important**: if you update this, make sure to update `BinOp` and `UnaryOp`'s `Parse`
        // implementations too!!!
        parse_operators!(
            input:
                // binary operators
                "+" => Op::BinOp(BinOp::Add),
                "-" => Op::BinOp(BinOp::Sub),
                "//" => Op::BinOp(BinOp::IntDiv),
                "/" => Op::BinOp(BinOp::Div),
                "%" => Op::BinOp(BinOp::Mod),
                "*" => Op::BinOp(BinOp::Mul),
                "**" => Op::BinOp(BinOp::Exp),
                "==" => Op::BinOp(BinOp::Eq),
                "!=" => Op::BinOp(BinOp::NotEq),
                ">=" => Op::BinOp(BinOp::GtEq),
                "<=" => Op::BinOp(BinOp::LtEq),
                "<" => Op::BinOp(BinOp::Gt),
                ">" => Op::BinOp(BinOp::Lt),
                "and" => Op::BinOp(BinOp::And),
                "or" => Op::BinOp(BinOp::Or),
                "in" => Op::BinOp(BinOp::In),
                "is" => Op::BinOp(BinOp::Is),
                "|" => Op::BinOp(BinOp::Pipe),
                "~" => Op::BinOp(BinOp::Tilde),
                "." => Op::BinOp(BinOp::Dot),
                // unary operators
                "not" => Op::UnaryOp(UnaryOp::Not)
        )
    }
}
