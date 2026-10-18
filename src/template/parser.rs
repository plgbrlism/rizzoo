use winnow::combinator::{alt, cut_err, delimited, opt, preceded, repeat, separated};
use winnow::prelude::*;
use winnow::token::{literal, take_while};

#[derive(Debug, Clone, PartialEq)]
pub enum TemplateNode {
    Text(String),
    Variable {
        name: String,
        filters: Vec<FilterCall>,
    },
    Conditional {
        condition: Condition,
        if_true: Vec<TemplateNode>,
        if_false: Vec<TemplateNode>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum Condition {
    Var(String),
    NotVar(String),
    Eq(String, String),
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

fn ws0<'i>(input: &mut &'i str) -> ModalResult<&'i str> {
    take_while(0.., |c: char| c.is_whitespace()).parse_next(input)
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

// `{{ var == "value" ? a : b }}`
// `{{ var ? a : b }}`
// `{{ !var ? a : b }}`
fn condition(input: &mut &str) -> ModalResult<Condition> {
    alt((
        preceded(literal("!"), identifier.map(Condition::NotVar)),
        comparison,
        identifier.map(Condition::Var),
    ))
    .parse_next(input)
}

fn comparison(input: &mut &str) -> ModalResult<Condition> {
    (identifier, ws0, literal("=="), ws0, identifier)
        .map(|(left, _, _, _, right)| Condition::Eq(left, right))
        .parse_next(input)
}

// A branch is a single expression: a variable (with optional filters) or a
// quoted string literal.
fn branch_expr(input: &mut &str) -> ModalResult<TemplateNode> {
    alt((
        string_arg.map(|s| match s {
            FilterArg::String(s) => TemplateNode::Text(s),
            _ => unreachable!("string_arg always yields String"),
        }),
        (identifier, repeat(0.., filter_call))
            .map(|(name, filters)| TemplateNode::Variable { name, filters }),
    ))
    .parse_next(input)
}

fn branches(input: &mut &str) -> ModalResult<(Vec<TemplateNode>, Vec<TemplateNode>)> {
    let _ = ws0(input)?;
    let if_true = branch_expr(input)?;
    // Require whitespace before the separator: `{{ x ? a : b }}`. Without it,
    // `a:b` would be consumed as a filter on `a`, making the syntax ambiguous.
    let _ = take_while(1.., |c: char| c.is_whitespace()).parse_next(input)?;
    let _ = literal(":").parse_next(input)?;
    let _ = ws0(input)?;
    let if_false = branch_expr(input)?;
    Ok((vec![if_true], vec![if_false]))
}

fn conditional_node(input: &mut &str) -> ModalResult<TemplateNode> {
    let _ = literal("{{").parse_next(input)?;
    let _ = take_while(0.., |c: char| c.is_whitespace()).parse_next(input)?;
    let condition = condition(input)?;
    let _ = take_while(0.., |c: char| c.is_whitespace()).parse_next(input)?;
    // The `?` must be present to qualify as a conditional. If absent, backtrack
    // so `variable_node` can handle `{{ x }}`.
    match opt(preceded(ws0, literal("?"))).parse_next(input)? {
        Some(_) => {
            let (if_true, if_false) = cut_err(branches).parse_next(input)?;
            let _ = ws0(input)?;
            let _ = literal("}}").parse_next(input)?;
            Ok(TemplateNode::Conditional {
                condition,
                if_true,
                if_false,
            })
        }
        None => Err(winnow::error::ErrMode::Backtrack(
            winnow::error::ContextError::new(),
        )),
    }
}

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

    *input = &original[consumed..];
    Ok(TemplateNode::Text(original[..consumed].to_string()))
}

fn template_node(input: &mut &str) -> ModalResult<TemplateNode> {
    alt((conditional_node, variable_node, text_node)).parse_next(input)
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
