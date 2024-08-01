mod data;
mod errors;
mod parse;
use data::{Body, Decl, Decls, Template, TemplatePart, Value, Var};

pub mod reduce;
pub use errors::VerifyError;
pub use reduce::Inputs;

pub fn eval<I: Inputs>(s: &str, inputs: &I) -> Result<String, VerifyError> {
    Ok(reduce::VerifiedTemplate::try_from_body_inputs(&parse::parse(s)?, inputs)?.reduce())
}

#[cfg(test)]
mod tests_macros;
