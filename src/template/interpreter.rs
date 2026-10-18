use std::collections::HashMap;

use crate::template::filters::FilterRegistry;
use crate::template::parser::{Condition, FilterArg, FilterCall, TemplateNode};

#[derive(Debug, Clone, Default)]
pub struct TemplateContext {
    pub vars: HashMap<String, String>,
}

impl TemplateContext {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_var(mut self, name: &str, value: &str) -> Self {
        self.vars.insert(name.to_string(), value.to_string());
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
            TemplateNode::Conditional {
                condition,
                if_true,
                if_false,
            } => {
                let matched = Self::evaluate_condition(condition, context);
                let branch = if matched { if_true } else { if_false };
                Self::evaluate(branch, context)
            }
        }
    }

    fn evaluate_condition(condition: &Condition, context: &TemplateContext) -> bool {
        match condition {
            Condition::Var(name) => context
                .vars
                .get(name)
                .map(|v| !v.is_empty())
                .unwrap_or(false),
            Condition::NotVar(name) => context
                .vars
                .get(name)
                .map(|v| v.is_empty())
                .unwrap_or(true),
            Condition::Eq(left, right) => context
                .vars
                .get(left)
                .map(|v| v == right)
                .unwrap_or(false),
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

    #[test]
    fn eq_condition_selects_true_branch() {
        let ctx = sample_context().with_var("mode", "dark");
        let out = eval("{{ mode == dark ? primary : secondary }}", &ctx).unwrap();
        assert_eq!(out, "#ff0000");
    }

    #[test]
    fn eq_condition_selects_false_branch() {
        let ctx = sample_context().with_var("mode", "light");
        let out = eval("{{ mode == dark ? primary : secondary }}", &ctx).unwrap();
        assert_eq!(out, "#00ff00");
    }

    #[test]
    fn eq_condition_missing_var_takes_false_branch() {
        let ctx = sample_context();
        let out = eval("{{ mode == dark ? primary : secondary }}", &ctx).unwrap();
        assert_eq!(out, "#00ff00");
    }

    #[test]
    fn var_condition_truthiness() {
        let ctx = sample_context().with_var("custom_accent", "#123456");
        let out = eval("{{ custom_accent ? custom_accent : primary }}", &ctx).unwrap();
        assert_eq!(out, "#123456");
    }

    #[test]
    fn var_condition_empty_takes_false_branch() {
        let ctx = sample_context().with_var("custom_accent", "");
        let out = eval("{{ custom_accent ? custom_accent : primary }}", &ctx).unwrap();
        assert_eq!(out, "#ff0000");
    }

    #[test]
    fn not_var_condition() {
        let ctx = sample_context();
        let out = eval("{{ !custom_accent ? primary : secondary }}", &ctx).unwrap();
        assert_eq!(out, "#ff0000");
    }

    #[test]
    fn not_var_condition_set_takes_false_branch() {
        let ctx = sample_context().with_var("custom_accent", "#123456");
        let out = eval("{{ !custom_accent ? primary : secondary }}", &ctx).unwrap();
        assert_eq!(out, "#00ff00");
    }

    #[test]
    fn string_literal_branches() {
        let ctx = sample_context().with_var("mode", "dark");
        let out = eval(r##"{{ mode == dark ? "on" : "off" }}"##, &ctx).unwrap();
        assert_eq!(out, "on");
    }

    #[test]
    fn branch_filters_still_apply() {
        let ctx = sample_context().with_var("mode", "dark");
        let out = eval("{{ mode == dark ? primary:hex_raw : secondary }}", &ctx).unwrap();
        assert_eq!(out, "ff0000");
    }
}
