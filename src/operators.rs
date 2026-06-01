//! Module including the [Operation] enum and some implementation functions.

use std::fmt::Display;

/// Enum representation of a single character operator
/// like '+'.
#[derive(Debug, PartialEq, Clone, Copy, Eq, PartialOrd, Ord)]
pub enum Operation {
  Add,
  Sub,
  Mul,
  Div,
  Exp,
  Mod,
  Fac,
}

impl Operation {
  pub fn as_str(&self) -> &str {
    match self {
      Self::Add => "+",
      Self::Sub => "-",
      Self::Mul => "*",
      Self::Div => "/",
      Self::Exp => "^",
      Self::Mod => "%",
      Self::Fac => "!",
    }
  }

  /// Matches unicode characters (not graphemes) to their enum
  /// representation. Mainly used in tokenization.
  pub(crate) fn from_char(chr: char) -> Option<Self> {
    match chr {
      '+' => Some(Operation::Add),
      // yo this is so 🤣🤣🤣🤣🤣🤣🤣🤣
      '-' | '–' | '—' | '‒' => Some(Operation::Sub),
      '*' | '×' => Some(Operation::Mul),
      '/' | '÷' => Some(Operation::Div),
      '^' => Some(Operation::Exp),
      '%' => Some(Operation::Mod),
      '!' => Some(Operation::Fac),
      _ => None,
    }
  }

  /// Gets the binding power of the operator if it can be used
  /// as an infix operator, returns None otherwise.
  /// Higher values mean a tighter binding and therefore increased
  /// precedence above other operators.
  pub fn get_infix_bp(&self) -> Option<(u8, u8)> {
    match self {
      Self::Add | Self::Sub => Some((1, 2)),
      Self::Mul | Self::Div | Self::Mod => Some((5, 6)),
      Self::Exp => Some((10, 11)),
      _ => None,
    }
  }

  /// Gets the binding power of the operator if it can be used
  /// as a prefix (unary) operator, returns None otherwise.
  pub fn get_prefix_bp(&self) -> Option<u8> {
    match self {
      Self::Sub => Some(u8::MAX),
      _ => None,
    }
  }

  /// Gets the binding power of the operator if it can be used
  /// as a postfix (unary) operator, returns None otherwise.
  pub fn get_postfix_bp(&self) -> Option<u8> {
    match self {
      Self::Fac => Some(10),
      _ => None,
    }
  }
}

impl Display for Operation {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self.as_str())
  }
}
