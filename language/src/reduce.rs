use crate::{Body, Decl, Decls, Template, TemplatePart::*, Value, Var};
use std::collections::HashMap;

/// key value pairs input to generate a concrete instance of the template
pub struct Inputs(HashMap<String, String>);

pub enum InputError {
    MissingDecl,
}

impl Inputs {
    fn try_into_values<'decls, 'inputs>(
        &'inputs self,
        decls: &'decls Decls,
    ) -> Result<Values<'decls, 'inputs>, InputError> {
        let map: HashMap<_, _> = decls
            .0
            .iter()
            .filter_map(|decl| match decl {
                Decl::Var(v @ Var::Ident(s)) => Some(
                    self.0
                        .get(s)
                        .map(|value| (v, value.as_str()))
                        .ok_or(InputError::MissingDecl),
                ),
                Decl::Var(Var::Ignore) => None,
            })
            .collect::<Result<_, _>>()?;
        Ok(Values(map))
    }
}

/// combination of `Inputs` and `Decls`
struct Values<'decls, 'inputs>(HashMap<&'decls Var, &'inputs str>);

// TODO: create new-type for verified template and Values
// - use the builder pattern to construct it?

impl Body {
    /// get usable values for the template
    fn get_values<'d, 'i>(&'d self, inputs: &'i Inputs) -> Result<Values<'d, 'i>, InputError> {
        match self {
            Body::Function { decls, .. } => inputs.try_into_values(decls),
        }
    }
    fn reduce(&self, values: Values) -> String {
        match self {
            Body::Function { template, .. } => template.reduce(values),
        }
    }
    pub fn reduce_with(self, inputs: Inputs) -> Result<String, InputError> {
        let values = self.get_values(&inputs)?;
        Ok(self.reduce(values))
    }
}
impl Template {
    fn reduce(&self, values: Values) -> String {
        self.0
            .iter()
            .flat_map(|x| match x {
                Char(c) => vec![*c],
                // unwrap is safe if `verify` and `get_values` pass
                // TODO: use new-type to prove this more securely
                Insert(Value::Var(v)) => values.0.get(v).unwrap().chars().collect::<Vec<_>>(),
            })
            .collect()
    }
}
