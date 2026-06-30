use crate::{
  eval::{
    EvaluationErrorRepr,
    TypeErrorReason::{self, InvalidCast},
  },
  operators::Operation,
  types::{
    object::Object::Null,
    vector::{Vector, VectorKind},
  },
};
use paste::paste;
use std::{
  fmt::Display,
  ops::{Add, Div, Mul, Neg, Rem, Sub},
};

#[derive(PartialEq, PartialOrd, Debug, Clone, Copy)]
pub enum Object {
  Null,
  Number(f64),
  Vector(Vector),
}

#[derive(PartialEq, PartialOrd, Debug, Clone, Copy)]
pub enum ObjectKind {
  Null,
  Number,
  Vector(VectorKind),
  AnyOf(&'static [ObjectKind]),
}

impl Display for ObjectKind {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      ObjectKind::Null => write!(f, "null"),
      ObjectKind::Number => write!(f, "number"),
      ObjectKind::Vector(vector_kind) => write!(f, "{}", vector_kind),
      ObjectKind::AnyOf(object_kinds) => {
        for i in 0..object_kinds.len() {
          #[allow(unused_must_use)]
          if i >= object_kinds.len() - 1 {
            write!(f, "{}", object_kinds[i]);
          } else {
            write!(f, "{} | ", object_kinds[i]);
          }
        }

        Ok(())
      }
    }
  }
}

macro_rules! obj_ops_impl {
  ($($t:ident)*) => ($(
    impl $t for Object {
      type Output = Result<Self, EvaluationErrorRepr>;

      paste! {
          fn [<$t:lower>](self, other: Self) -> Self::Output {
            match (self, other) {
              (Object::Vector(v1), Object::Vector(v2)) => Ok(Object::Vector(v1.[<$t:lower>](v2)?)),
              (Object::Number(rhs), Object::Vector(lhs)) | (Object::Vector(lhs), Object::Number(rhs)) => Ok(lhs.[<$t:lower>](rhs).into()),
              (Object::Number(lhs), Object::Number(rhs)) => Ok(lhs.[<$t:lower>](rhs).into()),
              (Object::Null, _) => Ok(other.clone()),
              (_, Object::Null) => Ok(self.clone()),
            }
          }
      }
    }
  )*)
}

macro_rules! obj_assign_ops_impl {
  ($($t:ident)*) => (
    impl Object {
      $(
        paste! {
         #[allow(unused)]
         pub(crate) fn [<checked_ $t:lower _assign>](&mut self, other: Self) -> Result<(), EvaluationErrorRepr> {
           match (&self, other) {
             (Object::Vector(v1), Object::Vector(v2)) => *self = v1.[<$t:lower>](v2)?.into(),
             // fuck off
             (Object::Number(rhs), Object::Vector(lhs)) => *self = lhs.[<$t:lower>](*rhs).into(),
             (Object::Vector(lhs), Object::Number(rhs)) => *self = lhs.[<$t:lower>](rhs).into(),
             (Object::Number(lhs), Object::Number(n_rhs)) => *self = lhs.[<$t:lower>](n_rhs).into(),
             (Object::Null, _) => {*self = other},
             _ => {}
           };

           Ok(())
         }
        }
      )*
    }
  )
}

obj_ops_impl! {Add Sub Mul Div Rem}
obj_assign_ops_impl! {Add Sub Mul Div Rem}

impl Object {
  pub(crate) fn powf(&self, exponent: Object) -> Result<Self, EvaluationErrorRepr> {
    match (self, exponent) {
      (Object::Number(n), Object::Number(exponent)) => Ok(n.powf(exponent).into()),
      (Object::Vector(vector), Object::Number(exponent)) => Ok(vector.powf(exponent).into()),
      _ => Err(EvaluationErrorRepr::TypeError {
        obj1: self.clone(),
        obj2: exponent.clone(),
        reason: TypeErrorReason::IncompatibleTypeOp(Operation::Exp),
      }),
    }
  }

  pub(crate) fn stringify_type(&self) -> &'static str {
    match self {
      Object::Null => "null",
      Object::Number(_) => "number",
      #[allow(unused_variables)]
      Object::Vector(vector) => match vector {
        Vector::Vec2D { x, y } => "vec2d",
        Vector::Vec3D { x, y, z } => "vec3d",
      },
    }
  }

  pub(crate) fn kind(&self) -> ObjectKind {
    match self {
      Null => ObjectKind::Null,
      Object::Number(_) => ObjectKind::Number,
      Object::Vector(vector) => ObjectKind::Vector(vector.kind()),
    }
  }
}

impl Neg for Object {
  type Output = Self;

  fn neg(self) -> Self::Output {
    match self {
      Object::Number(n) => (-n).into(),
      Object::Vector(vector) => (-vector).into(),
      Object::Null => self,
    }
  }
}

impl Display for Object {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Object::Null => write!(f, "null"),
      Object::Number(n) => write!(f, "{}", n),
      Object::Vector(vector) => write!(f, "{}", vector),
    }
  }
}

impl TryInto<f64> for Object {
  type Error = EvaluationErrorRepr;

  fn try_into(self) -> Result<f64, Self::Error> {
    match self {
      Object::Number(n) => Ok(n),
      _ => Err(EvaluationErrorRepr::TypeError {
        obj1: self,
        obj2: Null,
        reason: InvalidCast(ObjectKind::Number),
      }),
    }
  }
}

impl TryInto<Vector> for Object {
  type Error = EvaluationErrorRepr;

  fn try_into(self) -> Result<Vector, Self::Error> {
    match self {
      Object::Vector(n) => Ok(n),
      _ => Err(EvaluationErrorRepr::TypeError {
        obj1: self,
        obj2: Null,
        reason: InvalidCast(ObjectKind::Vector(VectorKind::Any)),
      }),
    }
  }
}

impl From<f64> for Object {
  fn from(value: f64) -> Self {
    Object::Number(value)
  }
}

impl From<Vector> for Object {
  fn from(value: Vector) -> Self {
    Object::Vector(value)
  }
}
