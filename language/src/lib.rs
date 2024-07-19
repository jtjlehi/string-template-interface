use chumsky::prelude::*;

mod reduce;

/// The file
#[derive(Debug, PartialEq)]
#[non_exhaustive]
pub enum Body {
    /// the template and the declarations of the variables that can be used in the template
    Function { decls: Decls, template: Template },
}
/// the main text
/// (what is actually used to generate the new strings)
#[derive(Debug, PartialEq, Clone)]
pub struct Template(Vec<TemplatePart>);
#[derive(Debug, PartialEq, Clone)]
enum TemplatePart {
    Char(char),
    /// inserted text
    Insert(Value),
}
use thiserror::Error;
// the `TemplatePart` name is really there because there aren't inline enums
use TemplatePart::*;

#[derive(Debug, PartialEq)]
pub struct Decls(Vec<Decl>);
impl Decls {
    fn has_defined<V: AsRef<Var>>(&self, var: V) -> bool {
        match var.as_ref() {
            // TODO: profile this to see if this would be better with map of some sort
            Ident(var) => self.0.iter().any(|d| d.is_var(var)),
            Ignore => true,
        }
    }
    fn get_defined<V: AsRef<Var>>(&self, var: V) -> Option<&Decl> {
        // self.get_defined(var).is_some()
        match var.as_ref() {
            // TODO: profile this to see if this would be better with map of some sort
            Ident(var) => self.0.iter().find(|d| d.is_var(var)),
            Ignore => None,
        }
    }
}

/// a declaration of a variable that can be used in the text of a function
#[derive(Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
enum Decl {
    Var(Var),
}
impl Decl {
    fn is_var(&self, var: &str) -> bool {
        match self {
            Decl::Var(Ident(v)) => v == var,
            Decl::Var(Ignore) => false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Var {
    Ident(String),
    Ignore,
}
use Var::*;
impl AsRef<Var> for &Value {
    fn as_ref(&self) -> &Var {
        match self {
            Value::Var(v) => v,
        }
    }
}
impl AsRef<Var> for &String {
    fn as_ref(&self) -> &Var {
        todo!()
    }
}

#[derive(Debug, PartialEq, Clone)]
#[non_exhaustive]
pub enum Value {
    Var(Var),
}

pub fn parser() -> impl text::TextParser<char, Body, Error = Simple<char>> {
    let var = just('_').to(Var::Ignore).or(text::ident().map(Var::Ident));

    let decl = var.clone().map(Decl::Var).padded();
    let decls = decl
        .clone()
        .then_ignore(just(','))
        .repeated()
        .chain(decl.or_not())
        .delimited_by(just('{').padded(), just('}').padded())
        .padded();

    let value = var.map(Value::Var);
    let insert = value.delimited_by(just("%{"), just("}")).map(Insert);

    let escaped = just('%').then_ignore(just('%'));
    let text = choice((escaped, any())).map(Char);

    let template = insert
        .or(text)
        .repeated()
        .at_least(1)
        .map(Template)
        .then_ignore(end());

    decls
        .then_ignore(just("->").then(text::newline().or_not()))
        .then(template)
        .map(|(decls, template)| Body::Function {
            decls: Decls(decls),
            template,
        })
        .then_ignore(end())
}

#[derive(Error, Debug, PartialEq)]
pub enum VerifyError<'v> {
    #[error("variable {0:?} is undefined")]
    Undefined(&'v Value),
}
impl Body {
    pub fn verify(&self) -> Result<(), Vec<VerifyError<'_>>> {
        let v: Vec<_> = match self {
            Body::Function { decls, template } => template.0.iter().filter_map(|part| match part {
                Insert(v) if !decls.has_defined(v) => Some(VerifyError::Undefined(v)),
                _ => None,
            }),
        }
        .collect();
        if v.is_empty() {
            Ok(())
        } else {
            Err(v)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! template {
        ($($args:expr),*) => {
            Template(vec![$($args),*])
        }
    }
    macro_rules! ident {
        ($str:expr) => {
            Ident($str.to_string())
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
    macro_rules! test_pass {
        ($name:ident, $str:expr, $expected:expr) => {
            #[test]
            fn $name() {
                let actual = parser().parse($str).unwrap();
                let expected = $expected;
                assert_eq!(actual, expected);
            }
        };
    }

    #[test]
    fn empty_str_fails() {
        parser()
            .parse("")
            .expect_err("the empty string didn't fail");
    }
    #[test]
    fn empty_template_fails() {
        parser().parse("{}->").expect_err("the empty body fails");
    }
    test_pass!(
        text_only_fn_body_passes,
        "{}->f",
        Body::Function {
            decls: Decls(vec![]),
            template: template![Char('f')]
        }
    );
    test_pass!(
        new_line_body_passes,
        "{}->\nf",
        Body::Function {
            decls: Decls(vec![]),
            template: template![Char('f')]
        }
    );
    test_pass!(
        insert_var_only_fn_body_passes,
        "{}->%{foo}",
        Body::Function {
            decls: Decls(vec![]),
            template: template![Insert(Value::Var(ident!("foo")))],
        }
    );
    test_pass!(single_decl_passes, "{foo}->\nf", decls!["foo"]);
    test_pass!(single_decl_passes1, "{foo,}->\nf", decls!["foo"]);
    test_pass!(
        multi_decl_passes,
        "{foo,bar,baz}->\nf",
        decls!["foo", "bar", "baz"]
    );
    test_pass!(
        multi_decl_passes1,
        "{foo,bar,baz,}->\nf",
        decls!["foo", "bar", "baz"]
    );
    test_pass!(white_space_pass, "{} \n  \t->\nf", decls![]);
    test_pass!(white_space_pass1, "{  }->f", decls![]);
    test_pass!(white_space_pass2, "\n\n\t  \n  {}->f", decls![]);
    test_pass!(
        white_space_pass3,
        "\n\n\t  \n  {  foo, \n\nbar, }  ->f",
        decls!["foo", "bar"]
    );
    test_pass!(multiple_template_part_passes, "{}->foo%{foo}b%{foo}bar", {
        let insert = Insert(Value::Var(ident!("foo")));
        Body::Function {
            decls: Decls(vec![]),
            template: template![
                Char('f'),
                Char('o'),
                Char('o'),
                insert.clone(),
                Char('b'),
                insert,
                Char('b'),
                Char('a'),
                Char('r')
            ],
        }
    });
    test_pass!(
        escapes_double_percent,
        "{}->f%%f",
        Body::Function {
            decls: Decls(vec![]),
            template: template![Char('f'), Char('%'), Char('f')]
        }
    );

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
            template: template![Insert(Value::Var(Ignore))],
        }
        .verify()
        .unwrap()
    }

    #[test]
    fn verifies_defined_value() {
        Body::Function {
            decls: Decls(vec![Decl::Var(ident!("foo"))]),
            template: template![Insert(Value::Var(ident!("foo")))],
        }
        .verify()
        .unwrap()
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
            vec![VerifyError::Undefined(&Value::Var(ident!("foo")))]
        )
    }
}
