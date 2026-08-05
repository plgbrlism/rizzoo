use std::collections::HashMap;

use crate::template::filters::FilterRegistry;
use crate::template::parser::{FilterArg, FilterCall, TemplateNode};

#[derive(Debug, Clone, Default)]
pub struct TemplateContext {
    pub vars: HashMap<String, String>,
    pub arrays: HashMap<String, Vec<String>>,
}

impl TemplateContext {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_var(mut self, name: &str, value: &str) -> Self {
        self.vars.insert(name.to_string(), value.to_string());
        self
    }

    pub fn with_array(mut self, name: &str, values: Vec<String>) -> Self {
        self.arrays.insert(name.to_string(), values);
        self
    }
}

pub struct TemplateEvaluator;

impl TemplateEvaluator {
    pub fn evaluate(nodes: &[TemplateNode], context: &TemplateContext) -> Result<String, String> {
        let mut output = String::new();
        for node in nodes {
            output.push_str(&Self::evaluate_node(node, context)?);
        }
        Ok(output)
    }

    fn evaluate_node(node: &TemplateNode, context: &TemplateContext) -> Result<String, String> {
        match node {
            TemplateNode::Text(text) => Ok(text.clone()),
            TemplateNode::Variable { name, filters } => {
                Self::evaluate_variable(name, filters, context)
            }
            TemplateNode::ForBlock {
                var_name,
                collection,
                body,
            } => Self::evaluate_for_block(var_name, collection, body, context),
        }
    }

    // Resolves a variable through an optional chain of filters.
    // Each filter's arguments are themselves resolved (bare args like `%var`
    // reference other template variables), creating a dependency graph of
    // filter invocations.
    fn evaluate_variable(
        name: &str,
        filters: &[FilterCall],
        context: &TemplateContext,
    ) -> Result<String, String> {
        let value = context
            .vars
            .get(name)
            .ok_or_else(|| format!("undefined variable: {name}"))?;

        if filters.is_empty() {
            return Ok(value.clone());
        }

        let mut current = value.clone();
        for filter in filters {
            let args: Vec<String> = filter
                .args
                .iter()
                .map(|a| Self::resolve_arg(a, context))
                .collect();
            current = FilterRegistry::apply(&filter.name, &current, &args)?;
        }
        Ok(current)
    }

    fn evaluate_for_block(
        var_name: &str,
        collection: &str,
        body: &[TemplateNode],
        context: &TemplateContext,
    ) -> Result<String, String> {
        let items = context
            .arrays
            .get(collection)
            .ok_or_else(|| format!("undefined array: {collection}"))?;

        let mut output = String::new();
        let mut child_ctx = context.clone();

        for item in items {
            child_ctx.vars.insert(var_name.to_string(), item.clone());
            output.push_str(&Self::evaluate(body, &child_ctx)?);
        }

        Ok(output)
    }

    fn resolve_arg(arg: &FilterArg, context: &TemplateContext) -> String {
        match arg {
            FilterArg::Number(n) => n.to_string(),
            FilterArg::String(s) => s.clone(),
            FilterArg::Variable(name) => context.vars.get(name).cloned().unwrap_or_default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn eval(template: &str, ctx: &TemplateContext) -> Result<String, String> {
        let nodes = crate::template::parser::TemplateParser::parse(template)?;
        TemplateEvaluator::evaluate(&nodes, ctx)
    }

    fn sample_context() -> TemplateContext {
        TemplateContext::new()
            .with_var("primary", "#ff0000")
            .with_var("secondary", "#00ff00")
            .with_var("accent", "#0000ff")
            .with_array("colors", vec!["#111111".into(), "#222222".into(), "#333333".into()])
    }

    #[test]
    fn plain_text_passthrough() {
        let ctx = TemplateContext::new();
        assert_eq!(eval("hello", &ctx).unwrap(), "hello");
    }

    #[test]
    fn variable_substitution() {
        let ctx = sample_context();
        assert_eq!(eval("{{ primary }}", &ctx).unwrap(), "#ff0000");
    }

    #[test]
    fn variable_with_text() {
        let ctx = sample_context();
        assert_eq!(eval("bg = {{ primary }}!", &ctx).unwrap(), "bg = #ff0000!");
    }

    #[test]
    fn undefined_variable_errors() {
        let ctx = sample_context();
        let err = eval("{{ nope }}", &ctx).unwrap_err();
        assert!(err.contains("undefined variable: nope"));
    }

    #[test]
    fn undefined_array_errors() {
        let ctx = sample_context();
        let err = eval("{{#for x in nope }}{{ x }}{{/x }}", &ctx).unwrap_err();
        assert!(err.contains("undefined array: nope"));
    }

    #[test]
    fn for_block_iteration() {
        let ctx = sample_context();
        let out = eval("{{#for c in colors }}{{ c }},{{/c }}", &ctx).unwrap();
        assert_eq!(out, "#111111,#222222,#333333,");
    }

    #[test]
    fn for_block_empty_array() {
        let ctx = TemplateContext::new().with_array("colors", vec![]);
        assert_eq!(eval("{{#for c in colors }}x{{/c }}", &ctx).unwrap(), "");
    }

    #[test]
    #[ignore = "nested for blocks unsupported in parser (see parser.rs tests)"]
    fn nested_for_blocks() {
        let ctx = TemplateContext::new()
            .with_array("outer", vec!["A".into(), "B".into()])
            .with_array("inner", vec!["1".into(), "2".into()]);
        let out = eval(
            "{{#for o in outer }}{{#for i in inner }}{{ o }}{{ i }}{{/i }}{{/o }}",
            &ctx,
        )
        .unwrap();
        assert_eq!(out, "A1A2B1B2");
    }

    #[test]
    fn filter_chain_renders() {
        let ctx = sample_context();
        assert_eq!(eval("{{ primary:hex_raw }}", &ctx).unwrap(), "ff0000");
    }

    #[test]
    fn bare_arg_resolves_from_context() {
        let ctx = sample_context();
        // blend primary toward secondary at 50% (CAM16-UCS, not sRGB lerp)
        let out = eval("{{ primary:blend(%secondary, 0.5) }}", &ctx).unwrap();
        assert!(crate::color::types::Rgb::from_hex(&out).is_some());
        assert_ne!(out, "#ff0000");
        assert_ne!(out, "#00ff00");
    }

    #[test]
    fn unknown_filter_errors() {
        let ctx = sample_context();
        let err = eval("{{ primary:nope }}", &ctx).unwrap_err();
        assert!(err.contains("unknown filter: nope"));
    }

    #[test]
    fn multiple_variables_in_one_template() {
        let ctx = sample_context();
        let out = eval("{{ primary }} / {{ secondary }}", &ctx).unwrap();
        assert_eq!(out, "#ff0000 / #00ff00");
    }
}
