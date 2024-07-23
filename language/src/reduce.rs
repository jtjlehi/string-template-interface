use crate::{errors::VerifyError, Body, Decl, Decls, Template, TemplatePart::*, Value, Var};
use std::collections::HashMap;

/// key value pairs input to generate a concrete instance of the template
pub struct Inputs(HashMap<String, String>);

impl Inputs {
    fn try_into_values<'decls, 'inputs>(
        &'inputs self,
        decls: &'decls Decls,
    ) -> Result<Values<'decls, 'inputs>, VerifyError> {
        let map: HashMap<_, _> = decls
            .0
            .iter()
            .filter_map(|decl| match decl {
                Decl::Var(v @ Var::Ident(s)) => Some(
                    self.0
                        .get(s)
                        .map(|value| (v, value.as_str()))
                        .ok_or(VerifyError::MissingDecl),
                ),
                Decl::Var(Var::Ignore) => None,
            })
            .collect::<Result<_, _>>()?;
        Ok(Values(map))
    }
}

/// combination of `Inputs` and `Decls`
struct Values<'decls, 'inputs>(HashMap<&'decls Var, &'inputs str>);

struct VerifiedTemplate<'body, 'inputs> {
    values: Values<'body, 'inputs>,
    template: &'body Template,
}

impl<'body, 'inputs: 'body> VerifiedTemplate<'body, 'inputs> {
    pub fn try_from_body_inputs(
        body: &'body Body,
        inputs: &'inputs Inputs,
    ) -> Result<Self, VerifyError<'body>> {
        body.verify()?;
        let (template, decls) = match body {
            Body::Function { template, decls } => (template, decls),
        };
        let values = inputs.try_into_values(decls)?;
        Ok(Self { values, template })
    }

    pub fn reduce(&self) -> String {
        self.template
            .0
            .iter()
            .flat_map(|x| match x {
                Char(c) => vec![*c],
                // unwrap is safe since verified template proves it is
                Insert(Value::Var(v)) => self.values.0.get(v).unwrap().chars().collect::<Vec<_>>(),
            })
            .collect()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::TemplatePart::*;

    macro_rules! template {
        ($($args:expr),*) => {
            Template(vec![$($args),*])
        }
    }
    macro_rules! ident {
        ($str:expr) => {
            Var::Ident($str.to_string())
        };
    }
    macro_rules! decls {
        ($($decl:expr),*) => {
            Body::Function {
                decls: Decls(vec![$(Decl::Var(ident!($decl))),*]),
                template: template![Char('f')],
            }
        };
    }

    #[test]
    fn verified_template_reduces() {
        assert_eq!(
            VerifiedTemplate {
                values: Values(HashMap::from([])),
                template: &template![],
            }
            .reduce(),
            ""
        );
    }
    #[test]
    fn verified_template_reduces1() {
        assert_eq!(
            VerifiedTemplate {
                values: Values(HashMap::from([])),
                template: &template![Char('a'), Char('b'), Char('c')],
            }
            .reduce(),
            "abc"
        );
    }
    #[test]
    fn verified_template_reduces2() {
        assert_eq!(
            VerifiedTemplate {
                values: Values(HashMap::from([(&ident!("var"), "foo")])),
                template: &template![
                    Char('a'),
                    Char('b'),
                    Insert(Value::Var(ident!("var"))),
                    Char('c')
                ],
            }
            .reduce(),
            "abfooc"
        );
    }
}
