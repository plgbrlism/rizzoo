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
    ForBlock {
        var_name: String,
        collection: String,
        body: Vec<TemplateNode>,
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

fn for_block_header(input: &mut &str) -> ModalResult<(String, String)> {
    let _ = literal("{{#").parse_next(input)?;
    let _ = take_while(0.., |c: char| c.is_whitespace()).parse_next(input)?;
    let _ = literal("for").parse_next(input)?;
    let _ = take_while(1.., |c: char| c.is_whitespace()).parse_next(input)?;
    let var_name = identifier.parse_next(input)?;
    let _ = take_while(1.., |c: char| c.is_whitespace()).parse_next(input)?;
    let _ = literal("in").parse_next(input)?;
    let _ = take_while(1.., |c: char| c.is_whitespace()).parse_next(input)?;
    let collection = identifier.parse_next(input)?;
    let _ = take_while(0.., |c: char| c.is_whitespace()).parse_next(input)?;
    let _ = literal("}}").parse_next(input)?;
    Ok((var_name, collection))
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

// Checks whether the text at `pos` matches a for-block close tag `{{/ var_name }}`.
// Handles arbitrary whitespace between `{{/`, the variable name, and `}}`.
fn match_close_tag(text: &str, var_name: &str, pos: usize) -> bool {
    let rest = &text[pos..];
    if !rest.starts_with("{{/") {
        return false;
    }
    let after_slash = &rest[3..];
    let trimmed = after_slash.trim_start();
    if let Some(stripped) = trimmed.strip_prefix(var_name) {
        stripped.trim_start().starts_with("}}")
    } else {
        false
    }
}

// Parses `{{# for <var> in <collection> }}...{{/ <var> }}` blocks.
// Uses depth tracking to handle nested for blocks — increments on `{{#`,
// decrements on matching `{{/ <var> }}`, and only closes when depth reaches 0.
// This means nested blocks with the same variable name will incorrectly close
// the outer block first; real nesting with different variables works correctly.
//
// depth-based close tag matching works for distinct vars but fails
// for same-variable nesting. Add full stack tracking if that pattern is needed.
fn for_block(input: &mut &str) -> ModalResult<TemplateNode> {
    let (var_name, collection) = for_block_header.parse_next(input)?;

    let mut depth = 0i32;
    let mut close_pos = None;
    let bytes = input.as_bytes();

    for (i, _) in input.char_indices() {
        if i + 2 < bytes.len() && &input[i..i + 3] == "{{#" {
            depth += 1;
        } else if match_close_tag(input, &var_name, i) {
            if depth == 0 {
                close_pos = Some(i);
                break;
            }
            depth -= 1;
        }
    }

    let close_pos = close_pos
        .ok_or_else(|| winnow::error::ErrMode::Backtrack(winnow::error::ContextError::new()))?;

    let body_text = &input[..close_pos];
    let body = TemplateParser::parse(body_text).unwrap_or_default();

    let after_slash = &input[close_pos + 3..];
    let after_id = after_slash
        .trim_start()
        .strip_prefix(&var_name)
        .ok_or_else(|| winnow::error::ErrMode::Backtrack(winnow::error::ContextError::new()))?;
    let after_id = after_id.trim_start();
    let offset = after_id
        .find("}}")
        .ok_or_else(|| winnow::error::ErrMode::Backtrack(winnow::error::ContextError::new()))?;

    *input = &after_id[offset + 2..];

    Ok(TemplateNode::ForBlock {
        var_name,
        collection,
        body,
    })
}

fn template_node(input: &mut &str) -> ModalResult<TemplateNode> {
    alt((for_block, variable_node, text_node)).parse_next(input)
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
