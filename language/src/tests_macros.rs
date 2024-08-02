macro_rules! template {
    ($($args:expr),*) => {
        crate::data::Template(vec![$($args),*])
    }
}
pub(crate) use template;
macro_rules! ident {
    ($str:expr) => {
        crate::data::Var::Ident($str.to_string())
    };
}
pub(crate) use ident;
macro_rules! decls {
    ($($decl:expr),*) => {
        crate::data::Body::Function {
            decls: Decls(vec![$(Decl::Var(ident!($decl))),*]),
            template: template![Char('f')],
        }
    };
}
pub(crate) use decls;
macro_rules! body_function {
    (decls: [$($decl:expr),*]; template: $template:expr) => {
        Body::Function {
            decls: Decls(vec![$(Decl::Var(ident!($decl))),*]),
            template: $template,
        }
    }
}
pub(crate) use body_function;
