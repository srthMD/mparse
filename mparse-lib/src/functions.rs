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

use std::{collections::HashMap, fmt::Display};

use crate::{
  eval::EvaluationErrorRepr::{self},
  types::object::{
    Object::{self},
    ObjectKind::{self},
  },
};

use mparse_proc_macros::StringifyEnum;
use once_cell::sync::Lazy;
use paste::paste;
#[allow(unused_imports)]
#[cfg(feature = "rand")]
use rand::random;

use thiserror::Error;

/// Similar to builtin_impl! except that it will look in f64::... for the function
/// and will always have one Number arg and a Number return type.
macro_rules! std_wrapper_builtin {
    ($map:expr, $($ty:ident)+) => {
       $(
         paste! {
           fn [<$ty:lower _wrapper>](_: &Function, args: &[Object]) -> BuiltinFnOutput {
             let flt: f64 = args[0].try_into()?;
             Ok(f64::[<$ty:lower>](flt).into())
           }

           $map.insert(FunctionType::$ty, Builtin {
             func: [<$ty:lower _wrapper>],
             arg_t: BuiltinArgs::Explicit(Box::new([ObjectKind::Number])),
             returns: ObjectKind::Number,
           });
         }
       )*
    };
}

/// Same as std_wrapper_builtin but it searches f64::... via the $func ident instead of using $ty:lower
macro_rules! std_wrapper_alias {
  ($map:expr, $ty:ident, $func:ident) => {
    paste! {
      fn [<$ty:lower _wrapper>](_: &Function, args: &[Object]) -> BuiltinFnOutput {
        let flt: f64 = args[0].try_into()?;
        Ok(f64::[<$func:lower>](flt).into())
      }

      $map.insert(FunctionType::$ty, Builtin {
        func: [<$ty:lower _wrapper>],
        arg_t: BuiltinArgs::Explicit(Box::new([ObjectKind::Number])),
        returns: ObjectKind::Number,
      });
    }
  };
}

// this is so fucking awesome and a war crime at the same time
/// Constructs a builtin given the FunctionType. Looks in impls module for its respective functions.
macro_rules! builtin_impl {
  ($map:expr, $($ty:ident)+) => {
    $(
      paste! {
        $map.insert(FunctionType::$ty,
          Builtin {
            func: impls::[<$ty:lower>],
            arg_t: impls::[<$ty:lower _args>](),
            returns: impls::[<$ty:lower _returns>]()
          });
      }
    )*
  };
}

static BUILTIN_FUNCTIONS: Lazy<HashMap<FunctionType, Builtin>> = Lazy::new(|| {
  let mut map = HashMap::<FunctionType, Builtin>::new();
  std_wrapper_builtin! { map,
    Sqrt Cbrt Sin Cos Tan Sinh Cosh Tanh Ln
  }

  std_wrapper_alias! {map, Lg, log10}
  std_wrapper_alias! {map, Sign, signum}
  std_wrapper_alias! {map, Arcsin, asin}
  std_wrapper_alias! {map, Arccos, acos}
  std_wrapper_alias! {map, Arctan, atan}
  std_wrapper_alias! {map, Arcsinh, asinh}
  std_wrapper_alias! {map, Arccosh, acosh}
  std_wrapper_alias! {map, Arctanh, atanh}
  std_wrapper_alias! {map, Deg, to_degrees}
  std_wrapper_alias! {map, Rad, to_radians}

  builtin_impl! {map,
    Root Log Rand Gcf Lcm Mean Abs Ceil Floor Vec2D Vec3D Magnitude Distance Cross Dot Project Unit Angle SignedAngle Out Binout
  }
  map
});

/// Errors that can be thrown during evaluation.
#[derive(Debug, PartialEq, PartialOrd, Error)]
pub enum FunctionEvaluationError {
  /// Thrown when a function recieves a bad argument it cannot work with.
  /// The string is the expected types name.
  #[error("function expects arg {idx} to be of type {expected}, got {obj}")]
  InvalidArgument {
    obj: ObjectKind,
    expected: ObjectKind,
    idx: usize,
  },
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
  #[error("function is not internally implemented")]
  UnimplementedFunction,
  /// Thrown when a function recieves an invalid number of arguments.
  #[error("function expected {expected} argument(s), got {got} argument(s)")]
  InvalidArgCount { expected: usize, got: usize },
  /// Thrown when a mixed function does not recieve the minimum amount of explicit args.
  #[error("function expects a minimum of {expected} argument(s), got {got} argument(s)")]
  InvalidMininumArgs { expected: usize, got: usize },
}

type BuiltinFnOutput = Result<Object, EvaluationErrorRepr>;
type BuiltinFn = fn(&Function, &[Object]) -> BuiltinFnOutput;

struct Builtin {
  func: BuiltinFn,
  arg_t: BuiltinArgs,
  /// Currently unused.
  #[allow(dead_code)]
  returns: ObjectKind,
}

/// Describes the argument types that a function takes in.
enum BuiltinArgs {
  /// Varargs where all argument types have to match the discriminant provided.
  Varargs(ObjectKind),
  /// Explicit arguments where each nth type provided has to match the nth type provided in the enum.
  Explicit(Box<[ObjectKind]>),
  /// Similar to Explicit except function can take varargs at the end.
  Mixed {
    explicit_t: Box<[ObjectKind]>,
    varargs_t: ObjectKind,
  },
  None,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, StringifyEnum, Hash)]
pub enum FunctionType {
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
  Vec2D,
  Vec3D,
  Magnitude,
  Unit,
  Project,
  Dot,
  Angle,
  Distance,
  Cross,
  SignedAngle,
  Out,
  Binout,
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
      "arcsin" | "asin" => Some(Self::Arcsinh),
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
      "vec2d" | "vec2" => Some(Self::Vec2D),
      "vec3d" | "vec3" => Some(Self::Vec3D),
      "magnitude" => Some(Self::Magnitude),
      "unit" => Some(Self::Unit),
      "proj" | "project" => Some(Self::Project),
      "dot" => Some(Self::Dot),
      "cross" => Some(Self::Cross),
      "dist" | "distance" => Some(Self::Distance),
      "angle" => Some(Self::Angle),
      "sangle" | "signedangle" | "signed_angle" => Some(Self::SignedAngle),
      "out" | "print" => Some(Self::Out),
      "binout" | "binprint" | "printbin" => Some(Self::Binout),
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

  fn is_trig(&self) -> bool {
    match self {
      Self::Sin | Self::Cos | Self::Tan | Self::Arcsin | Self::Arccos | Self::Arctan => true,
      _ => false,
    }
  }

  pub(crate) fn outputs_angle(&self) -> bool {
    match self {
      Self::Angle => true,
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

  /// Evaluates a function given some args.
  /// Evaluation can fail, see [FunctionEvaluationError] for more info.
  pub fn eval(&self, mut args: Vec<Object>, deg_mode: bool) -> Result<Object, EvaluationErrorRepr> {
    let builtin_opt = BUILTIN_FUNCTIONS.get(&self.ftype);
    if let Some(builtin) = builtin_opt {
      match Self::check_args(&args, &builtin.arg_t) {
        Ok(_) => {}
        Err(e) => return Err(EvaluationErrorRepr::FunctionEvaluationError(self.ftype, e)),
      }

      if self.ftype.is_trig() && deg_mode {
        let arg: f64 = (*args.get(0).expect("unreachable")).try_into()?;
        args.remove(0);
        args.insert(0, arg.to_degrees().into());
      }

      (builtin.func)(self, args.as_slice())
    } else {
      panic!("unimplemented function")
    }
  }

  fn check_args(
    user_args: &Vec<Object>,
    builtin_args: &BuiltinArgs,
  ) -> Result<(), FunctionEvaluationError> {
    fn verify_varargs(
      user_args: &Vec<Object>,
      expected_kind: ObjectKind,
    ) -> Result<(), FunctionEvaluationError> {
      for i in 0..user_args.len() {
        let obj = user_args.get(i).expect("unreachable");

        match expected_kind {
          ObjectKind::AnyOf(types) => {
            if !types.contains(&obj.kind()) {
              return Err(FunctionEvaluationError::InvalidArgument {
                obj: obj.kind(),
                expected: expected_kind,
                idx: i,
              });
            }
          }
          _ => {
            if obj.kind() != expected_kind {
              return Err(FunctionEvaluationError::InvalidArgument {
                obj: obj.kind(),
                expected: expected_kind,
                idx: i,
              });
            }
          }
        }
      }

      Ok(())
    }

    fn verify_explicit(
      user_args: &Vec<Object>,
      builtin_args: &Box<[ObjectKind]>,
    ) -> Result<(), FunctionEvaluationError> {
      if user_args.len() != builtin_args.len() {
        return Err(FunctionEvaluationError::InvalidArgCount {
          expected: builtin_args.len(),
          got: user_args.len(),
        });
      }

      for i in 0..user_args.len() {
        let user_arg = user_args.get(i).expect("unreachable");
        let builtin_arg = builtin_args.get(i).expect("unreachable");

        match builtin_arg {
          ObjectKind::AnyOf(types) => {
            if !types.contains(&user_arg.kind()) {
              return Err(FunctionEvaluationError::InvalidArgument {
                obj: user_arg.kind(),
                expected: *builtin_arg,
                idx: i,
              });
            }
          }

          ObjectKind::Any => {
            continue;
          }

          _ => {
            if user_arg.kind() != *builtin_arg {
              return Err(FunctionEvaluationError::InvalidArgument {
                obj: user_arg.kind(),
                expected: *builtin_arg,
                idx: i,
              });
            }
          }
        }
      }

      Ok(())
    }

    fn verify_mixed(
      user_args: &Vec<Object>,
      builtin_args: &Box<[ObjectKind]>,
      vararg_kind: ObjectKind,
    ) -> Result<(), FunctionEvaluationError> {
      if user_args.len() < builtin_args.len() {
        return Err(FunctionEvaluationError::InvalidMininumArgs {
          expected: user_args.len(),
          got: builtin_args.len(),
        });
      }

      let expl_res = verify_explicit(&user_args[0..=builtin_args.len()].to_vec(), builtin_args);
      if expl_res.is_err() {
        return expl_res;
      }

      if user_args.len() > builtin_args.len() {
        let vararg_res = verify_varargs(&user_args[builtin_args.len()..].to_vec(), vararg_kind);
        if vararg_res.is_err() {
          return vararg_res;
        }
      }

      Ok(())
    }

    match builtin_args {
      BuiltinArgs::Varargs(expected_kind) => {
        return verify_varargs(&user_args, *expected_kind);
      }
      BuiltinArgs::Explicit(object_kinds) => return verify_explicit(&user_args, object_kinds),
      BuiltinArgs::Mixed {
        explicit_t,
        varargs_t,
      } => return verify_mixed(&user_args, explicit_t, *varargs_t),
      BuiltinArgs::None => {
        if user_args.len() != 0 {
          return Err(FunctionEvaluationError::InvalidArgCount {
            expected: 0,
            got: user_args.len(),
          });
        }
      }
    }

    Ok(())
  }
}

impl Display for FunctionType {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self.as_str())
  }
}

mod impls {
  use std::vec;

  use rand::random;

  use super::{BuiltinArgs, ObjectKind};
  use crate::{
    eval::EvaluationErrorRepr,
    functions::{
      BuiltinFnOutput, Function,
      FunctionEvaluationError::{self},
      helpers,
    },
    types::{
      object::{
        Object::{self},
        ObjectKind::AnyOf,
      },
      vector::{Vec2D, Vec3D},
    },
  };

  use paste::paste;

  macro_rules! decl_arg_types {
      ($func:ident, vararg: $ty:ident) => {
        paste! {
          #[allow(unused)]
          pub(super) fn [<$func:lower _args>]() -> BuiltinArgs {
            BuiltinArgs::Varargs(ObjectKind::$ty)
          }
        }
      };

      ($func:ident, $($ty:path)+) => {
        paste! {
          #[allow(dead_code)]
          pub(super) fn [<$func:lower _args>]() -> BuiltinArgs {
            BuiltinArgs::Explicit(Box::new([$(ObjectKind::$ty),+]))
          }
        }
      };

      ($func:ident, $(anyof: ($($ty:path),+)),+) => {
        paste! {
          #[allow(dead_code)]
          pub(super) fn [<$func:lower _args>]() -> BuiltinArgs {
            BuiltinArgs::Explicit(Box::new([$(ObjectKind::AnyOf(&[$(ObjectKind::$ty),+])),+]))
          }
        }
      };

      ($func:ident, $($ty:ident)+, vararg: $vararg_ty:ident) => {
        paste! {
          #[allow(dead_code)]
          pub(super) fn [<$func:lower _args>]() -> BuiltinArgs {
            BuiltinArgs::Mixed {
              explicit_t: Box::new([$(ObjectKind::$ty),+]),
              varargs_t: ObjectKind::$vararg_ty
            }
          }
        }
      };

      ($func:ident) => {
        paste! {
          #[allow(dead_code)]
          pub(super) fn [<$func:lower _args>]() -> BuiltinArgs {
            BuiltinArgs::None
          }
        }
      };
  }

  macro_rules! decl_returns {
    ($func:ident, $ty:ident) => {
      paste! {
        #[allow(dead_code)]
        pub(super) fn [<$func:lower _returns>]() -> ObjectKind {
          ObjectKind::$ty
        }
      }
    };

    ($func:ident, $ty:path) => {
      paste! {
        #[allow(dead_code)]
        pub(super) fn [<$func:lower _returns>]() -> ObjectKind {
          ObjectKind::$ty
        }
      }
    };

    ($func:ident, anyof: $($ty:path),+) => {
      paste! {
        #[allow(dead_code)]
        pub(super) fn [<$func:lower _returns>]() -> ObjectKind {
          ObjectKind::AnyOf(&[])
        }
      }
    };
  }

  pub(super) fn rand(_: &Function, _: &[Object]) -> BuiltinFnOutput {
    Ok(random::<f64>().into())
  }
  decl_arg_types!(rand);
  decl_returns! {rand, Number}

  pub(super) fn log(self_func: &Function, args: &[Object]) -> BuiltinFnOutput {
    let arg0: f64 = args[0].try_into()?;
    if self_func.has_base() {
      Ok(f64::log(arg0, self_func.get_base_unwrap()).into())
    } else {
      Err(EvaluationErrorRepr::FunctionEvaluationError(
        self_func.ftype,
        FunctionEvaluationError::NoBase,
      ))
    }
  }
  decl_arg_types! {log, Number}
  decl_returns! {log, Number}

  pub(super) fn root(self_func: &Function, args: &[Object]) -> BuiltinFnOutput {
    let arg0: f64 = args[0].try_into()?;
    if self_func.has_base() {
      let res = f64::powf(arg0, 1f64 / self_func.get_base_unwrap());
      if res.is_nan() {
        Err(EvaluationErrorRepr::FunctionEvaluationError(
          self_func.ftype,
          FunctionEvaluationError::ProducedNaN,
        ))
      } else {
        Ok(res.into())
      }
    } else {
      Err(EvaluationErrorRepr::FunctionEvaluationError(
        self_func.ftype,
        FunctionEvaluationError::NoBase,
      ))
    }
  }
  decl_arg_types! {root, Number}
  decl_returns! {root, Number}

  pub fn mean(self_func: &Function, args: &[Object]) -> BuiltinFnOutput {
    let mut flts: Vec<f64> = vec![];

    for obj in args {
      flts.push(Object::try_into(*obj)?);
    }

    match helpers::mean(flts.as_slice()) {
      Ok(n) => Ok(n),
      Err(e) => Err(EvaluationErrorRepr::FunctionEvaluationError(
        self_func.ftype,
        e,
      )),
    }
  }
  decl_arg_types! {mean, vararg: Number}
  decl_returns! {mean, Number}

  pub fn gcf(_: &Function, args: &[Object]) -> BuiltinFnOutput {
    let mut res: f64 = args[0].try_into()?;

    for num in args {
      res = helpers::gcf(res, Object::try_into(*num)?);
      if res == 1f64 {
        return Ok(1f64.into());
      }
    }

    Ok(res.into())
  }
  decl_arg_types! {gcf, Number Number, vararg: Number}
  decl_returns! {gcf, Number}

  pub fn lcm(self_func: &Function, args: &[Object]) -> BuiltinFnOutput {
    let mut res: f64 = args[0].try_into()?;

    for num in args {
      res = match helpers::lcm(res, Object::try_into(*num)?) {
        Ok(n) => n,
        Err(e) => {
          return Err(EvaluationErrorRepr::FunctionEvaluationError(
            self_func.ftype,
            e,
          ));
        }
      };
      if res == 1f64 {
        return Ok(1f64.into());
      }
    }

    Ok(res.into())
  }
  decl_arg_types! {lcm, Number Number, vararg: Number}
  decl_returns! {lcm, Number}

  pub fn vec2d(_: &Function, args: &[Object]) -> BuiltinFnOutput {
    Ok(
      Vec2D {
        x: args[0].try_into()?,
        y: args[1].try_into()?,
      }
      .into(),
    )
  }
  decl_arg_types! {vec2d, Number Number}
  decl_returns! {vec2d, Vec2D}

  pub fn vec3d(_: &Function, args: &[Object]) -> BuiltinFnOutput {
    Ok(
      Vec3D {
        x: args[0].try_into()?,
        y: args[1].try_into()?,
        z: args[2].try_into()?,
      }
      .into(),
    )
  }
  decl_arg_types! {vec3d, Number Number Number}
  decl_returns! {vec3d, Vec3D}

  pub fn floor(_: &Function, args: &[Object]) -> BuiltinFnOutput {
    match args[0] {
      Object::Number(n) => Ok(n.floor().into()),
      Object::Vec2D(v) => Ok(v.floor().into()),
      Object::Vec3D(v) => Ok(v.floor().into()),
      _ => panic!("argument mismatch"),
    }
  }
  decl_arg_types! {floor, anyof: (Number, Vec2D, Vec3D)}
  decl_returns! {floor, anyof: Number, Vec2D, Vec3D}

  pub fn ceil(_: &Function, args: &[Object]) -> BuiltinFnOutput {
    match args[0] {
      Object::Number(n) => Ok(n.ceil().into()),
      Object::Vec2D(v) => Ok(v.ceil().into()),
      Object::Vec3D(v) => Ok(v.ceil().into()),
      _ => panic!("argument mismatch"),
    }
  }
  decl_arg_types! {ceil, anyof: (Number, Vec2D, Vec3D)}
  decl_returns! {ceil, anyof: Number, Vec2D, Vec3D}

  pub fn abs(_: &Function, args: &[Object]) -> BuiltinFnOutput {
    match args[0] {
      Object::Number(n) => Ok(n.abs().into()),
      Object::Vec2D(v) => Ok(v.abs().into()),
      Object::Vec3D(v) => Ok(v.abs().into()),
      _ => panic!("argument mismatch"),
    }
  }
  decl_arg_types! {abs, anyof: (Number, Vec2D, Vec3D)}
  decl_returns! {abs, anyof: Number, Vec2D, Vec3D}

  pub fn magnitude(self_func: &Function, args: &[Object]) -> BuiltinFnOutput {
    match args[0] {
      Object::Vec2D(vec2_d) => Ok(vec2_d.unit().into()),
      Object::Vec3D(vec3_d) => Ok(vec3_d.unit().into()),
      _ => Err(EvaluationErrorRepr::FunctionEvaluationError(
        self_func.ftype,
        FunctionEvaluationError::InvalidArgument {
          obj: args[0].kind(),
          expected: AnyOf(&[ObjectKind::Vec2D, ObjectKind::Vec3D]),
          idx: 0,
        },
      )),
    }
  }

  decl_arg_types! {magnitude, anyof: (Vec2D, Vec3D)}
  decl_returns! {magnitude, anyof: Vec2D, Vec3D}

  pub fn unit(self_func: &Function, args: &[Object]) -> BuiltinFnOutput {
    match args[0] {
      Object::Vec2D(vec2_d) => Ok(vec2_d.unit().into()),
      Object::Vec3D(vec3_d) => Ok(vec3_d.unit().into()),
      _ => Err(EvaluationErrorRepr::FunctionEvaluationError(
        self_func.ftype,
        FunctionEvaluationError::InvalidArgument {
          obj: args[0].kind(),
          expected: AnyOf(&[ObjectKind::Vec2D, ObjectKind::Vec3D]),
          idx: 0,
        },
      )),
    }
  }
  decl_arg_types! {unit, anyof: (Vec2D, Vec3D)}
  decl_returns! {unit, anyof: Vec2D, Vec3D}

  pub fn distance(self_func: &Function, args: &[Object]) -> BuiltinFnOutput {
    match (args[0], args[1]) {
      (Object::Vec2D(vec1), Object::Vec2D(vec2)) => Ok(vec1.distance(vec2).into()),
      (Object::Vec3D(vec1), Object::Vec3D(vec2)) => Ok(vec1.distance(vec2).into()),

      (_, Object::Vec2D(_) | Object::Vec3D(_)) => {
        Err(EvaluationErrorRepr::FunctionEvaluationError(
          self_func.ftype,
          FunctionEvaluationError::InvalidArgument {
            obj: args[0].kind(),
            expected: AnyOf(&[ObjectKind::Vec2D, ObjectKind::Vec3D]),
            idx: 0,
          },
        ))
      }

      (Object::Vec2D(_) | Object::Vec3D(_), _) | _ => {
        Err(EvaluationErrorRepr::FunctionEvaluationError(
          self_func.ftype,
          FunctionEvaluationError::InvalidArgument {
            obj: args[1].kind(),
            expected: AnyOf(&[ObjectKind::Vec2D, ObjectKind::Vec3D]),
            idx: 1,
          },
        ))
      }
    }
  }
  decl_arg_types! {distance, anyof: (Vec2D, Vec3D), anyof: (Vec2D, Vec3D)}
  decl_returns! {distance, anyof: Vec2D, Vec3D}

  pub fn project(self_func: &Function, args: &[Object]) -> BuiltinFnOutput {
    match (args[0], args[1]) {
      (Object::Vec2D(vec1), Object::Vec2D(vec2)) => Ok(vec1.project(vec2).into()),
      (Object::Vec3D(vec1), Object::Vec3D(vec2)) => Ok(vec1.project(vec2).into()),

      (_, Object::Vec2D(_) | Object::Vec3D(_)) => {
        Err(EvaluationErrorRepr::FunctionEvaluationError(
          self_func.ftype,
          FunctionEvaluationError::InvalidArgument {
            obj: args[0].kind(),
            expected: AnyOf(&[ObjectKind::Vec2D, ObjectKind::Vec3D]),
            idx: 0,
          },
        ))
      }

      (Object::Vec2D(_) | Object::Vec3D(_), _) | _ => {
        Err(EvaluationErrorRepr::FunctionEvaluationError(
          self_func.ftype,
          FunctionEvaluationError::InvalidArgument {
            obj: args[1].kind(),
            expected: AnyOf(&[ObjectKind::Vec2D, ObjectKind::Vec3D]),
            idx: 1,
          },
        ))
      }
    }
  }
  decl_arg_types! {project, anyof: (Vec2D, Vec3D), anyof: (Vec2D, Vec3D)}
  decl_returns! {project, anyof: Vec2D, Vec3D}

  pub fn dot(self_func: &Function, args: &[Object]) -> BuiltinFnOutput {
    match (args[0], args[1]) {
      (Object::Vec2D(vec1), Object::Vec2D(vec2)) => Ok(vec1.dot(vec2).into()),
      (Object::Vec3D(vec1), Object::Vec3D(vec2)) => Ok(vec1.dot(vec2).into()),

      (_, Object::Vec2D(_) | Object::Vec3D(_)) => {
        Err(EvaluationErrorRepr::FunctionEvaluationError(
          self_func.ftype,
          FunctionEvaluationError::InvalidArgument {
            obj: args[0].kind(),
            expected: AnyOf(&[ObjectKind::Vec2D, ObjectKind::Vec3D]),
            idx: 0,
          },
        ))
      }

      (Object::Vec2D(_) | Object::Vec3D(_), _) | _ => {
        Err(EvaluationErrorRepr::FunctionEvaluationError(
          self_func.ftype,
          FunctionEvaluationError::InvalidArgument {
            obj: args[1].kind(),
            expected: AnyOf(&[ObjectKind::Vec2D, ObjectKind::Vec3D]),
            idx: 1,
          },
        ))
      }
    }
  }
  decl_arg_types! {dot, anyof: (Vec2D, Vec3D), anyof: (Vec2D, Vec3D)}
  decl_returns! {dot, Number}

  pub fn angle(_: &Function, args: &[Object]) -> BuiltinFnOutput {
    let as_vec1: Vec2D = args[0].try_into()?;
    let as_vec2: Vec2D = args[1].try_into()?;
    Ok(as_vec1.angle(as_vec2).into())
  }
  decl_arg_types! {angle, Vec2D Vec2D}
  decl_returns! {angle, Vec2D}

  // boo hoo its not snake case
  pub fn signedangle(_: &Function, args: &[Object]) -> BuiltinFnOutput {
    let as_vec1: Vec2D = args[0].try_into()?;
    let as_vec2: Vec2D = args[1].try_into()?;
    Ok(as_vec1.signed_angle_2d(as_vec2).into())
  }
  decl_arg_types! {signedangle, Vec2D Vec2D}
  decl_returns! {signedangle, Vec2D}

  #[allow(dead_code)]
  pub fn cross(self_func: &Function, args: &[Object]) -> BuiltinFnOutput {
    match (args[0], args[1]) {
      (Object::Vec2D(vec1), Object::Vec2D(vec2)) => Ok(vec1.cross(vec2).into()),
      (Object::Vec3D(vec1), Object::Vec3D(vec2)) => Ok(vec1.cross(vec2).into()),

      (_, Object::Vec2D(_) | Object::Vec3D(_)) => {
        Err(EvaluationErrorRepr::FunctionEvaluationError(
          self_func.ftype,
          FunctionEvaluationError::InvalidArgument {
            obj: args[0].kind(),
            expected: AnyOf(&[ObjectKind::Vec2D, ObjectKind::Vec3D]),
            idx: 0,
          },
        ))
      }

      (Object::Vec2D(_) | Object::Vec3D(_), _) | _ => {
        Err(EvaluationErrorRepr::FunctionEvaluationError(
          self_func.ftype,
          FunctionEvaluationError::InvalidArgument {
            obj: args[1].kind(),
            expected: AnyOf(&[ObjectKind::Vec2D, ObjectKind::Vec3D]),
            idx: 1,
          },
        ))
      }
    }
  }
  decl_arg_types! {cross, anyof: (Vec2D, Vec3D), anyof: (Vec2D, Vec3D)}
  decl_returns! {cross, anyof: Vec2D, Vec3D}

  pub fn out(_: &Function, args: &[Object]) -> BuiltinFnOutput {
    println!("{}", args[0]);
    Ok(args[0])
  }
  decl_arg_types! {out, Any}
  decl_returns! {out, Any}

  pub fn binout(_: &Function, args: &[Object]) -> BuiltinFnOutput {
    match args[0] {
      Object::Number(n) => {
        println!("{:08b}", n as i64);
        Ok(args[0])
      }
      _ => panic!("argument mismatch"),
    }
  }
  decl_arg_types! {binout, Number}
  decl_returns! {binout, Number}
}

mod helpers {
  use std::ops::Div;

  use crate::{functions::FunctionEvaluationError, types::object::Object};

  pub fn gcf(a: f64, b: f64) -> f64 {
    if b == 0f64 { a } else { gcf(b, a % b) }
  }

  pub fn lcm(a: f64, b: f64) -> Result<f64, FunctionEvaluationError> {
    if a == 0f64 || b == 0f64 {
      return Ok(0f64.into());
    }

    let ab = a * b;
    if !ab.is_infinite() {
      return match gcf(a, b).try_into().expect("unreachable") {
        0f64 => Err(FunctionEvaluationError::DivisionByZero),
        g_res => Ok((ab / g_res).into()),
      };
    }

    Err(FunctionEvaluationError::NumberOverflow)
  }

  pub fn mean(args: &[f64]) -> Result<Object, FunctionEvaluationError> {
    let len = args.len();
    if len == 0 {
      return Err(FunctionEvaluationError::DivisionByZero);
    }

    let sum: f64 = args.iter().sum();
    if sum.is_infinite() {
      Err(FunctionEvaluationError::NumberOverflow)
    } else {
      Ok(sum.div(len as f64).into())
    }
  }

  // TODO: add mean test cases
  #[cfg(test)]
  mod tests {
    use crate::functions::helpers::{gcf, lcm};

    #[test]
    fn test_gcf() {
      assert_eq!(gcf(6f64, 3f64), 3f64.into());
      assert_eq!(gcf(99f64, 2f64), 1f64.into());
      assert_eq!(gcf(1f64, 1f64), 1f64.into());
      assert_eq!(gcf(6f64, 0f64), 6f64.into());
      assert_eq!(gcf(0f64, 24f64), 24f64.into());
      assert_eq!(gcf(2934f64, 24f64), 6f64.into());
    }

    #[test]
    fn test_lcm() {
      assert_eq!(lcm(12f64, 18f64).unwrap(), 36f64.into());
      assert_eq!(lcm(82f64, 4f64).unwrap(), 164f64.into());
      assert_eq!(lcm(3f64, 35f64).unwrap(), 105f64.into());
      assert_eq!(lcm(0f64, 18f64).unwrap(), 0f64.into());
      assert_eq!(lcm(18f64, 0f64).unwrap(), 0f64.into());
    }
  }
}
