use crate::{data::TemplatePart::*, data::*, errors::VerifyError};
use std::collections::HashMap;

pub trait Inputs {
    fn try_into_values<'decls>(&self, decls: &'decls Decls) -> Result<Values<'decls>, VerifyError>;
}
impl Inputs for HashMap<String, String> {
    fn try_into_values<'decls>(&self, decls: &'decls Decls) -> Result<Values<'decls>, VerifyError> {
        let map = decls.0.iter().filter_map(|decl| match &decl.var {
            Var::Ignore => None,
            Var::Ident(s) => Some(
                self.get(s)
                    .map(DeclValue::from)
                    .or(decl.default.clone())
                    .map(|value| (&decl.var, value))
                    .ok_or(VerifyError::MissingDecl),
            ),
        });
        Ok(Values(map.collect::<Result<_, _>>()?))
    }
}

/// combination of `Inputs` and `Decls`
#[derive(PartialEq, Debug)]
pub struct Values<'decls>(HashMap<&'decls Var, DeclValue>);

#[derive(PartialEq, Debug)]
pub struct VerifiedTemplate<'body> {
    values: Values<'body>,
    template: &'body Template,
}

impl Decls {
    fn has_defined<V: AsRef<Var>>(&self, var: V) -> bool {
        match var.as_ref() {
            Var::Ident(var) => self.0.iter().any(|d| match &d.var {
                Var::Ident(v) => v == var,
                Var::Ignore => false,
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

impl<'body, 'inputs: 'body> VerifiedTemplate<'body> {
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
                Insert(TemplateValue::Var(v)) => match self.values.0.get(v).unwrap() {
                    DeclValue::Str(s) => s.chars().collect(),
                },
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
            template: template![Insert(TemplateValue::Var(crate::data::Var::Ignore))],
        }
        .verify()
        .unwrap();
    }

    #[test]
    fn verifies_defined_value() {
        Body::Function {
            decls: Decls(vec![Decl {
                var: ident!("foo"),
                default: None,
            }]),
            template: template![Insert(TemplateValue::Var(ident!("foo")))],
        }
        .verify()
        .unwrap();
    }

    #[test]
    fn fails_undefined_value() {
        assert_eq!(
            Body::Function {
                decls: Decls(vec![]),
                template: template![Insert(TemplateValue::Var(ident!("foo")))]
            }
            .verify()
            .unwrap_err(),
            VerifyError::Errors(vec![VerifyError::Undefined(TemplateValue::Var(ident!(
                "foo"
            )))])
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
                values: Values(HashMap::from([(&ident!("var"), "foo".into())])),
                template: &template![
                    Char('a'),
                    Char('b'),
                    Insert(TemplateValue::Var(ident!("var"))),
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
            Insert(TemplateValue::Var(ident!("foo"))),
            Insert(TemplateValue::Var(ident!("bar")))
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
                    (&ident!("foo"), "ot".into()),
                    (&ident!("bar"), (" and hand".into()))
                ])),
                template: &template,
            }
        )
    }
    #[test]
    fn fails_bad_values() {
        let template = template![
            Insert(TemplateValue::Var(ident!("foo"))),
            Insert(TemplateValue::Var(ident!("bar")))
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
