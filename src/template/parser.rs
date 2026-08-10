use winnow::combinator::{alt, delimited, opt, preceded, repeat, separated};
use winnow::prelude::*;
use winnow::token::{literal, take_while};

#[derive(Debug, Clone, PartialEq)]
pub enum TemplateNode {
    Text(String),
    Variable {
        name: String,
        filters: Vec<FilterCall>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct FilterCall {
    pub name: String,
    pub args: Vec<FilterArg>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum FilterArg {
    Number(f64),
    String(String),
    Variable(String),
}

fn identifier(input: &mut &str) -> ModalResult<String> {
    take_while(1.., |c: char| c.is_alphanumeric() || c == '_')
        .map(|s: &str| s.to_string())
        .parse_next(input)
}

fn number_arg(input: &mut &str) -> ModalResult<FilterArg> {
    winnow::ascii::float
        .map(FilterArg::Number)
        .parse_next(input)
}

fn string_arg(input: &mut &str) -> ModalResult<FilterArg> {
    delimited(
        literal("\""),
        take_while(0.., |c: char| c != '"').map(|s: &str| s.to_string()),
        literal("\""),
    )
    .map(FilterArg::String)
    .parse_next(input)
}

fn bare_arg(input: &mut &str) -> ModalResult<FilterArg> {
    preceded(literal("%"), identifier.map(FilterArg::Variable)).parse_next(input)
}

fn filter_arg(input: &mut &str) -> ModalResult<FilterArg> {
    alt((
        number_arg,
        string_arg,
        bare_arg,
        identifier.map(FilterArg::String),
    ))
    .parse_next(input)
}

fn filter_args(input: &mut &str) -> ModalResult<Vec<FilterArg>> {
    delimited(
        literal("("),
        separated(
            0..,
            filter_arg,
            take_while(1.., |c: char| c.is_whitespace() || c == ','),
        ),
        literal(")"),
    )
    .parse_next(input)
}

fn filter_call(input: &mut &str) -> ModalResult<FilterCall> {
    let _ = literal(":").parse_next(input)?;
    let name = identifier.parse_next(input)?;
    let args = opt(filter_args).parse_next(input)?.unwrap_or_default();
    Ok(FilterCall { name, args })
}

fn variable_node(input: &mut &str) -> ModalResult<TemplateNode> {
    let _ = literal("{{").parse_next(input)?;
    let _ = take_while(0.., |c: char| c.is_whitespace()).parse_next(input)?;
    let name = identifier.parse_next(input)?;
    let filters = repeat(0.., filter_call).parse_next(input)?;
    let _ = take_while(0.., |c: char| c.is_whitespace()).parse_next(input)?;
    let _ = literal("}}").parse_next(input)?;
    Ok(TemplateNode::Variable { name, filters })
}

// Hand-rolled character scanner for raw text between template tags.
// Consumes characters until it finds `{{`, tracking byte positions manually
// so the remaining input slice is correctly positioned after extraction.
fn text_node(input: &mut &str) -> ModalResult<TemplateNode> {
    let original = *input;
    let mut consumed = 0usize;

    for c in input.chars() {
        if c == '{' {
            let rest = &input[consumed + 1..];
            if rest.starts_with('{') {
                break;
            }
        }
        consumed += c.len_utf8();
    }

    if consumed == 0 {
        return Err(winnow::error::ErrMode::Backtrack(
            winnow::error::ContextError::new(),
        ));
    }

    let text: String = original.chars().take(consumed).collect();
    *input = &original[consumed..];
    Ok(TemplateNode::Text(text))
}

fn template_node(input: &mut &str) -> ModalResult<TemplateNode> {
    alt((variable_node, text_node)).parse_next(input)
}

pub struct TemplateParser;

impl TemplateParser {
    pub fn parse(input: &str) -> Result<Vec<TemplateNode>, String> {
        if input.is_empty() {
            return Ok(vec![TemplateNode::Text(String::new())]);
        }
        repeat(1.., template_node).parse(input).map_err(
            |e: winnow::error::ParseError<&str, winnow::error::ContextError>| {
                format!("parse error: {e}")
            },
        )
    }
}
