mod data;
mod errors;
pub mod parse;
pub mod reduce;

pub use data::{Body, Decls, Template, Value, Var};
use data::{Decl, TemplatePart};
pub use errors::VerifyError;

#[cfg(test)]
mod tests_macros;
