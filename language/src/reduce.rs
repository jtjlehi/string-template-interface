use crate::{data::TemplatePart::*, data::*, errors::VerifyError};
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

impl Decls {
    fn has_defined<V: AsRef<Var>>(&self, var: V) -> bool {
        match var.as_ref() {
            Var::Ident(var) => self.0.iter().any(|d| match &d {
                Decl::Var(Var::Ident(v)) => v == var,
                Decl::Var(_) => false,
            }),
            Var::Ignore => true,
        }
    }
}

impl Body {
    fn verify(&self) -> Result<(&Template, &Decls), VerifyError> {
        match self {
            Body::Function { template, decls } => {
                let v: Vec<_> = template
                    .0
                    .iter()
                    .filter_map(|part| match part {
                        Insert(v) if !decls.has_defined(v) => {
                            Some(VerifyError::Undefined(v.clone()))
                        }
                        _ => None,
                    })
                    .collect();
                if !v.is_empty() {
                    Err(VerifyError::Errors(v))?;
                };
                Ok((&template, &decls))
            }
        }
    }
}

impl<'body, 'inputs: 'body> VerifiedTemplate<'body, 'inputs> {
    pub fn try_from_body_inputs<I: Inputs>(
        body: &'body Body,
        inputs: &'inputs I,
    ) -> Result<Self, VerifyError> {
        let (template, decls) = body.verify()?;
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
    use crate::tests_macros::*;

    #[test]
    fn verifies_char_only_template() {
        assert!(Body::Function {
            decls: Decls(vec![]),
            template: template![Char('f')]
        }
        .verify()
        .is_ok());
    }
    #[test]
    fn verifies_ignore_template() {
        Body::Function {
            decls: Decls(vec![]),
            template: template![Insert(Value::Var(crate::data::Var::Ignore))],
        }
        .verify()
        .unwrap();
    }

    #[test]
    fn verifies_defined_value() {
        Body::Function {
            decls: Decls(vec![Decl::Var(ident!("foo"))]),
            template: template![Insert(Value::Var(ident!("foo")))],
        }
        .verify()
        .unwrap();
    }

    #[test]
    fn fails_undefined_value() {
        assert_eq!(
            Body::Function {
                decls: Decls(vec![]),
                template: template![Insert(Value::Var(ident!("foo")))]
            }
            .verify()
            .unwrap_err(),
            VerifyError::Errors(vec![VerifyError::Undefined(Value::Var(ident!("foo")))])
        )
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
