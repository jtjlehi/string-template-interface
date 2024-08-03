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
    Insert(TemplateValue),
}

#[derive(Debug, PartialEq)]
pub struct Decls(pub(crate) Vec<Decl>);

/// a declaration of a variable that can be used in the text of a function
#[derive(Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub(crate) struct Decl {
    pub(crate) var: Var,
    pub(crate) default: Option<DeclValue>,
}
#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum DeclValue {
    Str(String),
}
impl<S: Into<String>> From<S> for DeclValue {
    fn from(value: S) -> Self {
        DeclValue::Str(value.into())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Var {
    Ident(String),
    Ignore,
}
impl AsRef<Var> for &TemplateValue {
    fn as_ref(&self) -> &Var {
        match self {
            TemplateValue::Var(v) => v,
        }
    }
}

#[derive(Debug, PartialEq, Clone, Eq, Hash)]
#[non_exhaustive]
pub enum TemplateValue {
    Var(Var),
}
