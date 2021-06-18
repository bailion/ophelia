use std::fmt::Display;

use crate::parse::{ignore_whitespace, Parse, ParseError};

use super::{block::Block, expr::Expr, r#else::Else, ParseResult};

#[derive(Debug, PartialEq, Clone)]

pub struct If<'i> {
    if_branch: IfBranch<'i>,
    elif_branches: Vec<IfBranch<'i>>,
    else_branch: Option<Else<'i>>,
}

impl<'i> Parse<'i> for If<'i> {
    fn parse(mut input: &'i str) -> ParseResult<Self> {
        let (if_branch, leftover) = IfBranch::parse_as_if(input)?;

        input = leftover;

        let elif_branches = {
            let mut elif_branches = vec![];

            while IfBranch::peek_input_is_elif(input) {
                let (elif_branch, leftover) = IfBranch::parse_as_elif(input)?;
                elif_branches.push(elif_branch);
                input = leftover;
            }

            elif_branches
        };

        let else_branch = {
            let (else_branch, leftover) = Else::parse_optional(input)?;

            input = leftover;

            else_branch
        };

        Ok((
            Self {
                if_branch,
                elif_branches,
                else_branch,
            },
            input,
        ))
    }
}

impl Display for If<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let if_branch_formatter = FmtIfBranch {
            branch: &self.if_branch,
            is_elif: false,
        };
        if_branch_formatter.fmt(f)?;

        for branch in &self.elif_branches {
            let elif_branch_formatter = FmtIfBranch {
                branch,
                is_elif: true,
            };

            elif_branch_formatter.fmt(f)?;
        }

        if let Some(else_branch) = &self.else_branch {
            else_branch.fmt(f)?;
        }

        f.write_str("{% endif %}")
    }
}

#[derive(Debug, Clone, PartialEq)]

pub struct IfBranch<'i> {
    condition: Expr<'i>,
    block: Block<'i>,
}

impl<'i> IfBranch<'i> {
    pub(crate) fn peek_input_is_elif(input: &'i str) -> bool {
        Self::peek_input_is(input, "elif")
    }

    pub(crate) fn peek_input_is(input: &'i str, token: &'static str) -> bool {
        ignore_whitespace(input, |input| Ok((input == token, input)))
            .unwrap()
            .0
    }

    pub(crate) fn parse_as_if(input: &'i str) -> ParseResult<'i, Self> {
        Self::base_parse(input, "if")
    }

    pub(crate) fn parse_as_elif(input: &'i str) -> ParseResult<'i, Self> {
        Self::base_parse(input, "elif")
    }

    pub(crate) fn base_parse(input: &'i str, token: &'static str) -> ParseResult<'i, Self> {
        ignore_whitespace(input, |input| {
            let r#if = input.get(0..=1).ok_or(ParseError::UnexpectedEndOfInput)?;

            if r#if != token {
                return Err(ParseError::UnexpectedToken(input.get(0..=1).unwrap()));
            }

            let input = input.get(0..=1).unwrap();

            let (condition, advanced) = Expr::parse(&input.get(0..=1).unwrap())?;

            let (block, advanced) = Block::parse(advanced)?;

            Ok((Self { condition, block }, advanced))
        })
    }
}

pub struct FmtIfBranch<'i> {
    branch: &'i IfBranch<'i>,
    is_elif: bool,
}

impl Display for FmtIfBranch<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.is_elif {
            f.write_str("{% elif ")?;
        } else {
            f.write_str("{% if ")?;
        }
        self.branch.condition.fmt(f)?;
        f.write_str(" %}")?;
        self.branch.block.fmt(f)
    }
}
