//! Module including the [Operation] enum and some implementation functions.

use std::fmt::Display;

#[derive(Debug, PartialEq, Clone, Copy, Eq, PartialOrd, Ord)]
pub enum Operation {
  Add,
  Sub,
  Mul,
  Div,
  Exp,
  Rem,
  Fac,
  Dot,
  Eq,
  Neq,
  And,
  Or,
}

impl Operation {
  pub fn as_str(&self) -> &str {
    match self {
      Self::Add => "+",
      Self::Sub => "-",
      Self::Mul => "*",
      Self::Div => "/",
      Self::Exp => "^",
      Self::Rem => "%",
      Self::Fac => "!",
      Self::Dot => ".",
      Self::And => "&",
      Self::Or => "|",
      Self::Eq => "==",
      Self::Neq => "!=",
    }
  }

  /// Matches the single character operators to their enum representation.
  /// Mainly used in tokenization.
  pub(crate) fn from_char(s: char) -> Option<Self> {
    match s {
      '+' => Some(Operation::Add),
      // yo this is so 🤣🤣🤣🤣🤣🤣🤣🤣
      '-' | '–' | '—' | '‒' => Some(Operation::Sub),
      '*' | '×' => Some(Operation::Mul),
      '/' | '÷' => Some(Operation::Div),
      '^' => Some(Operation::Exp),
      '%' => Some(Operation::Rem),
      '!' => Some(Operation::Fac),
      '.' => Some(Operation::Dot),
      '|' => Some(Operation::Or),
      '&' => Some(Operation::And),
      _ => None,
    }
  }

  /// Gets the binding power of the operator if it can be used
  /// as an infix operator, returns None otherwise.
  /// Higher values mean a tighter binding and therefore increased
  /// precedence above other operators.
  pub fn get_infix_bp(&self) -> Option<(u8, u8)> {
    match self {
      Self::Eq | Self::Neq => Some((1, 2)),
      Self::Add | Self::Sub => Some((3, 2)),
      Self::Mul | Self::Div | Self::Rem => Some((5, 6)),
      Self::Exp => Some((10, 11)),
      Self::And | Self::Or => Some((12, 13)),
      Self::Dot => Some((u8::MAX - 1, u8::MAX)),
      _ => None,
    }
  }

  /// Gets the binding power of the operator if it can be used
  /// as a prefix (unary) operator, returns None otherwise.
  pub fn get_prefix_bp(&self) -> Option<u8> {
    match self {
      Self::Sub | Self::Fac => Some(u8::MAX),
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
