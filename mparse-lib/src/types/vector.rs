use paste::paste;
use std::{
  fmt::Display,
  ops::{Add, Div, Mul, Neg, Rem, Sub},
};

#[derive(Debug, PartialEq, PartialOrd, Clone, Copy)]
pub struct Vec2D {
  pub x: f64,
  pub y: f64,
}

#[derive(Debug, PartialEq, PartialOrd, Clone, Copy)]
pub struct Vec3D {
  pub x: f64,
  pub y: f64,
  pub z: f64,
}

impl Vec2D {
  pub(crate) fn magnitude(&self) -> f64 {
    (self.x.powi(2) + self.y.powi(2)).sqrt()
  }

  pub(crate) fn unit(&self) -> Self {
    *self / self.magnitude()
  }

  pub(crate) fn project(&self, other: Self) -> Self {
    *self * (self.dot(other) / self.magnitude().powi(2))
  }

  pub(crate) fn dot(&self, other: Self) -> f64 {
    self.x * other.x + self.y * other.y
  }

  pub(crate) fn angle(&self, other: Self) -> f64 {
    (self.dot(other) / (self.magnitude() * other.magnitude()))
      .clamp(-1.0, 1.0)
      .acos()
  }

  pub(crate) fn signed_angle_2d(&self, other: Self) -> f64 {
    let cross: f64 = self.cross(other);
    cross.atan2(self.dot(other))
  }

  pub(crate) fn cross(&self, other: Self) -> f64 {
    self.x * other.y - self.y * other.x
  }

  pub(crate) fn distance(&self, other: Self) -> f64 {
    (*self - other).magnitude()
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

  pub(crate) fn apply_func(&self, func: impl Fn(f64) -> f64) -> Self {
    Self {
      x: func(self.x),
      y: func(self.y),
    }
  }

  pub(crate) fn apply_op(&self, rhs: &Self, operator: impl Fn(f64, f64) -> f64) -> Self {
    Self {
      x: operator(self.x, rhs.x),
      y: operator(self.y, rhs.y),
    }
  }

  pub(crate) fn apply_op_f64(&self, rhs: f64, operator: impl Fn(f64, f64) -> f64) -> Self {
    Self {
      x: operator(self.x, rhs),
      y: operator(self.y, rhs),
    }
  }

  pub(crate) fn powf(&self, exponent: f64) -> Self {
    self.apply_op(
      &Self {
        x: exponent,
        y: exponent,
      },
      f64::powf,
    )
  }
}

impl Vec3D {
  pub(crate) fn magnitude(&self) -> f64 {
    (self.x.powi(2) + self.x.powi(2) + self.z.powi(2)).sqrt()
  }

  pub(crate) fn unit(&self) -> Self {
    *self / self.magnitude()
  }

  pub(crate) fn project(&self, other: Self) -> Self {
    *self * (self.dot(other) / self.magnitude().powi(2))
  }

  pub(crate) fn dot(&self, other: Self) -> f64 {
    self.x * other.x + self.y * other.y + self.z * other.z
  }

  pub(crate) fn cross(&self, other: Self) -> Self {
    Vec3D {
      x: self.y * other.z - self.z * other.y,
      y: self.z * other.x - self.x * other.z,
      z: self.x * other.y - self.y - other.x,
    }
  }

  pub(crate) fn distance(&self, other: Self) -> f64 {
    (*self - other).magnitude()
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

  pub(crate) fn apply_func(&self, func: impl Fn(f64) -> f64) -> Self {
    Self {
      x: func(self.x),
      y: func(self.y),
      z: func(self.z),
    }
  }

  pub(crate) fn apply_op(&self, rhs: &Self, operator: impl Fn(f64, f64) -> f64) -> Self {
    Self {
      x: operator(self.x, rhs.x),
      y: operator(self.y, rhs.y),
      z: operator(self.z, rhs.z),
    }
  }

  pub(crate) fn apply_op_f64(&self, rhs: f64, operator: impl Fn(f64, f64) -> f64) -> Self {
    Self {
      x: operator(self.x, rhs),
      y: operator(self.y, rhs),
      z: operator(self.z, rhs),
    }
  }

  pub(crate) fn powf(&self, exponent: f64) -> Self {
    self.apply_op(
      &Self {
        x: exponent,
        y: exponent,
        z: exponent,
      },
      f64::powf,
    )
  }
}

impl Display for Vec2D {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{{ x: {}, y: {} }}", self.x, self.y)
  }
}

impl Display for Vec3D {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{{ x: {}, y: {}, z: {} }}", self.x, self.y, self.z)
  }
}

macro_rules! vec_ops_impl {
  ($vec:ident, $($t:ident)*) => ($(
    impl $t for $vec {
      type Output = Self;

      paste! {
          fn [<$t:lower>](self, rhs: Self) -> Self::Output {
            Self::apply_op(&self, &rhs, f64::[<$t:lower>])
          }
      }
    }

    impl $t<f64> for $vec {
      type Output = Self;

      paste! {
          fn [<$t:lower>](self, rhs: f64) -> Self::Output {
            Self::apply_op_f64(&self, rhs, f64::[<$t:lower>])
          }
      }
    }
  )*)
}

macro_rules! vec_unary_impl {
  ($vec:ident, $($t:ident)*) => {
    $(
      impl $t for $vec {
            type Output = Self;

            paste! {
                fn [<$t:lower>](self) -> Self::Output {
                  Self::apply_func(&self, f64::[<$t:lower>])
                }
            }
          }
    )*
  };
}

vec_ops_impl! {Vec2D, Add Sub Mul Div Rem}
vec_ops_impl! {Vec3D, Add Sub Mul Div Rem}
vec_unary_impl! {Vec2D, Neg}
vec_unary_impl! {Vec3D, Neg}
