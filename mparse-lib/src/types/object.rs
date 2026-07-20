use crate::{
  eval::{
    EvaluationErrorRepr,
    TypeErrorReason::{self, InvalidCast},
  },
  operators::Operation,
  types::{
    field::FieldAccess,
    object::Object::Null,
    vector::{Vec2D, Vec3D},
  },
};
use paste::paste;
use std::{
  fmt::Display,
  ops::{Add, BitAnd, BitOr, BitXor, Div, Mul, Neg, Rem, Sub},
};

#[derive(PartialEq, PartialOrd, Clone, Copy, Debug)]
pub enum Object {
  Null,
  Number(f64),
  Bool(bool),
  Vec2D(Vec2D),
  Vec3D(Vec3D),
}

#[derive(PartialEq, PartialOrd, Debug, Clone, Copy)]
pub enum ObjectKind {
  Null,
  Number,
  Vec2D,
  Vec3D,
  Bool,
  Any,
  AnyOf(&'static [ObjectKind]),
}

impl Display for ObjectKind {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      ObjectKind::Null => write!(f, "null"),
      ObjectKind::Number => write!(f, "number"),
      ObjectKind::Vec2D => write!(f, "vec2d"),
      ObjectKind::Vec3D => write!(f, "vec3d"),
      ObjectKind::Any => write!(f, "any"),
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
      ObjectKind::Bool => write!(f, "bool"),
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
              (Object::Vec2D(v1), Object::Vec2D(v2)) => Ok(Object::Vec2D(v1.[<$t:lower>](v2))),
              (Object::Vec3D(v1), Object::Vec3D(v2)) => Ok(Object::Vec3D(v1.[<$t:lower>](v2))),
              (Object::Number(rhs), Object::Vec2D(lhs)) | (Object::Vec2D(lhs), Object::Number(rhs)) => Ok(lhs.[<$t:lower>](rhs).into()),
              (Object::Number(rhs), Object::Vec3D(lhs)) | (Object::Vec3D(lhs), Object::Number(rhs)) => Ok(lhs.[<$t:lower>](rhs).into()),
              (Object::Number(lhs), Object::Number(rhs)) => Ok(lhs.[<$t:lower>](rhs).into()),
              (Object::Null, _) => Ok(other.clone()),
              (_, Object::Null) => Ok(self.clone()),
              _ => {
                return Err(EvaluationErrorRepr::TypeError {obj1: self, obj2: other, reason: TypeErrorReason::IncompatibleTypeOp(Operation::[<$t>])})
              }

            }
          }
      }
    }
  )*);
}

macro_rules! obj_bit_ops_impl {
  ($t:ident, $alias:path) => (
     impl $t for Object {
       type Output = Result<Self, EvaluationErrorRepr>;

       paste! {
           fn [<$t:lower>](self, other: Self) -> Self::Output {
             match (self, other) {
               (Object::Number(lhs), Object::Number(rhs)) => Ok((lhs as i64).[<$t:lower>](rhs as i64).into()),
               (Object::Bool(lhs), Object::Bool(rhs)) => Ok((lhs as i64).[<$t:lower>](rhs as i64).into()),
               (Object::Null, _) => Ok(other.clone()),
               (_, Object::Null) => Ok(self.clone()),
               _ => {
                 return Err(EvaluationErrorRepr::TypeError {obj1: self, obj2: other, reason: TypeErrorReason::IncompatibleTypeOp(Operation::[<$alias>])})
               }
             }
           }
       }
     }
   )
}

macro_rules! obj_assign_ops_impl {
  ($($t:ident)*) => (
    impl Object {
      $(
        paste! {
         #[allow(unused)]
         pub(crate) fn [<checked_ $t:lower _assign>](&mut self, other: Self) -> Result<(), EvaluationErrorRepr> {
           match (*self, other) {
              (Object::Vec2D(v1), Object::Vec2D(v2)) => *self = v1.[<$t:lower>](v2).into(),
              (Object::Vec3D(v1), Object::Vec3D(v2)) => *self = v1.[<$t:lower>](v2).into(),
              (Object::Number(rhs), Object::Vec2D(lhs)) | (Object::Vec2D(lhs), Object::Number(rhs)) => *self = lhs.[<$t:lower>](rhs).into(),
              (Object::Number(rhs), Object::Vec3D(lhs)) | (Object::Vec3D(lhs), Object::Number(rhs)) => *self = lhs.[<$t:lower>](rhs).into(),
              (Object::Number(lhs), Object::Number(n_rhs)) => *self = lhs.[<$t:lower>](n_rhs).into(),
              (Object::Null, _) => {*self = other},
              _ => {
                return Err(EvaluationErrorRepr::TypeError {obj1: *self, obj2: other, reason: TypeErrorReason::IncompatibleTypeOp(Operation::[<$t>])})
              }
           };

           Ok(())
         }
        }
      )*
    }
  );
}

obj_ops_impl! {Add Sub Mul Div Rem}
obj_bit_ops_impl! {BitAnd, And}
obj_bit_ops_impl! {BitOr, Or}
obj_bit_ops_impl! {BitXor, Exp}
obj_assign_ops_impl! {Add Sub Mul Div Rem}

impl Object {
  pub(crate) fn access_field(&self, field: String) -> Result<Self, EvaluationErrorRepr> {
    match self {
      Object::Vec2D(vec2_d) => vec2_d.access_field(field),
      Object::Vec3D(vec3_d) => vec3_d.access_field(field),
      _ => Err(EvaluationErrorRepr::NoFields(self.kind())),
    }
  }

  pub(crate) fn powf(&self, exponent: Object) -> Result<Self, EvaluationErrorRepr> {
    match (self, exponent) {
      (Object::Number(n), Object::Number(exponent)) => Ok(n.powf(exponent).into()),
      (Object::Vec2D(v), Object::Number(n)) => Ok(v.powf(n).into()),
      (Object::Vec3D(v), Object::Number(n)) => Ok(v.powf(n).into()),

      _ => Err(EvaluationErrorRepr::TypeError {
        obj1: self.clone(),
        obj2: exponent.clone(),
        reason: TypeErrorReason::IncompatibleTypeOp(Operation::Exp),
      }),
    }
  }

  pub(crate) fn stringify_type(&self) -> String {
    format!("{}", self.kind())
  }

  pub(crate) fn kind(&self) -> ObjectKind {
    match self {
      Null => ObjectKind::Null,
      Object::Number(_) => ObjectKind::Number,
      Object::Vec2D(_) => ObjectKind::Vec2D,
      Object::Vec3D(_) => ObjectKind::Vec3D,
      Object::Bool(_) => ObjectKind::Bool,
    }
  }
}

impl Neg for Object {
  type Output = Self;

  fn neg(self) -> Self::Output {
    match self {
      Object::Number(n) => (-n).into(),
      Object::Vec2D(vector) => (-vector).into(),
      Object::Vec3D(vector) => (-vector).into(),
      Object::Null => self,
      Object::Bool(b) => (!b).into(),
    }
  }
}

impl Display for Object {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Object::Null => write!(f, "null"),
      Object::Number(n) => write!(f, "{}", n),
      Object::Vec2D(vector) => write!(f, "{}", vector),
      Object::Vec3D(vector) => write!(f, "{}", vector),
      Object::Bool(b) => write!(f, "{}", b),
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

impl TryInto<Vec2D> for Object {
  type Error = EvaluationErrorRepr;

  fn try_into(self) -> Result<Vec2D, Self::Error> {
    match self {
      Object::Vec2D(n) => Ok(n),
      _ => Err(EvaluationErrorRepr::TypeError {
        obj1: self,
        obj2: Null,
        reason: InvalidCast(ObjectKind::Vec2D),
      }),
    }
  }
}

impl TryInto<Vec3D> for Object {
  type Error = EvaluationErrorRepr;

  fn try_into(self) -> Result<Vec3D, Self::Error> {
    match self {
      Object::Vec3D(n) => Ok(n),
      _ => Err(EvaluationErrorRepr::TypeError {
        obj1: self,
        obj2: Null,
        reason: InvalidCast(ObjectKind::Vec3D),
      }),
    }
  }
}

impl From<f64> for Object {
  fn from(value: f64) -> Self {
    Object::Number(value)
  }
}

impl From<i64> for Object {
  fn from(value: i64) -> Self {
    Object::Number(value as f64)
  }
}

impl From<u64> for Object {
  fn from(value: u64) -> Self {
    Object::Number(value as f64)
  }
}

impl From<Vec2D> for Object {
  fn from(value: Vec2D) -> Self {
    Object::Vec2D(value)
  }
}

impl From<Vec3D> for Object {
  fn from(value: Vec3D) -> Self {
    Object::Vec3D(value)
  }
}

impl From<bool> for Object {
  fn from(value: bool) -> Self {
    Object::Bool(value)
  }
}
