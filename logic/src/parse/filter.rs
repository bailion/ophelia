use std::fmt::Display;

use super::{ast::Ast, ident::Ident, parse_token, Parse};

#[derive(Debug, Clone, PartialEq)]
pub struct Filter<'i> {
    name: Ident<'i>,
    block: Box<Ast<'i>>,
}

impl<'i> Parse<'i> for Filter<'i> {
    fn parse(input: &'i str) -> super::ParseResult<Self> {
        let (_, input) = parse_token(input, "{%")?;
        let (_, input) = parse_token(input, "filter")?;

        let (name, input) = Ident::parse(input)?;

        let (_, input) = parse_token(input, "%}")?;

        let (block, input) = Ast::parse(input)?;
        let block = box (block);

        let (_, input) = parse_token(input, "{%")?;
        let (_, input) = parse_token(input, "endfilter")?;
        let (_, input) = parse_token(input, "%}")?;

        Ok((Self { name, block }, input))
    }
}

impl Display for Filter<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("{% filter ")?;
        self.name.fmt(f)?;
        f.write_str("%}")?;

        self.block.fmt(f)?;

        f.write_str("{% endfilter %}")
    }
}
