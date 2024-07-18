use chumsky::prelude::*;

/// The file
#[derive(Debug, PartialEq)]
#[non_exhaustive]
pub enum Body {
    /// the template and the declarations of the variables that can be used in the template
    Function {
        decls: Vec<Decl>,
        template: Template,
    },
}
/// the main text
/// (what is actually used to generate the new strings)
#[derive(Debug, PartialEq, Clone)]
struct Template(Vec<TemplatePart>);
#[derive(Debug, PartialEq, Clone)]
enum TemplatePart {
    Char(char),
    /// inserted text
    Insert(Value),
}
// the `TemplatePart` name is really there because there aren't inline enums
use TemplatePart::*;
/// a declaration of a variable that can be used in the text of a function
#[derive(Debug, PartialEq)]
#[non_exhaustive]
enum Decl {
    Var(Var),
}
#[derive(Debug, Clone, PartialEq)]
enum Var {
    Ident(String),
    Ignore,
}
use Var::*;
#[derive(Debug, PartialEq, Clone)]
#[non_exhaustive]
enum Value {
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
        .map(|(decls, template)| Body::Function { decls, template })
        .then_ignore(end())
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
                decls: vec![$(Decl::Var(ident!($decl))),*],
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
            decls: vec![],
            template: template![Char('f')]
        }
    );
    test_pass!(
        new_line_body_passes,
        "{}->\nf",
        Body::Function {
            decls: vec![],
            template: template![Char('f')]
        }
    );
    test_pass!(
        insert_var_only_fn_body_passes,
        "{}->%{foo}",
        Body::Function {
            decls: vec![],
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
            decls: vec![],
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
            decls: vec![],
            template: template![Char('f'), Char('%'), Char('f')]
        }
    );
}
