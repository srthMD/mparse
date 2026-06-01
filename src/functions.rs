//! Enum representation of all the supported functions in the parser.
//! A function is consisted of 2 parts: its type and an optional base.
//! The type is made of a [FunctionType] that describes what type of function it is.
//! And the base is a primitive number that can serve different purposes
//! depending on the function it's attached to, if bases are supported on that function.
//! Putting a base on the log function (not to be confused with ln or lg), is self explanatory,
//! meanwhile putting a base on the root function acts as taking the n'th root (where n is the base)
//! of a primitive. Bases can be written in the form 'function_n(...)', where n must be a
//! primitive number and the function must support bases, otherwise an error is thrown.
//! See [FunctionType::supports_base] to see all functions that support bases.

use std::fmt::Display;

#[cfg(feature = "rand")]
use rand::random;
use thiserror::Error;

/// Errors that can be thrown during evaluation.
#[derive(Debug, PartialEq, PartialOrd, Error)]
pub enum FunctionEvaluationError {
  /// Thrown when a function recieves a bad argument it cannot work with.
  #[error("function recieved invalid argument {0}")]
  InvalidArgument(f64),
  /// Thrown when a function that expects a base does not have one,
  /// this should generally not be possible as it should be catched
  /// in tokenization, and is likely a result of faulty code.
  #[error("no base found for function during evaluation time (likely internal error)")]
  NoBase,
  /// Thrown when an overflow happens within function evaluation.
  #[error("number overflow occured during function evaluation")]
  NumberOverflow,
  /// Thrown when NaN is produced anywhere within the function evaluation.
  #[error("NaN was produced during function evaluation")]
  ProducedNaN,
  /// Thrown when a division by zero is expected within the function evaluation.
  #[error("division by zero caught during function evaluation")]
  DivisionByZero,
  /// Thrown when I forget to acctualy implement the evaluation part of a function (oops).
  #[error("funciton is not internally implemented")]
  UnimplementedFunction,
}

/// QOL macro for me just to be able to display function names from the enum name.
macro_rules! enum_with_str {
    (
      $(#[$meta:meta])*
      enum $name:ident {
            $($variant:ident),* $(,)?
        }
    ) => {
        #[derive(Debug, PartialEq, Clone, Copy, PartialOrd)]
        $(#[$meta])*
        pub enum $name {
            $($variant),*
        }

        impl $name {
            fn __as_str(&self) -> &'static str {
                match self {
                    $(
                        Self::$variant => stringify!($variant),
                    )*
                }
            }

            pub fn as_str(&self) -> String {
              self.__as_str().to_lowercase()
            }
        }
    };
}

enum_with_str! {
  /// List of all the supported functions.
  enum FunctionType {
    Sqrt,
    Cbrt,
    Root,
    Log,
    Lg,
    Ln,
    Sin,
    Cos,
    Tan,
    Arcsin,
    Arccos,
    Arctan,
    Sinh,
    Cosh,
    Tanh,
    Arcsinh,
    Arccosh,
    Arctanh,
    Ceil,
    Floor,
    Sign,
    Rad,
    Deg,
    Gcf,
    Lcm,
    Abs,
    Rand,
    Mean,
  }
}

impl FunctionType {
  /// Attempts to construct a function type from a string representation.
  /// Returns None if it cannot find a matching type.
  pub(crate) fn from_string(st: &String) -> Option<Self> {
    let lower = st.to_lowercase();

    match lower.as_str() {
      "log" => Some(Self::Log),
      "lg" => Some(Self::Lg),
      "ln" => Some(Self::Ln),
      "sqrt" => Some(Self::Sqrt),
      "cbrt" => Some(Self::Cbrt),
      "sin" => Some(Self::Sin),
      "cos" => Some(Self::Cos),
      "tan" => Some(Self::Tan),
      "arcsin" | "asin" => Some(Self::Arcsin),
      "arccos" | "acos" => Some(Self::Arccos),
      "arctan" | "atan" => Some(Self::Arctan),
      "sinh" => Some(Self::Sinh),
      "cosh" => Some(Self::Cosh),
      "tanh" => Some(Self::Tanh),
      "arcsinh" | "asinh" => Some(Self::Arcsinh),
      "arccosh" | "acosh" => Some(Self::Arccosh),
      "arctanh" | "atanh" => Some(Self::Arctanh),
      "ceil" => Some(Self::Ceil),
      "floor" => Some(Self::Floor),
      "root" => Some(Self::Root),
      "sign" => Some(Self::Sign),
      "rad" | "radians" => Some(Self::Rad),
      "deg" | "degrees" => Some(Self::Deg),
      "gcf" | "gcd" => Some(Self::Gcf),
      "lcm" => Some(Self::Lcm),
      "abs" => Some(Self::Abs),
      "rand" | "rng" | "random" => Some(Self::Rand),
      "mean" | "avg" | "average" => Some(Self::Mean),
      _ => None,
    }
  }

  /// Returns true if a function can have a base attached to it.
  /// If a function annotated with a base does not support a base,
  /// an error will be thrown somewhere within the parsing process.
  pub(crate) fn supports_base(&self) -> bool {
    match self {
      Self::Log | Self::Root => true,
      _ => false,
    }
  }
}

/// Encapsulating structure describing a function and its base if it
/// has one.
#[derive(Debug, PartialEq, Clone, Copy, PartialOrd)]
pub struct Function {
  ftype: FunctionType,
  sub: Option<f64>,
}

impl Function {
  pub fn new(ftype: FunctionType, sub: Option<f64>) -> Self {
    Self { ftype, sub: sub }
  }

  #[allow(dead_code)]
  pub fn with_no_base(ftype: FunctionType) -> Self {
    Self::new(ftype, None)
  }

  pub fn get_function_type(&self) -> FunctionType {
    self.ftype
  }

  pub fn has_base(&self) -> bool {
    self.sub.is_some()
  }

  pub fn get_base(&self) -> Option<f64> {
    self.sub
  }

  pub fn get_base_unwrap(&self) -> f64 {
    self.sub.unwrap()
  }

  /// Returns how many arguments the function STRICTLY requires.
  pub fn arg_count(&self) -> usize {
    match self.ftype {
      FunctionType::Gcf | FunctionType::Lcm => 2,
      _ => 1,
    }
  }

  /// Returns true if the function requires arguments.
  pub fn requires_args(&self) -> bool {
    match self.ftype {
      FunctionType::Rand => false,
      _ => true,
    }
  }

  /// Returns true if the function supports a varaible number
  /// of arguments.
  pub fn supports_varargs(&self) -> bool {
    match self.ftype {
      _ => false,
    }
  }

  /// Evaluates a function given some args.
  /// Evaluation can fail, see [FunctionEvaluationError] for more info.
  pub fn eval(&self, args: &[f64], deg_mode: bool) -> Result<f64, FunctionEvaluationError> {
    let as_rads = if !args.is_empty() {
      if deg_mode {
        args[0].to_radians()
      } else {
        args[0]
      }
    } else {
      0f64
    };

    let res = match self.ftype {
      FunctionType::Sqrt => args[0].sqrt(),
      FunctionType::Cbrt => args[0].cbrt(),
      FunctionType::Lg => args[0].log10(),
      FunctionType::Ln => args[0].ln(),
      FunctionType::Sin => as_rads.sin(),
      FunctionType::Cos => as_rads.cos(),
      FunctionType::Tan => as_rads.tan(),
      FunctionType::Arcsin => as_rads.asin(),
      FunctionType::Arccos => as_rads.acos(),
      FunctionType::Arctan => as_rads.atan(),
      FunctionType::Sinh => args[0].sinh(),
      FunctionType::Cosh => args[0].cosh(),
      FunctionType::Tanh => args[0].tanh(),
      FunctionType::Arcsinh => args[0].asinh(),
      FunctionType::Arccosh => args[0].acosh(),
      FunctionType::Arctanh => args[0].atanh(),
      FunctionType::Ceil => args[0].ceil(),
      FunctionType::Floor => args[0].floor(),
      FunctionType::Sign => args[0].signum(),
      FunctionType::Rad => args[0].to_degrees(),
      FunctionType::Deg => args[0].to_radians(),
      FunctionType::Abs => args[0].abs(),

      FunctionType::Root => {
        if let Some(base) = self.get_base() {
          args[0].powf(1f64 / base)
        } else {
          return Err(FunctionEvaluationError::NoBase);
        }
      }

      FunctionType::Log => {
        if let Some(base) = self.get_base() {
          args[0].log(base)
        } else {
          return Err(FunctionEvaluationError::NoBase);
        }
      }

      FunctionType::Gcf => {
        helpers::check_args_greater_than(args, 1);
        helpers::gcf(args[0], args[1])
      }

      FunctionType::Lcm => {
        helpers::check_args_greater_than(args, 1);
        helpers::lcm(args[0], args[1])?
      }

      #[cfg(feature = "rand")]
      FunctionType::Rand => random(),

      FunctionType::Mean => {
        helpers::check_args_greater_than(args, 1);
        helpers::mean(args)?
      }

      #[allow(unreachable_patterns)]
      _ => return Err(FunctionEvaluationError::UnimplementedFunction),
    };

    if res.is_nan() {
      Err(FunctionEvaluationError::ProducedNaN)
    } else if res.is_infinite() {
      Err(FunctionEvaluationError::NumberOverflow)
    } else {
      Ok(res)
    }
  }
}

impl Display for FunctionType {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self.as_str())
  }
}

mod helpers {
  use std::ops::Div;

  use crate::functions::FunctionEvaluationError;

  #[inline]
  pub fn check_args_greater_than(args: &[f64], req_len: usize) {
    if args.len() <= req_len {
      panic!("evaluator improperly handled argument length check");
    }
  }

  // TODO: probably change this to an iterative solution
  pub fn gcf(a: f64, b: f64) -> f64 {
    if b == 0f64 { a } else { gcf(b, a % b) }
  }

  pub fn lcm(a: f64, b: f64) -> Result<f64, FunctionEvaluationError> {
    if a == 0f64 || b == 0f64 {
      return Ok(0f64);
    }

    let ab = a * b;
    if !ab.is_infinite() {
      return match gcf(a, b) {
        0f64 => Err(FunctionEvaluationError::DivisionByZero),
        g_res => Ok(ab / g_res),
      };
    }

    Err(FunctionEvaluationError::NumberOverflow)
  }

  pub fn mean(args: &[f64]) -> Result<f64, FunctionEvaluationError> {
    let len = args.len();
    if len == 0 {
      return Err(FunctionEvaluationError::DivisionByZero);
    }

    let sum: f64 = args.iter().sum();
    if sum.is_infinite() {
      Err(FunctionEvaluationError::NumberOverflow)
    } else {
      Ok(sum.div(len as f64))
    }
  }

  // TODO: add mean test cases
  #[cfg(test)]
  mod tests {
    use crate::functions::helpers::{gcf, lcm};

    #[test]
    fn test_gcf() {
      assert_eq!(gcf(6f64, 3f64), 3f64);
      assert_eq!(gcf(99f64, 2f64), 1f64);
      assert_eq!(gcf(1f64, 1f64), 1f64);
      assert_eq!(gcf(6f64, 0f64), 6f64);
      assert_eq!(gcf(0f64, 24f64), 24f64);
      assert_eq!(gcf(2934f64, 24f64), 6f64);
    }

    #[test]
    fn test_lcm() {
      assert_eq!(lcm(12f64, 18f64).unwrap(), 36f64);
      assert_eq!(lcm(82f64, 4f64).unwrap(), 164f64);
      assert_eq!(lcm(3f64, 35f64).unwrap(), 105f64);
      assert_eq!(lcm(0f64, 18f64).unwrap(), 0f64);
      assert_eq!(lcm(18f64, 0f64).unwrap(), 0f64);
    }
  }
}
