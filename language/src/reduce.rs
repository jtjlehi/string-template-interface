use crate::{errors::VerifyError, Body, Decl, Decls, Template, TemplatePart::*, Value, Var};
use std::collections::HashMap;

pub trait Inputs {
    fn try_into_values<'decls, 'inputs>(
        &'inputs self,
        decls: &'decls Decls,
    ) -> Result<Values<'decls, 'inputs>, VerifyError>;
}
impl Inputs for HashMap<String, String> {
    fn try_into_values<'decls, 'inputs>(
        &'inputs self,
        decls: &'decls Decls,
    ) -> Result<Values<'decls, 'inputs>, VerifyError> {
        let map: HashMap<_, _> = decls
            .0
            .iter()
            .filter_map(|decl| match decl {
                Decl::Var(v @ Var::Ident(s)) => Some(
                    self.get(s)
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
#[derive(PartialEq, Debug)]
pub struct Values<'decls, 'inputs>(HashMap<&'decls Var, &'inputs str>);

#[derive(PartialEq, Debug)]
pub struct VerifiedTemplate<'body, 'inputs> {
    values: Values<'body, 'inputs>,
    template: &'body Template,
}

impl<'body, 'inputs: 'body> VerifiedTemplate<'body, 'inputs> {
    pub fn try_from_body_inputs<I: Inputs>(
        body: &'body Body,
        inputs: &'inputs I,
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
            body_function!(decls: [$($decl),*]; template: [Char('f')])
            // Body::Function {
            //     decls: Decls(vec![$(Decl::Var(ident!($decl))),*]),
            //     template: template![Char('f')],
            // }
        };
    }
    macro_rules! body_function {
        (decls: [$($decl:expr),*]; template: $template:expr) => {
            Body::Function {
                decls: Decls(vec![$(Decl::Var(ident!($decl))),*]),
                template: $template,
            }
        }
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

    #[test]
    fn creates_verified_template() {
        let template = template![
            Insert(Value::Var(ident!("foo"))),
            Insert(Value::Var(ident!("bar")))
        ];
        assert_eq!(
            VerifiedTemplate::try_from_body_inputs(
                &body_function!(
                    decls: ["foo", "bar"];
                    template: template.clone()
                ),
                &HashMap::from([
                    ("foo".to_string(), "ot".to_string()),
                    ("bar".to_string(), " and hand".to_string())
                ])
            )
            .unwrap(),
            VerifiedTemplate {
                values: Values(HashMap::from([
                    (&ident!("foo"), "ot"),
                    (&ident!("bar"), " and hand")
                ])),
                template: &template,
            }
        )
    }
    #[test]
    fn fails_bad_values() {
        let template = template![
            Insert(Value::Var(ident!("foo"))),
            Insert(Value::Var(ident!("bar")))
        ];
        assert_eq!(
            VerifiedTemplate::try_from_body_inputs(
                &body_function!(
                    decls: ["foo", "bar"];
                    template: template.clone()
                ),
                &HashMap::from([("foo".to_string(), "ot".to_string()),])
            )
            .unwrap_err(),
            VerifyError::MissingDecl
        )
    }
}
