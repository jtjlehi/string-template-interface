use thiserror::Error;

use crate::Value;

#[derive(Error, Debug, PartialEq)]
pub enum VerifyError<'v> {
    #[error("variable {0:?} is undefined")]
    Undefined(&'v Value),
    #[error("")]
    MissingDecl,
    #[error("")]
    Errors(Vec<VerifyError<'v>>),
}

impl<'v> FromIterator<VerifyError<'v>> for VerifyError<'v> {
    fn from_iter<T: IntoIterator<Item = VerifyError<'v>>>(iter: T) -> Self {
        VerifyError::Errors(iter.into_iter().collect())
    }
}
