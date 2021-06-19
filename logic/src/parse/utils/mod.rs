pub mod bracketed;

pub trait Parse<'i>: Sized {
    fn parse(input: &'i str) -> ParseResult<Self>;

    fn parse_optional(input: &'i str) -> ParseResult<Option<Self>> {
        let (inner, leftover) = <Self as Parse>::parse(input)?;
        Ok((Some(inner), leftover))
    }
}

/// Result<(<type>, <characters read>), <error type>>
pub type ParseResult<'i, T> = Result<(T, &'i str), ParseError<'i>>;

#[derive(Debug)]
pub enum ParseError<'i> {
    UnexpectedToken(&'i str),
    UnexpectedEndOfInput,
    UndiagnosedError,
    OperatorUsedInExpressionPosition,
}

pub(crate) fn ignore_whitespace<'i, T, F>(input: &'i str, func: F) -> ParseResult<T>
where
    F: FnOnce(&'i str) -> ParseResult<T>,
{
    let mut index = 0;
    while let Some(string) = input.get(index..index + 1) {
        if string
            .chars()
            .next()
            .expect("this really should not happen")
            == ' '
        {
            index += 1;
        } else {
            return (func)(input.get(index..).expect("could not ignore whitespace"));
        }
    }
    (func)(input)
}

pub(crate) fn parse_token<'i>(input: &'i str, selector: &str) -> ParseResult<'i, &'i str> {
    ignore_whitespace(input, |input| {
        if input.len() < selector.len() {
            return Err(ParseError::UnexpectedEndOfInput);
        }

        let selection = if let Some(selection) = input.get(0..selector.len()) {
            selection
        } else {
            return Err(ParseError::UnexpectedToken(
                input
                    .get(0..input.char_indices().next().unwrap().0)
                    .unwrap(),
            ));
        };

        if selection == selector {
            Ok((
                input.get(0..selector.len()).unwrap(),
                input.get(selector.len()..).unwrap(),
            ))
        } else {
            Err(ParseError::UnexpectedToken(
                input.get(0..selector.len()).unwrap(),
            ))
        }
    })
}

pub(crate) fn parse_multiple<'i>(
    mut input: &'i str,
    selectors: &[&str],
) -> ParseResult<'i, Vec<&'i str>> {
    assert!(selectors.len() > 0);

    let mut output_segments = vec![];

    for selector in selectors {
        let (segment, rest) = parse_token(input, selector)?;
        output_segments.push(segment);
        input = rest;
    }

    Ok((output_segments, input))
}

/// Peeks for an **ASCII** token.
pub(crate) fn peek_token<'i>(input: &'i str, selector: &str) -> ParseResult<'i, bool> {
    let res = ignore_whitespace(input, |input| {
        Ok((
            {
                if input.len() < selector.len() {
                    return Err(ParseError::UnexpectedEndOfInput);
                } else {
                    if let Some(cmp) = input.get(0..selector.len()) {
                        cmp == selector
                    } else {
                        return Err(ParseError::UnexpectedToken(&input[..]));
                    }
                }
            },
            input,
        ))
    });

    res
}

pub(crate) fn peek_token_bool<'i>(input: &'i str, selector: &str) -> bool {
    peek_token(input, selector).unwrap_or((false, "")).0
}

pub(crate) fn peek_multiple<'i>(input: &'i str, selectors: &[&str]) -> ParseResult<'i, bool> {
    assert!(selectors.len() > 0);

    let mut cursor = input.get(0..).ok_or(ParseError::UnexpectedEndOfInput)?;
    let mut index = 0;
    let mut error = None;

    let predicate = selectors.iter().all(|selector| {
        if cursor.starts_with(selector) {
            index += selector.len();
            cursor = match input.get(index..) {
                Some(t) => t,
                None => {
                    error = Some(ParseError::UnexpectedEndOfInput);
                    return false;
                }
            };
            true
        } else {
            false
        }
    });

    match (predicate, error) {
        (predicate, None) => Ok((predicate, input)),
        (_, Some(e)) => Err(e),
    }
}

pub(crate) fn peek_multiple_bool<'i>(input: &'i str, selectors: &[&str]) -> bool {
    peek_multiple(input, selectors).unwrap_or((false, "")).0
}

pub(crate) fn up_to<'i>(mut input: &'i str, tokens: &[&str]) -> ParseResult<'i, &'i str> {
    let initial_input = input;
    let mut idx = 0;
    assert!(!tokens.is_empty());

    loop {
        if input.is_empty() {
            return Err(ParseError::UnexpectedEndOfInput);
        }

        for token in tokens {
            if input.starts_with(token) {
                return Ok((initial_input.get(..idx).unwrap(), input));
            }
        }

        let len = input.chars().next().unwrap().len_utf8();

        input = input.get(len..).unwrap();
        idx += len;
    }
}

pub(crate) fn up_to_optional<'i>(
    input: &'i str,
    tokens: &[&str],
) -> ParseResult<'i, Option<&'i str>> {
    match up_to(input, tokens) {
        Ok(t) => Ok((Some(t.0), t.1)),
        Err(ParseError::UnexpectedEndOfInput) => Ok((None, "")),
        Err(e) => Err(e),
    }
}

pub(crate) fn skip<'i, T, F: Fn(&'i str) -> ParseResult<T>>(
    input: &'i str,
    n: usize,
    op: F,
) -> ParseResult<T> {
    if input.len() <= n {
        Err(ParseError::UnexpectedEndOfInput)
    } else {
        (op)(input.get(n..).unwrap())
    }
}

pub(crate) fn next(input: &str) -> Result<char, ParseError> {
    input.chars().next().ok_or(ParseError::UnexpectedEndOfInput)
}

#[cfg(test)]
mod test_up_to {
    use super::*;

    #[test]
    fn test_non_zero_window() {
        let input = "{# Comment McCommentFace #} and then there were none";

        let (before, after) = up_to(input, &["#}"]).expect("failed to parse");

        assert_eq!(before, "{# Comment McCommentFace ");
        assert_eq!(after, "#} and then there were none");
    }
}

#[cfg(test)]
mod test_ignore_whitespace {
    use super::*;

    #[test]
    fn test_ignore_whitespace() {
        ignore_whitespace(" input", |input| {
            assert_eq!(input, "input");
            Ok(((), input))
        })
        .unwrap();

        ignore_whitespace("  input", |input| {
            assert_eq!(input, "input");
            Ok(((), input))
        })
        .unwrap();

        ignore_whitespace(" input and then some more", |input| {
            assert_eq!(input, "input and then some more");
            Ok(((), input))
        })
        .unwrap();
    }
}
