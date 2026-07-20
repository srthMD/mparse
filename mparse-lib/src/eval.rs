use std::{
  fmt::{self},
  ops::{Add, BitAnd, BitOr, Div, Mul, Neg, Not, Rem, Sub},
  vec,
};

use thiserror::Error;

use crate::{
  ast::Expression,
  functions::{FunctionEvaluationError, FunctionType},
  operators::Operation,
  types::{
    object::{Object, ObjectKind},
    vector::{Vec2D, Vec3D},
  },
};

#[derive(Debug, PartialEq, PartialOrd, Error)]
pub enum EvaluationErrorRepr {
  #[error("unexpected operator {0} found during evaluation")]
  UnexpectedOperator(Operation),
  #[error(fmt = fmt_function_evaluation_err)]
  FunctionEvaluationError(FunctionType, FunctionEvaluationError),
  #[error("{0} cannot be used as a factorial argument: {1}")]
  InvalidFactorialArg(f64, InvalidFactorialReason),
  #[error(fmt = fmt_type_error)]
  TypeError {
    obj1: Object,
    obj2: Object,
    reason: TypeErrorReason,
  },
  #[error("no such field \"{field}\" in {kind}")]
  InvalidField { kind: ObjectKind, field: String },
  #[error("attempt to index type {0} that has no fields")]
  NoFields(ObjectKind),
}

fn fmt_function_evaluation_err(
  ftype: &FunctionType,
  err: &FunctionEvaluationError,
  f: &mut fmt::Formatter,
) -> fmt::Result {
  write!(
    f,
    "error evaluating function {}: {}",
    ftype.as_str().to_lowercase(),
    err
  )
}

#[derive(Debug, PartialEq, PartialOrd)]
pub enum TypeErrorReason {
  IncompatibleTypeOp(Operation),
  InvalidCast(ObjectKind),
}

fn fmt_type_error(
  obj1: &Object,
  obj2: &Object,
  reason: &TypeErrorReason,
  f: &mut fmt::Formatter,
) -> fmt::Result {
  match reason {
    TypeErrorReason::IncompatibleTypeOp(operation) => write!(
      f,
      "cannot preform operation \'{}\' between {} and {}",
      operation.as_str(),
      obj1.stringify_type(),
      obj2.stringify_type(),
    ),
    TypeErrorReason::InvalidCast(type_name) => {
      write!(f, "cannot cast {} to {}", obj1.stringify_type(), type_name)
    }
  }
}

#[derive(Debug, PartialEq, PartialOrd, Error)]
pub enum InvalidFactorialReason {
  #[error("input must be an integer")]
  RationalNumber,
  #[error("input must be positive and non zero")]
  NegativeOrZero,
}

pub fn evaluate(expr: &Expression, deg_mode: bool) -> Result<Object, EvaluationErrorRepr> {
  let mut final_result = Object::Null;

  match expr {
    Expression::Number(n) => final_result.checked_add_assign((*n).into())?,
    Expression::Constant(constant) => {
      final_result.checked_add_assign(constant.get_value().into())?
    }
    Expression::Boolean(b) => final_result = Object::Bool(*b),
    Expression::Null => final_result = Object::Null,
    // unreachable?
    Expression::Field(_) => todo!(),

    Expression::Unary { op, expr } => {
      let res = match op {
        Operation::Sub => evaluate(expr, deg_mode)?.neg(),
        Operation::Fac => {
          let obj = evaluate(expr, deg_mode)?;
          match obj {
            Object::Bool(b) => Object::Bool(!b),
            _ => factorial(obj)?,
          }
        }
        _ => return Err(EvaluationErrorRepr::UnexpectedOperator(*op)),
      };

      final_result.checked_add_assign(res)?;
    }

    Expression::Binary { op, left, right } => {
      let res = match op {
        //im questioning everything get it so funyn hahah ha h🤣🤣🤣🤣🤣🤣🤣🤣⚠️
        Operation::Add => evaluate(left, deg_mode)?.add(evaluate(right, deg_mode)?)?,
        Operation::Sub => evaluate(left, deg_mode)?.sub(evaluate(right, deg_mode)?)?,
        Operation::Mul => evaluate(left, deg_mode)?.mul(evaluate(right, deg_mode)?)?,
        Operation::Div => evaluate(left, deg_mode)?.div(evaluate(right, deg_mode)?)?,
        Operation::Exp => evaluate(left, deg_mode)?.powf(evaluate(right, deg_mode)?)?,
        Operation::Rem => evaluate(left, deg_mode)?.rem(evaluate(right, deg_mode)?)?,
        Operation::And => evaluate(left, deg_mode)?.bitand(evaluate(right, deg_mode)?)?,
        Operation::Or => evaluate(left, deg_mode)?.bitor(evaluate(right, deg_mode)?)?,
        Operation::Eq => evaluate(left, deg_mode)?
          .eq(&evaluate(right, deg_mode)?)
          .into(),
        Operation::Neq => (evaluate(left, deg_mode)?.eq(&evaluate(right, deg_mode)?))
          .not()
          .into(),
        Operation::Dot => {
          let field = match **right {
            Expression::Field(ref s) => s.clone(),
            _ => todo!(),
          };

          let obj_left = evaluate(left, deg_mode)?;
          let val = obj_left.access_field(field)?;
          val
        }
        _ => return Err(EvaluationErrorRepr::UnexpectedOperator(*op)),
      };

      final_result.checked_add_assign(res)?;
    }

    Expression::Function { func, exprs } => {
      let mut results = vec![];

      for exp in exprs {
        results.push(evaluate(exp, deg_mode)?);
      }

      let res = func.eval(results, deg_mode);
      if let Ok(mut yipee) = res {
        if func.get_function_type().outputs_angle() && deg_mode {
          // if the function outputs an angle it has to be a number, right... RIGHT?????
          let num: f64 = yipee.try_into().expect("unreachable");
          yipee = num.to_degrees().into();
        }

        final_result.checked_add_assign(yipee)?
      } else {
        return res;
      }
    }
  }

  Ok(final_result)
}

fn factorial(obj: Object) -> Result<Object, EvaluationErrorRepr> {
  match obj {
    Object::Number(num) => Ok(factorial_f64(num)?.into()),
    Object::Vec2D(ref v) => Ok(factorial_vec2d(v)?.into()),
    Object::Vec3D(ref v) => Ok(factorial_vec3d(v)?.into()),
    _ => panic!("argument mismatch"),
  }
}

fn factorial_f64(num: f64) -> Result<f64, EvaluationErrorRepr> {
  if num < 0f64 {
    return Err(EvaluationErrorRepr::InvalidFactorialArg(
      num,
      InvalidFactorialReason::NegativeOrZero,
    ));
  }

  if num.fract() != 0f64 {
    return Err(EvaluationErrorRepr::InvalidFactorialArg(
      num,
      InvalidFactorialReason::RationalNumber,
    ));
  }

  let mut res = 1f64;

  for i in 1..=(num as u64) {
    res *= i as f64;
  }

  return Ok(res);
}

fn factorial_vec2d(vec: &Vec2D) -> Result<Vec2D, EvaluationErrorRepr> {
  Ok(Vec2D {
    x: factorial_f64(vec.x)?,
    y: factorial_f64(vec.y)?,
  })
}

fn factorial_vec3d(vec: &Vec3D) -> Result<Vec3D, EvaluationErrorRepr> {
  Ok(Vec3D {
    x: factorial_f64(vec.x)?,
    y: factorial_f64(vec.y)?,
    z: factorial_f64(vec.z)?,
  })
}
