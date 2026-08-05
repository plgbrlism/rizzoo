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
