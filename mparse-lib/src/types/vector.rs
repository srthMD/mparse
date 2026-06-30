use crate::{
  eval::{EvaluationErrorRepr, TypeErrorReason},
  functions::FunctionEvaluationError,
  types::object::{Object, ObjectKind},
};
use paste::paste;
use std::{
  fmt::Display,
  ops::{Add, Div, Mul, Neg, Rem, Sub},
};

#[derive(Debug, PartialEq, PartialOrd, Clone, Copy)]
pub enum Vector {
  Vec2D { x: f64, y: f64 },
  Vec3D { x: f64, y: f64, z: f64 },
}

impl Vector {
  pub(crate) fn magnitude(&self) -> f64 {
    match self {
      Vector::Vec2D { x, y } => f64::sqrt(x.powi(2) + y.powi(2)),
      Vector::Vec3D { x, y, z } => f64::sqrt(x.powi(2) + y.powi(2) + z.powi(2)),
    }
  }

  pub(crate) fn unit(&self) -> Self {
    *self / self.magnitude()
  }

  pub(crate) fn project(&self, other: Self) -> Result<Self, FunctionEvaluationError> {
    match (self, other) {
      (Vector::Vec2D { x: _, y: _ }, Vector::Vec2D { x: _, y: _ })
      | (Vector::Vec3D { x: _, y: _, z: _ }, Vector::Vec3D { x: _, y: _, z: _ }) => {
        Ok(*self * (self.dot(other)? / self.magnitude().powi(2)))
      }
      _ => Err(FunctionEvaluationError::InvalidArgument {
        obj: ObjectKind::Vector(other.kind()),
        expected: ObjectKind::Vector(self.kind()),
        idx: 1,
      }),
    }
  }

  pub(crate) fn dot(&self, other: Self) -> Result<f64, FunctionEvaluationError> {
    match (self, other) {
      (Vector::Vec2D { x: x1, y: y1 }, Vector::Vec2D { x: x2, y: y2 }) => Ok(x1 * x2 + y1 * y2),
      (
        Vector::Vec3D {
          x: x1,
          y: y1,
          z: z1,
        },
        Vector::Vec3D {
          x: x2,
          y: y2,
          z: z2,
        },
      ) => Ok(x1 * x2 + y1 * y2 + z1 * z2),
      _ => Err(FunctionEvaluationError::InvalidArgument {
        obj: ObjectKind::Vector(other.kind()),
        expected: ObjectKind::Vector(self.kind()),
        idx: 1,
      }),
    }
  }

  pub(crate) fn angle(&self, other: Self) -> Result<f64, FunctionEvaluationError> {
    match (self, other) {
      (Vector::Vec2D { x: _, y: _ }, Vector::Vec2D { x: _, y: _ })
      | (Vector::Vec3D { x: _, y: _, z: _ }, Vector::Vec3D { x: _, y: _, z: _ }) => Ok(
        (self.dot(other).expect("unreachable") / (self.magnitude() * other.magnitude()))
          .clamp(-1.0, 1.0)
          .acos(),
      ),
      _ => Err(FunctionEvaluationError::InvalidArgument {
        obj: ObjectKind::Vector(other.kind()),
        expected: ObjectKind::Vector(self.kind()),
        idx: 1,
      }),
    }
  }

  pub(crate) fn signed_angle_2d(&self, other: Self) -> Result<f64, FunctionEvaluationError> {
    match (self, other) {
      (Vector::Vec2D { x: _, y: _ }, Vector::Vec2D { x: _, y: _ }) => {
        let cross: f64 = self
          .cross(other)
          .expect("unreachable")
          .try_into()
          .expect("unreachable");
        Ok(cross.atan2(self.dot(other).expect("unreachable")))
      }
      _ => Err(FunctionEvaluationError::InvalidArgument {
        obj: ObjectKind::Vector(other.kind()),
        expected: ObjectKind::Vector(self.kind()),
        idx: 1,
      }),
    }
  }

  pub(crate) fn cross(&self, other: Self) -> Result<Object, FunctionEvaluationError> {
    match (self, other) {
      (Vector::Vec2D { x: x1, y: y1 }, Vector::Vec2D { x: x2, y: y2 }) => {
        Ok((x1 * y2 - y1 * x2).into())
      }
      (
        Vector::Vec3D {
          x: x1,
          y: y1,
          z: z1,
        },
        Vector::Vec3D {
          x: x2,
          y: y2,
          z: z2,
        },
      ) => Ok(
        Vector::Vec3D {
          x: y1 * z2 - z1 * y2,
          y: z1 * x2 - x1 * z2,
          z: x1 * y2 - y1 * x2,
        }
        .into(),
      ),
      _ => Err(FunctionEvaluationError::InvalidArgument {
        obj: ObjectKind::Vector(other.kind()),
        expected: ObjectKind::Vector(self.kind()),
        idx: 1,
      }),
    }
  }

  pub(crate) fn distance(&self, other: Self) -> Result<f64, FunctionEvaluationError> {
    if self.kind() != other.kind() {}

    match (self, other) {
      (Vector::Vec2D { x: _, y: _ }, Vector::Vec2D { x: _, y: _ })
      | (Vector::Vec3D { x: _, y: _, z: _ }, Vector::Vec3D { x: _, y: _, z: _ }) => {
        Ok((*self - other).expect("unreachable").magnitude())
      }
      _ => Err(FunctionEvaluationError::InvalidArgument {
        obj: ObjectKind::Vector(other.kind()),
        expected: ObjectKind::Vector(self.kind()),
        idx: 1,
      }),
    }
  }

  pub(crate) fn ceil(&self) -> Self {
    self.apply_func(f64::ceil)
  }

  pub(crate) fn floor(&self) -> Self {
    self.apply_func(f64::floor)
  }

  pub(crate) fn abs(&self) -> Self {
    self.apply_func(f64::abs)
  }

  pub(crate) fn kind(&self) -> VectorKind {
    match self {
      Vector::Vec2D { x: _, y: _ } => VectorKind::Vec2D,
      Vector::Vec3D { x: _, y: _, z: _ } => VectorKind::Vec3D,
    }
  }

  fn apply_func(&self, func: impl Fn(f64) -> f64) -> Self {
    match self {
      Vector::Vec2D { x: x1, y: y1 } => Vector::Vec2D {
        x: func(*x1),
        y: func(*y1),
      },
      Vector::Vec3D {
        x: x1,
        y: y1,
        z: z1,
      } => Vector::Vec3D {
        x: func(*x1),
        y: func(*y1),
        z: func(*z1),
      },
    }
  }

  fn apply_op(
    &self,
    rhs: &Self,
    operator: impl Fn(f64, f64) -> f64,
  ) -> Result<Self, EvaluationErrorRepr> {
    let res = match (self, rhs) {
      (Vector::Vec2D { x: x1, y: y1 }, Vector::Vec2D { x: x2, y: y2 }) => Vector::Vec2D {
        x: operator(*x1, *x2),
        y: operator(*y1, *y2),
      },
      (
        Vector::Vec3D {
          x: x1,
          y: y1,
          z: z1,
        },
        Vector::Vec3D {
          x: x2,
          y: y2,
          z: z2,
        },
      ) => Vector::Vec3D {
        x: operator(*x1, *x2),
        y: operator(*y1, *y2),
        z: operator(*z1, *z2),
      },
      _ => {
        return Err(EvaluationErrorRepr::TypeError {
          obj1: Object::Vector(self.clone()),
          obj2: Object::Vector(rhs.clone()),
          reason: TypeErrorReason::IncompatibleTypes,
        });
      }
    };

    Ok(res)
  }

  fn apply_op_f64(&self, rhs: f64, operator: impl Fn(f64, f64) -> f64) -> Self {
    match self {
      Vector::Vec2D { x: x1, y: y1 } => Vector::Vec2D {
        x: operator(*x1, rhs),
        y: operator(*y1, rhs),
      },
      Vector::Vec3D {
        x: x1,
        y: y1,
        z: z1,
      } => Vector::Vec3D {
        x: operator(*x1, rhs),
        y: operator(*y1, rhs),
        z: operator(*z1, rhs),
      },
    }
  }

  // i dont really know why somebody would want to raise a vector to an exponent but like you do you i guess
  pub(crate) fn powf(&self, exponent: f64) -> Self {
    match self {
      Vector::Vec2D { x: x1, y: y1 } => Vector::Vec2D {
        x: x1.powf(exponent),
        y: y1.powf(exponent),
      },
      Vector::Vec3D {
        x: x1,
        y: y1,
        z: z1,
      } => Vector::Vec3D {
        x: x1.powf(exponent),
        y: y1.powf(exponent),
        z: z1.powf(exponent),
      },
    }
  }
}

impl Neg for Vector {
  type Output = Self;

  fn neg(self) -> Self::Output {
    match self {
      Vector::Vec2D { x: x1, y: y1 } => Vector::Vec2D { x: -x1, y: -y1 },

      Vector::Vec3D {
        x: x1,
        y: y1,
        z: z1,
      } => Vector::Vec3D {
        x: -x1,
        y: -y1,
        z: -z1,
      },
    }
  }
}

impl Display for Vector {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Vector::Vec2D { x, y } => write!(f, "{{ x: {}, y: {} }}", x, y),
      Vector::Vec3D { x, y, z } => write!(f, "{{ x: {}, y: {}, z: {} }}", x, y, z),
    }
  }
}

macro_rules! vec_ops_impl {
  ($($t:ident)*) => ($(
    impl $t for Vector {
      type Output = Result<Self, EvaluationErrorRepr>;

      paste! {
          fn [<$t:lower>](self, rhs: Self) -> Self::Output {
            Self::apply_op(&self, &rhs, f64::[<$t:lower>])
          }
      }
    }

    impl $t<f64> for Vector {
      type Output = Self;

      paste! {
          fn [<$t:lower>](self, rhs: f64) -> Self::Output {
            Self::apply_op_f64(&self, rhs, f64::[<$t:lower>])
          }
      }
    }
  )*)
}

vec_ops_impl! {Add Sub Mul Div Rem}

#[derive(Debug, PartialEq, PartialOrd, Clone, Copy)]
pub enum VectorKind {
  Vec2D,
  Vec3D,
  Any,
}

impl Display for VectorKind {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      VectorKind::Vec2D => write!(f, "vec2d"),
      VectorKind::Vec3D => write!(f, "vec3d"),
      VectorKind::Any => write!(f, "vec"),
    }
  }
}
