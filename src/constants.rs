//! Enum representation of supported mathematical constants within the parser.
//! Atleast for now, constants will always be converted to numerical values whenever
//! they are found in the evaluation step, meaning no exact values. The main purpose
//! is to instead just serve as a QOL feature so as to not be required to type out
//! constants via primitive numbers all the time.  

use std::f64;

/// All currently supported constants. 
#[derive(Debug, PartialEq, Clone, Copy, PartialOrd)]
pub enum Constant {
  Pi,
  Euler,
  Tau,
  GoldenRatio,
}

impl Constant {
  /// Turns the enum into a string for display purposes, should
  /// only display constants in their symbol form, not phonetic representation.
  pub(crate) fn as_str(&self) -> &str {
    match self {
      Constant::Pi => "π",
      Constant::Euler => "e",
      Constant::Tau => "τ",
      Constant::GoldenRatio => "φ",
    }
  }

  /// Mainly uses the builtin constants to get the values of 
  /// their respective mathematical constants.
  pub fn get_value(&self) -> f64 {
    match self {
      Constant::Pi => f64::consts::PI,
      Constant::Euler => f64::consts::E,
      Constant::Tau => f64::consts::TAU,
      Constant::GoldenRatio => f64::consts::GOLDEN_RATIO,
    }
  }

  /// Constructs a [Constant] from a string representation.
  /// Returns None if the input does not correspond to a known constant.
  pub fn from_string(st: &String) -> Option<Self> {
    let lower = st.to_lowercase();

    match lower.as_str() {
      "pi" | "π" => Some(Self::Pi),
      "tau" | "τ" => Some(Self::Tau),
      "e" => Some(Self::Euler),
      // idk if "phi" should count, however i dont know what else it would
      // match to in this context so we can go with it
      "phi" | "Φ" => Some(Self::GoldenRatio),
      &_ => None,
    }
  }
}
