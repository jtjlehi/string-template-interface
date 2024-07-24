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
