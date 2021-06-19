use std::fmt::Display;

use crate::parse::{ignore_whitespace, peek_multiple_bool, r#macro::Macro};

use super::{
    filter::Filter, import::Import, include::Include, r#else::Else, r#for::ForStmt, r#if::If,
    set::Set, Parse, ParseError,
};

#[derive(Debug, Clone, PartialEq)]

pub enum Stmt<'i> {
    For(Box<ForStmt<'i>>, Option<Box<Else<'i>>>),
    If(Box<If<'i>>),
    Macro(Macro<'i>),
    Filter(Filter<'i>),
    Set(Set<'i>),
    Include(Include<'i>),
    Import(Import<'i>),
}

impl<'i> Parse<'i> for Stmt<'i> {
    fn parse(input: &'i str) -> super::ParseResult<Self> {
        ignore_whitespace(input, |input| {
            if peek_multiple_bool(input, &["{%", "for"]) {
                let (stmt, leftover) = ForStmt::parse(input)?;

                return Ok((Self::For(Box::new(stmt), None), leftover));
            } else if peek_multiple_bool(input, &["{%", "if"]) {
                let (stmt, leftover) = If::parse(input)?;

                return Ok((Self::If(Box::new(stmt)), leftover));
            } else if peek_multiple_bool(input, &["{%", "macro"]) {
                let (stmt, leftover) = Macro::parse(input)?;

                return Ok((Self::Macro(stmt), leftover));
            } else if peek_multiple_bool(input, &["{%", "filter"]) {
                let (filter, leftover) = Filter::parse(input)?;

                return Ok((Self::Filter(filter), leftover));
            } else if peek_multiple_bool(input, &["{%", "set"]) {
                let (set, leftover) = Set::parse(input)?;

                return Ok((Self::Set(set), leftover));
            } else if peek_multiple_bool(input, &["{%", "include"]) {
                let (include, leftover) = Include::parse(input)?;

                return Ok((Self::Include(include), leftover));
            } else if peek_multiple_bool(input, &["{%", "import"])
                || peek_multiple_bool(input, &["{%", "from"])
            {
                let (import, leftover) = Import::parse(input)?;

                return Ok((Self::Import(import), leftover));
            } else {
                return Err(ParseError::UnexpectedToken(input.get(0..).unwrap()));
            }
        })
    }
}

impl Display for Stmt<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Stmt::For(stmt, i) => {
                stmt.fmt(f)?;
                if let Some(i) = i {
                    i.fmt(f)?;
                }
                Ok(())
            }
            Stmt::If(i) => i.fmt(f),
            Stmt::Macro(m) => m.fmt(f),
            Stmt::Filter(filter) => filter.fmt(f),
            Stmt::Set(set) => set.fmt(f),
            Stmt::Include(i) => i.fmt(f),
            Stmt::Import(i) => i.fmt(f),
        }
    }
}
