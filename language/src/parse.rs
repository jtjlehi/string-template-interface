use chumsky::prelude::*;

use crate::data::*;
use TemplatePart::*;

fn parser() -> impl text::TextParser<char, Body, Error = Simple<char>> {
    let var = just('_').to(Var::Ignore).or(text::ident().map(Var::Ident));

    let str_value = just::<_, _, Simple<char>>('"')
        .ignore_then(none_of("\"").repeated().collect::<String>())
        .then_ignore(just('"'))
        .map(DeclValue::Str)
        .padded();
    let default_val = just('?').ignore_then(str_value).or_not().padded();
    let decl = var
        .clone()
        .then(default_val)
        .map(|(var, default)| Decl { var, default })
        .padded();
    let decls = decl
        .clone()
        .then_ignore(just(','))
        .repeated()
        .chain(decl.or_not())
        .delimited_by(just('{').padded(), just('}').padded())
        .padded();

    let value = var.map(TemplateValue::Var);
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
pub(super) fn parse(s: &str) -> Result<Body, crate::errors::VerifyError> {
    parser()
        .parse(s)
        .map_err(|errs| errs.into_iter().map(Into::into).collect())
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::tests_macros::*;

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
            template: template![Insert(TemplateValue::Var(ident!("foo")))],
        }
    );
    test_pass!(single_decl_passes, "{foo}->\nf", decls!["foo"]);
    test_pass!(single_decl_passes1, "{foo,}->\nf", decls!["foo"]);
    test_pass!(
        default_str_decl,
        "{foo ? \"\"}->f",
        Body::Function {
            decls: Decls(vec![Decl {
                var: Var::Ident("foo".to_string()),
                default: Some(DeclValue::Str(String::new()))
            }]),
            template: template![Char('f')],
        }
    );
    test_pass!(
        default_str_decl1,
        "{foo ? \"this is my string\"}->f",
        Body::Function {
            decls: Decls(vec![Decl {
                var: Var::Ident("foo".to_string()),
                default: Some(DeclValue::Str("this is my string".to_string()))
            }]),
            template: template![Char('f')],
        }
    );
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
        let insert = Insert(TemplateValue::Var(ident!("foo")));
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
}
