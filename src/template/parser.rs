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

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(input: &str) -> Vec<TemplateNode> {
        TemplateParser::parse(input).expect(input)
    }

    #[test]
    fn plain_text() {
        assert_eq!(parse("hello world"), vec![TemplateNode::Text("hello world".into())]);
    }

    #[test]
    fn empty_input() {
        assert_eq!(parse(""), vec![TemplateNode::Text(String::new())]);
    }

    #[test]
    fn simple_variable() {
        assert_eq!(
            parse("{{ primary }}"),
            vec![TemplateNode::Variable { name: "primary".into(), filters: vec![] }]
        );
    }

    #[test]
    fn variable_without_spaces() {
        assert_eq!(
            parse("{{primary}}"),
            vec![TemplateNode::Variable { name: "primary".into(), filters: vec![] }]
        );
    }

    #[test]
    fn filter_no_space_before_colon() {
        assert_eq!(
            parse("{{ primary:hex_raw }}"),
            vec![TemplateNode::Variable {
                name: "primary".into(),
                filters: vec![FilterCall { name: "hex_raw".into(), args: vec![] }],
            }]
        );
    }

    #[test]
    fn filter_chain() {
        let nodes = parse("{{ primary:hex_raw }}");
        match &nodes[0] {
            TemplateNode::Variable { filters, .. } => {
                assert_eq!(filters.len(), 1);
                assert_eq!(filters[0].name, "hex_raw");
            }
            other => panic!("expected variable, got {other:?}"),
        }
    }

    #[test]
    fn filter_number_arg() {
        let nodes = parse("{{ primary:lighten(0.1) }}");
        match &nodes[0] {
            TemplateNode::Variable { filters, .. } => {
                assert_eq!(filters[0].args, vec![FilterArg::Number(0.1)]);
            }
            other => panic!("expected variable, got {other:?}"),
        }
    }

    #[test]
    fn filter_string_arg() {
        let nodes = parse(r##"{{ primary:harmonize("#abc123") }}"##);
        match &nodes[0] {
            TemplateNode::Variable { filters, .. } => {
                assert_eq!(filters[0].args, vec![FilterArg::String("#abc123".into())]);
            }
            other => panic!("expected variable, got {other:?}"),
        }
    }

    #[test]
    fn filter_bare_var_and_number_args() {
        // real-world shape: {{ on_surface:ensure_contrast(%surface, 4.5) }}
        let nodes = parse("{{ on_surface:ensure_contrast(%surface, 4.5) }}");
        match &nodes[0] {
            TemplateNode::Variable { filters, .. } => {
                assert_eq!(
                    filters[0].args,
                    vec![FilterArg::Variable("surface".into()), FilterArg::Number(4.5)]
                );
            }
            other => panic!("expected variable, got {other:?}"),
        }
    }

    #[test]
    fn whitespace_tolerance_in_variable() {
        assert_eq!(
            parse("{{  primary  }}"),
            vec![TemplateNode::Variable { name: "primary".into(), filters: vec![] }]
        );
    }

    #[test]
    fn unclosed_variable_errors() {
        assert!(TemplateParser::parse("{{ primary").is_err());
    }

    #[test]
    fn text_then_variable() {
        let nodes = parse("bg = {{ primary }}");
        assert_eq!(nodes.len(), 2);
        assert_eq!(nodes[0], TemplateNode::Text("bg = ".into()));
        assert!(matches!(nodes[1], TemplateNode::Variable { ref name, .. } if name == "primary"));
    }

    #[test]
    fn conditional_variable_branches() {
        let nodes = parse("{{ mode == dark ? primary : primary_light }}");
        match &nodes[0] {
            TemplateNode::Conditional { condition, if_true, if_false } => {
                assert_eq!(*condition, Condition::Eq("mode".into(), "dark".into()));
                assert_eq!(if_true.len(), 1);
                assert_eq!(if_false.len(), 1);
                assert!(matches!(&if_true[0], TemplateNode::Variable { name, .. } if name == "primary"));
                assert!(matches!(&if_false[0], TemplateNode::Variable { name, .. } if name == "primary_light"));
            }
            other => panic!("expected conditional, got {other:?}"),
        }
    }

    #[test]
    fn conditional_with_spaces() {
        let nodes = parse("{{ mode == dark ? primary : primary_light }}");
        assert!(matches!(&nodes[0], TemplateNode::Conditional { .. }));
    }

    #[test]
    fn conditional_without_spaces_errors() {
        // `:primary_light` is greedily parsed as a filter on `primary`, so the
        // separator is unreachable — the branch separator needs whitespace.
        assert!(TemplateParser::parse("{{mode==dark?primary:primary_light}}").is_err());
    }

    #[test]
    fn conditional_no_space_around_question_mark() {
        let nodes = parse("{{mode==dark? primary : primary_light}}");
        assert!(matches!(&nodes[0], TemplateNode::Conditional { .. }));
    }

    #[test]
    fn conditional_string_branches() {
        let nodes = parse(r##"{{ flag ? "yes" : "no" }}"##);
        match &nodes[0] {
            TemplateNode::Conditional { if_true, if_false, .. } => {
                assert_eq!(if_true, &vec![TemplateNode::Text("yes".into())]);
                assert_eq!(if_false, &vec![TemplateNode::Text("no".into())]);
            }
            other => panic!("expected conditional, got {other:?}"),
        }
    }

    #[test]
    fn plain_variable_still_parses() {
        let nodes = parse("{{ primary }}");
        assert!(matches!(nodes[0], TemplateNode::Variable { .. }));
    }

    #[test]
    fn missing_question_mark_errors() {
        assert!(TemplateParser::parse("{{ primary == dark }}").is_err());
    }

    #[test]
    fn text_with_multibyte_chars_stops_at_variable() {
        // em-dash (3 bytes) before `{{` must not shift the text/variable split
        let nodes = parse("bg — {{ primary }}");
        assert_eq!(nodes[0], TemplateNode::Text("bg — ".into()));
        assert!(matches!(nodes[1], TemplateNode::Variable { ref name, .. } if name == "primary"));
    }
}
