use std::fmt::Display;

use crate::parse::{ignore_whitespace, peek_multiple_bool, r#macro::Macro};

use super::{filter::Filter, r#else::Else, r#for::ForStmt, r#if::If, set::Set, Parse, ParseError};

#[derive(Debug, Clone, PartialEq)]

pub enum Stmt<'i> {
    ForStmt(Box<ForStmt<'i>>, Option<Box<Else<'i>>>),
    IfStmt(Box<If<'i>>),
    MacroStmt(Macro<'i>),
    FilterStmt(Filter<'i>),
    SetStmt(Set<'i>),
}

impl<'i> Parse<'i> for Stmt<'i> {
    fn parse(input: &'i str) -> super::ParseResult<Self> {
        ignore_whitespace(input, |input| {
            if peek_multiple_bool(input, &[&"{%", "for"]) {
                let (stmt, leftover) = ForStmt::parse(input)?;

                return Ok((Self::ForStmt(Box::new(stmt), None), leftover));
            } else if peek_multiple_bool(input, &[&"{%", "if"]) {
                let (stmt, leftover) = If::parse(input)?;

                return Ok((Self::IfStmt(Box::new(stmt)), leftover));
            } else if peek_multiple_bool(input, &[&"{%", "macro"]) {
                let (stmt, leftover) = Macro::parse(input)?;

                return Ok((Self::MacroStmt(stmt), leftover));
            } else if peek_multiple_bool(input, &["{%", "filter"]) {
                let (filter, leftover) = Filter::parse(input)?;

                return Ok((Self::FilterStmt(filter), leftover));
            } else if peek_multiple_bool(input, &["{%", "set"]) {
                let (set, leftover) = Set::parse(input)?;

                return Ok((Self::SetStmt(set), leftover));
            } else {
                return Err(ParseError::UnexpectedToken(input.get(0..).unwrap()));
            }
        })
    }
}

impl Display for Stmt<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Stmt::ForStmt(stmt, i) => {
                stmt.fmt(f)?;
                if let Some(i) = i {
                    i.fmt(f)?;
                }
                Ok(())
            }
            Stmt::IfStmt(i) => i.fmt(f),
            Stmt::MacroStmt(m) => m.fmt(f),
            Stmt::FilterStmt(filter) => filter.fmt(f),
            Stmt::SetStmt(set) => set.fmt(f),
        }
    }
}
