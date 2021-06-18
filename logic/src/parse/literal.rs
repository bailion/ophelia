use std::{borrow::BorrowMut, fmt::Display};

use crate::parse::{
    ignore_whitespace, next, parse_multiple, parse_token, peek_token_bool, up_to, ParseError,
};

use super::{peek_multiple_bool, Parse, ParseResult};

/// A literal.
///
/// See https://jinja.palletsprojects.com/en/3.0.x/templates/#literals for more details.
#[derive(Debug, Clone, PartialEq)]

pub enum Literal<'i> {
    String(&'i str),
    Integer(i32),
    Float(f32),
    List(Vec<Literal<'i>>),
    Tuple(Vec<Literal<'i>>),
    Dict(Vec<(Literal<'i>, Literal<'i>)>),
    Bool(bool),
}

impl<'i> Parse<'i> for Literal<'i> {
    fn parse(input: &'i str) -> super::ParseResult<Self> {
        ignore_whitespace(input, |input| {
            if input.is_empty() {
                return Err(ParseError::UnexpectedEndOfInput);
            }

            if input.starts_with("[") {
                return parse_list(input).map(|(a, b)| (Self::List(a), b));
            }

            if input.starts_with("{") {
                return parse_dict(input).map(|(a, b)| (Self::Dict(a), b));
            }

            if input.starts_with("(") {
                return parse_tuple(input).map(|(a, b)| (Self::Tuple(a), b));
            }

            if input.starts_with("true") {
                return Ok((Self::Bool(true), input.get("true".len()..).unwrap()));
            }

            if input.starts_with("false") {
                return Ok((Self::Bool(false), input.get("false".len()..).unwrap()));
            }

            if input.starts_with("\"") || input.starts_with("'") {
                let (string, rest) = up_to(input.get(1..).unwrap(), &["\"", "'"])?;

                return Ok((Self::String(string), rest.get(1..).unwrap_or("")));
            };

            if input.chars().next().unwrap().is_digit(10) {
                let (parsed, rest) = NumberParser::parse(input)?;

                return Ok((
                    if parsed.is_float() {
                        Self::Float(parsed.into_float().unwrap())
                    } else {
                        Self::Integer(parsed.into_int().unwrap())
                    },
                    rest,
                ));
            }

            Err(ParseError::UnexpectedToken(input.get(0..).unwrap_or("")))
        })
    }
}

fn parse_list<'i>(input: &'i str) -> ParseResult<Vec<Literal<'i>>> {
    ignore_whitespace(input, |input| {
        let mut list = vec![];

        let (_, mut input) = parse_token(input, "[")?;

        loop {
            let (literal, rest) = up_to(input, &[",", "]"])?;

            input = rest;

            let (literal, _) = Literal::parse(literal)?;

            list.push(literal);

            if peek_token_bool(input, "]") {
                input = input.get(1..).unwrap_or("");
                break;
            }
        }

        Ok((list, input))
    })
}

fn parse_dict<'i>(input: &'i str) -> ParseResult<Vec<(Literal<'i>, Literal<'i>)>> {
    let (_, mut input) = parse_token(input, "{")?;

    let mut dict = vec![];

    if peek_multiple_bool(input, &[",", "}"]) {
        let (_, rest) = parse_multiple(input, &[",", ")"])?;
        input = rest;
        return Ok((dict, input));
    }
    if peek_token_bool(input, ")") {
        let (_, rest) = parse_token(input, ")")?;
        input = rest;
        return Ok((dict, input));
    }

    loop {
        let (key, rest) = Literal::parse(input)?;

        let (_, rest) = parse_token(rest, ":")?;

        input = rest;

        let (value, rest) = Literal::parse(input)?;

        input = rest;

        dict.push((key, value));

        let should_exit = peek_token_bool(input, "}") || peek_multiple_bool(input, &[",", "}"]);
        if should_exit {
            break;
        }
    }

    Ok((dict, input))
}

fn parse_tuple<'i>(input: &'i str) -> ParseResult<Vec<Literal<'i>>> {
    let (_, mut input) = parse_token(input, "(")?;

    let mut tuple = vec![];

    if peek_multiple_bool(input, &[",", ")"]) {
        let (_, rest) = parse_multiple(input, &[",", ")"])?;
        input = rest;
        return Ok((tuple, input));
    }
    if peek_token_bool(input, ")") {
        let (_, rest) = parse_token(input, ")")?;
        input = rest;
        return Ok((tuple, input));
    }

    loop {
        let (item, rest) = Literal::parse(input)?;

        input = rest;

        tuple.push(item);

        if peek_multiple_bool(input, &[",", ")"]) {
            let (_, rest) = parse_multiple(input, &[",", ")"])?;
            input = rest;
            break;
        }
        if peek_token_bool(input, ")") {
            let (_, rest) = parse_token(input, ")")?;
            input = rest;
            break;
        }
    }

    Ok((tuple, input))
}

pub struct NumberParser<'i> {
    state: NumberParserState,
    int_part: Option<(usize, usize)>,
    float_part: Option<(usize, usize)>,
    exponent_part: Option<(usize, usize)>,
    input: &'i str,
}

impl<'i> NumberParser<'i> {
    fn new(input: &'i str) -> Self {
        Self {
            state: NumberParserState::IntPart,
            int_part: None,
            float_part: None,
            exponent_part: None,
            input,
        }
    }

    /// If it's not a float, it's *probably* an integer (we'll see if this is true when testing.)
    fn is_float(&self) -> bool {
        self.float_part.is_some()
    }

    fn into_float(self) -> Option<f32> {
        if self.int_part.is_none() {
            return None;
        }

        let stop = if let Some((_, stop)) = self.exponent_part {
            stop
        } else if let Some((_, stop)) = self.float_part {
            stop
        } else {
            self.int_part.unwrap().1
        };

        let section = self.input.get(self.int_part.unwrap().0..stop).unwrap();
        section.parse::<f32>().ok()
    }

    fn into_int(self) -> Option<i32> {
        // todo: warn that exponent parts are discarded for integers (integers are not closed under
        // exponentiation, so this just avoids bugs – don't do lots of maths in your templates)
        if let Some((start, stop)) = self.int_part {
            let section = self.input.get(start..stop).unwrap();
            section.parse::<i32>().ok()
        } else {
            None
        }
    }
}

impl<'i> Parse<'i> for NumberParser<'i> {
    fn parse(input: &'i str) -> ParseResult<Self> {
        let mut myself = Self::new(input);

        let mut index = 0;

        'parse_loop: loop {
            match myself.state {
                NumberParserState::IntPart => {
                    if peek_token_bool(input, ".") {
                        myself.state = NumberParserState::FloatPart;
                        continue 'parse_loop;
                    }
                    if peek_token_bool(input, "e") {
                        myself.state = NumberParserState::ExponentPart;
                        continue 'parse_loop;
                    }

                    let c = next(input)?;
                    if !c.is_digit(10) {
                        break;
                    }

                    if let Some((_, stop)) = myself.int_part.borrow_mut() {
                        *stop += 1;
                    } else {
                        *myself.int_part.borrow_mut() = Some((index, index));
                    }

                    index += 1;
                }
                NumberParserState::FloatPart => {
                    if peek_token_bool(input, ".") {
                        myself.state = NumberParserState::FloatPart;
                        continue 'parse_loop;
                    }

                    let c = next(input)?;
                    if !c.is_digit(10) {
                        break;
                    }

                    if let Some((_, stop)) = myself.int_part.borrow_mut() {
                        *stop += 1;
                    } else {
                        *myself.int_part.borrow_mut() = Some((index, index));
                    }

                    index += 1;
                }
                NumberParserState::ExponentPart => {
                    let c = next(input)?;

                    if !c.is_digit(10) {
                        break;
                    }

                    if let Some((_, stop)) = myself.int_part.borrow_mut() {
                        *stop += 1;
                    } else {
                        *myself.int_part.borrow_mut() = Some((index, index));
                    }

                    index += 1;
                }
            }
        }

        if index == 0 {
            if input.is_empty() {
                return Err(ParseError::UnexpectedEndOfInput);
            } else {
                return Err(ParseError::UnexpectedToken(input.get(0..0).unwrap()));
            }
        }

        Ok((myself, input.get(index..).unwrap_or("")))
    }
}

enum NumberParserState {
    IntPart,
    FloatPart,
    ExponentPart,
}

impl Display for Literal<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Literal::String(string) => {
                f.write_str("\"")?;
                f.write_str(string)?;
                f.write_str("\"")
            }
            Literal::Integer(int) => int.fmt(f),
            Literal::Float(float) => float.fmt(f),
            Literal::List(l) => {
                f.write_str("[")?;
                for literal in l {
                    literal.fmt(f)?;
                    f.write_str(",")?;
                }
                f.write_str("]")
            }
            Literal::Tuple(t) => {
                f.write_str("(")?;
                for literal in t {
                    literal.fmt(f)?;
                    f.write_str(",")?;
                }
                f.write_str(")")
            }
            Literal::Dict(d) => {
                f.write_str("{")?;
                for (key, value) in d {
                    key.fmt(f)?;
                    f.write_str(":")?;
                    value.fmt(f)?;
                    f.write_str(",")?;
                }
                f.write_str("}")
            }
            Literal::Bool(b) => f.write_str(if *b { "true" } else { "false" }),
        }
    }
}
