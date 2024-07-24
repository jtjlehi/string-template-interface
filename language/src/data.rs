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
pub struct Template(pub(crate) Vec<TemplatePart>);
#[derive(Debug, PartialEq, Clone)]
pub(crate) enum TemplatePart {
    Char(char),
    /// inserted text
    Insert(Value),
}

#[derive(Debug, PartialEq)]
pub struct Decls(pub(crate) Vec<Decl>);

/// a declaration of a variable that can be used in the text of a function
#[derive(Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub(crate) enum Decl {
    Var(Var),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Var {
    Ident(String),
    Ignore,
}
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
