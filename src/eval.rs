use std::ops::{Add, Div, Mul, Neg, Rem, Sub};

use thiserror::Error;

use crate::{ast::Expression, functions::FunctionEvaluationError, operators::Operation};

#[derive(Debug, PartialEq, PartialOrd, Error)]
pub enum EvaluationError {
  #[error("unexpected operator {0} found during evaluation")]
  UnexpectedOperator(Operation),
  #[error("{0} cannot be used as a factorial argument: {1}")]
  InvalidFactorialArg(f64, InvalidFactorialReason),
  #[error("function expects {required:?} arguments, got {given:?}")]
  InvalidArgumentCount { required: usize, given: usize },
  #[error(transparent)]
  FunctionEvaluationError(#[from] FunctionEvaluationError),
}

#[derive(Debug, PartialEq, PartialOrd, Error)]
pub enum InvalidFactorialReason {
  #[error("input must be an integer")]
  RationalNumber,
  #[error("input must be positive and non zero")]
  NegativeOrZero,
}

pub fn evaluate(expr: &Expression, deg_mode: bool) -> Result<f64, EvaluationError> {
  let mut final_result = 0f64;

  match expr {
    Expression::Number(n) => final_result += n,
    Expression::Constant(constant) => final_result += constant.get_value(),
    Expression::Unary { op, expr } => {
      let res = match op {
        Operation::Sub => evaluate(expr, deg_mode)?.neg(),
        Operation::Fac => factorial(evaluate(expr, deg_mode)?)?,
        _ => return Err(EvaluationError::UnexpectedOperator(*op)),
      };

      final_result += res;
    }
    Expression::Binary { op, left, right } => {
      let res = match op {
        Operation::Add => evaluate(left, deg_mode)?.add(evaluate(right, deg_mode)?),
        Operation::Sub => evaluate(left, deg_mode)?.sub(evaluate(right, deg_mode)?),
        Operation::Mul => evaluate(left, deg_mode)?.mul(evaluate(right, deg_mode)?),
        Operation::Div => evaluate(left, deg_mode)?.div(evaluate(right, deg_mode)?),
        Operation::Exp => evaluate(left, deg_mode)?.powf(evaluate(right, deg_mode)?),
        Operation::Mod => evaluate(left, deg_mode)?.rem(evaluate(right, deg_mode)?),
        _ => return Err(EvaluationError::UnexpectedOperator(*op)),
      };

      final_result += res;
    }
    Expression::Function { func, exprs } => {
      let res = if !func.requires_args() {
        func.eval(&[], false)
      } else {
        let mut results = Vec::<f64>::new();

        for exp in exprs {
          results.push(evaluate(exp, deg_mode)?);
        }

        let arg_count = func.arg_count();
        if arg_count != results.len() && !func.supports_varargs() {
          return Err(EvaluationError::InvalidArgumentCount {
            required: arg_count,
            given: results.len(),
          });
        }

        func.eval(&results, deg_mode)
      };

      if let Ok(yipee) = res {
        final_result += yipee
      } else {
        return Err(EvaluationError::FunctionEvaluationError(
          res.expect_err("unreachable"),
        ));
      }
    }
  }

  Ok(final_result)
}

fn factorial(num: f64) -> Result<f64, EvaluationError> {
  if num < 0f64 {
    return Err(EvaluationError::InvalidFactorialArg(
      num,
      InvalidFactorialReason::NegativeOrZero,
    ));
  }

  if num.fract() != 0f64 {
    return Err(EvaluationError::InvalidFactorialArg(
      num,
      InvalidFactorialReason::RationalNumber,
    ));
  }

  let mut res = 1f64;

  for i in 1..=(num as u64) {
    res *= i as f64;
  }

  Ok(res)
}