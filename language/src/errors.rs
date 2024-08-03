use chumsky::prelude::Simple;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum VerifyError {
    #[error("")]
    ParseError(Simple<char>),
    #[error("variable {0:?} is undefined")]
    Undefined(crate::data::TemplateValue),
    #[error("")]
    MissingDecl,
    #[error("")]
    Errors(Vec<VerifyError>),
}

impl From<Simple<char>> for VerifyError {
    fn from(value: Simple<char>) -> Self {
        Self::ParseError(value)
    }
}

impl FromIterator<VerifyError> for VerifyError {
    fn from_iter<T: IntoIterator<Item = VerifyError>>(iter: T) -> Self {
        VerifyError::Errors(iter.into_iter().collect())
    }
}
