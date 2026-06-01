use thiserror::Error;

use crate::{
  ast::{Expression, ParseError},
  eval::EvaluationError,
  tokenize::{TokenizeError, Tokens},
};

pub mod ast;
pub mod constants;
pub mod eval;
pub mod functions;
pub mod operators;
pub mod tokenize;

#[derive(Debug, PartialEq, PartialOrd, Error)]
pub enum Error {
  // see the display impl
  #[error(transparent)]
  TokenizeError(#[from] TokenizeError),
  #[error("error during expression parsing: {0}")]
  ASTParseError(#[from] ParseError),
  #[error("error during evaluation: {0}")]
  EvaluationError(#[from] EvaluationError),
}

/// All in one function that will do tokenization, parsing, and evaluation
/// from the input string.
pub fn parse(input_str: &str, deg_mode: bool) -> Result<f64, crate::Error> {
  let tokens = match Tokens::new(input_str) {
    Ok(it) => it,
    Err(err) => return Err(Error::TokenizeError(err)),
  };

  let ast = match Expression::new(&tokens) {
    Ok(it) => it,
    Err(err) => return Err(Error::ASTParseError(err)),
  };

  let result = match eval::evaluate(&ast, deg_mode) {
    Ok(it) => it,
    Err(err) => return Err(Error::EvaluationError(err)),
  };

  Ok(result)
}
