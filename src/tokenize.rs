//! Contains all the logic for tokenizing an input string for later parsing.
//! See the main struct [Token] to see how to get started with tokenization.

// This was the first file I wrote for this and I feel its very messy compared
// to the other stuff

use thiserror::Error;

use crate::ast::ParseError;
use crate::constants::Constant;
use crate::functions::Function;
use crate::functions::FunctionType;
use crate::operators::Operation;
use crate::tokenize::util::seek_next_non_whitespace_char;

use std::cell::Cell;
use std::fmt::Display;
use std::{
  f64::{self},
  vec,
};

/// The final result of a tokenized string ready to be parsed.
/// Basicly just a Vec<Token>.
///
/// See the constructor [Tokens::new] on how to construct a tokenized string.
#[derive(Debug, Clone, PartialEq)]
pub struct Tokens {
  tokens: Vec<Token>,
  _pos: Cell<usize>,
}

impl Tokens {
  /// Attempts to construct a list of tokens based off of the input string.
  ///
  /// Tokenization by itself will also conduct some checks on the validity of
  /// the string, mostly for function calls. See [TokenizeErrorType] for
  /// a general idea of what tokenization will check for.
  pub fn new(input_str: &str) -> Result<Self, TokenizeError> {
    let result = tokenize(input_str)?;
    Ok(Self {
      tokens: result,
      _pos: Cell::new(0),
    })
  }

  // why does the lsp think this way
  #[allow(dead_code)]
  fn from_existing_vec(tokens: Vec<Token>) -> Self {
    Self {
      tokens,
      _pos: Cell::new(0),
    }
  }

  pub fn token_ref(&self) -> &Vec<Token> {
    &self.tokens
  }

  pub(crate) fn next(&self) -> Token {
    let tok = self
      .tokens
      .get(self._pos.get())
      .cloned()
      .unwrap_or(Token::Eof);
    self._pos.set(self._pos.get() + 1);
    tok
  }

  pub(crate) fn peek(&self) -> Token {
    self
      .tokens
      .get(self._pos.get())
      .cloned()
      .unwrap_or(Token::Eof)
  }

  pub(crate) fn expect(&self, other: Token) -> Result<Token, ParseError> {
    let next = self.next();

    if next != other {
      Err(ParseError::UnexpectedToken {
        tok: next,
        expected: other,
      })
    } else {
      Ok(next)
    }
  }
}

impl Display for Tokens {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    for token in self.tokens.iter() {
      #[allow(unused_must_use)]
      match token {
        Token::Number(num) => write!(f, "{}", num.to_string()),
        Token::Const(constant) => write!(f, "{}", constant.as_str()),
        Token::Operator(operation) => write!(f, "{}", operation.as_str()),
        Token::Function(function) => {
          if function.has_base() {
            write!(
              f,
              "{}_{}",
              function.get_function_type().as_str(),
              function.get_base_unwrap(),
            )
          } else {
            write!(f, "{}", function.get_function_type().as_str(),)
          }
        }
        Token::OpenBracket => {
          write!(f, "(")
        }
        Token::CloseBracket => {
          write!(f, ")")
        }
        Token::Eof => {
          continue;
        }
        Token::Comma => write!(f, ","),
      };
    }

    Ok(())
  }
}

/// Enum describing all the possible tokens the tokenizer can interpret.
#[derive(Debug, PartialEq, Clone, Copy, PartialOrd)]
pub enum Token {
  /// Any primitive number, represented as a float.
  Number(f64),
  /// Mathematical constants like 'e' and 'pi'.
  Const(Constant),
  /// Single character operators like '+' and '^'
  Operator(Operation),
  /// A function call, like 'cos()'. Some functions also
  /// support bases, like the root_n() function which will
  /// take the n'th root of a number.
  Function(Function),
  /// The '(' or '[' characters
  OpenBracket,
  /// The ')' or ']' characters
  CloseBracket,
  /// Self explanatory
  Comma,
  /// Internally used when the end of the token sequence is reached.
  Eof,
}

/// All the possible errors that can happen during tokenization.
#[derive(Debug, PartialEq, Clone, PartialOrd, Error)]
pub enum TokenizeErrorType {
  /// Thrown when the tokenizer finds any alphanumeric substring that
  /// does not correspond to a [Constant] or [FunctionType].
  #[error("could not resolve symbol {0} to a constant or function")]
  InvalidSymbol(String),
  /// Generally thrown on functions with impropertly written bases.
  #[error("found function symbol {0} but has malformed base or open/close brackets")]
  MalformedFunction(String),
  /// Thrown when there is an error with parsing a primitive number
  /// from a float to string.
  #[error("tokenizer failed to parse number")]
  NumberParseError,
  /// Thrown when the input string is empty.
  #[error("input string is empty")]
  EmptyString,
  /// Thrown when a provided function base is not made of a singular
  /// primitive number (unsupported for now, might implement later)
  #[error("function {0} has a non-numerical or non-integer base")]
  NonNumericalBase(FunctionType),
  /// Thrown when assigning a base to a function that does not take
  /// bases. See [FunctionType::supports_base] to see what functions
  /// take bases.
  #[error("function {0} does not support bases")]
  FunctionDoesNotSupportBases(FunctionType),

  /// Thrown when trying to use the rand function without the rand
  /// feature enabled.
  #[error("found rand function but mparse was not compiled with the rand feature enabled")]
  RandNotSupported,
}

/// Encapsulating struct for tokenization errors so that we can
/// pass an index.
#[derive(Debug, PartialEq, Clone, PartialOrd, Error)]
pub struct TokenizeError {
  err_type: TokenizeErrorType,
  index: usize,
}

impl TokenizeError {
  pub fn new(err_type: TokenizeErrorType, idx: usize) -> Self {
    Self {
      err_type,
      index: idx,
    }
  }

  pub fn get_index(&self) -> usize {
    self.index
  }

  pub fn set_index(&mut self, idx: usize) {
    self.index = idx
  }

  pub fn err_type(&self) -> &TokenizeErrorType {
    &self.err_type
  }
}

impl Display for TokenizeError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(
      f,
      "error during tokenization at index {}: {}",
      self.index, self.err_type
    )
  }
}

mod util {
  use std::num::ParseFloatError;

  use crate::{
    functions::FunctionType,
    tokenize::{
      Constant, Token, TokenizeError,
      TokenizeErrorType::{self},
    },
  };

  // Seeks to the end of a number, exclusive.
  pub fn seek_end_of_num(chars: &Vec<char>, mut idx: usize) -> usize {
    loop {
      if chars.len() == idx {
        break;
      }

      let chr = chars[idx];

      if !is_ascii_numeric(chr) && chr != '.' {
        break;
      }

      idx += 1;
    }

    idx
  }

  pub fn parse_number(
    chars: &Vec<char>,
    start_idx: usize,
  ) -> Result<(f64, usize), ParseFloatError> {
    let end_idx = seek_end_of_num(chars, start_idx);
    let slice = &chars[start_idx..end_idx];
    let as_string = slice.iter().collect::<String>();
    let res = as_string.parse::<f64>()?;

    Ok((res, end_idx))
  }

  pub fn seek_word(chars: &Vec<char>, start_idx: usize) -> (String, usize) {
    let mut s = String::with_capacity(4);
    let mut current_idx = start_idx;

    loop {
      if chars.len() == current_idx {
        break;
      }

      let chr = chars[current_idx];
      if chr.is_ascii_alphabetic() {
        s.push(chr as char);
      } else {
        break;
      }

      current_idx += 1;
    }

    (s, current_idx)
  }

  pub fn seek_char_predicate(
    chars: &Vec<char>,
    pred: impl Fn(char) -> bool,
    start_idx: usize,
  ) -> Option<usize> {
    let mut current_idx = start_idx;
    loop {
      if chars.len() == current_idx {
        break;
      }

      let chr = chars[current_idx];
      if pred(chr) {
        return Some(current_idx);
      }

      current_idx += 1;
    }

    None
  }

  pub fn seek_next_non_whitespace_char(chars: &Vec<char>, start_idx: usize) -> Option<usize> {
    return seek_char_predicate(chars, |chr: char| !chr.is_whitespace(), start_idx);
  }

  pub fn is_ascii_numeric(chr: char) -> bool {
    chr >= '0' && chr <= '9'
  }

  pub fn seek_first_valid_const(word: &[u8]) -> (Option<Constant>, usize) {
    let mut input = String::new();
    let mut idx = 0;
    for ele in word {
      let chr = *ele as char;
      input.push(chr);

      if let Some(con) = Constant::from_string(&input) {
        return (Some(con), idx);
      }
      idx += 1;
    }

    (None, idx)
  }

  pub fn extract_consts_from_word(word: &[u8]) -> (Vec<Constant>, usize) {
    let mut found_consts: Vec<Constant> = vec![];
    let mut c_idx = 0;
    loop {
      let (c_opt, const_end_idx_relative) = seek_first_valid_const(&word[c_idx..]);
      if let Some(found_const) = c_opt {
        found_consts.push(found_const);
        c_idx = const_end_idx_relative;
      } else {
        break;
      }
    }

    (found_consts, c_idx)
  }

  pub fn extract_base(
    chars: &Vec<char>,
    ftype: FunctionType,
    word: &String,
    idx: usize,
    end_idx: usize,
  ) -> Result<(f64, usize), TokenizeError> {
    // if a function is going to have a base there should be atleast two more characters
    if chars.len() <= end_idx + 3 {
      return Err(TokenizeError::new(
        TokenizeErrorType::MalformedFunction(word.clone()),
        idx,
      ));
    }

    let base: f64;
    let mut flag = false;
    let (char1, char2) = (chars[end_idx + 1], chars[end_idx + 2]);
    let num_parse_idx: usize;
    let char_to_test: char;

    if char1 == '-' {
      flag = true;
      char_to_test = char2;
      num_parse_idx = end_idx + 2;
    } else {
      char_to_test = char1;
      num_parse_idx = end_idx + 1
    }

    if !is_ascii_numeric(char_to_test as char) {
      return Err(TokenizeError::new(
        TokenizeErrorType::NonNumericalBase(ftype),
        idx,
      ));
    }

    if let Ok((num, num_end_idx)) = parse_number(chars, num_parse_idx) {
      let next_bracket_idx_opt = seek_char_predicate(
        chars,
        |chr: char| char_is_bracket(chr).is_some(),
        num_end_idx,
      );
      let next_char_idx_opt = seek_next_non_whitespace_char(chars, num_end_idx);

      if next_char_idx_opt.is_some() && next_bracket_idx_opt.is_some() {
        if next_char_idx_opt.expect("unreachable") != next_bracket_idx_opt.expect("unreachable") {
          return Err(TokenizeError::new(
            TokenizeErrorType::NonNumericalBase(ftype),
            idx,
          ));
        }
      } else {
        return Err(TokenizeError::new(
          TokenizeErrorType::MalformedFunction(word.clone()),
          idx,
        ));
      }

      if flag {
        base = -num
      } else {
        base = num;
      }

      return Ok((base, num_end_idx));
    } else {
      return Err(TokenizeError::new(
        TokenizeErrorType::NumberParseError,
        num_parse_idx,
      ));
    }
  }

  #[derive(Debug, PartialEq)]
  pub enum BracketType {
    Opening,
    Closing,
  }

  impl BracketType {
    pub fn to_token(&self) -> Token {
      match self {
        Self::Opening => Token::OpenBracket,
        Self::Closing => Token::CloseBracket,
      }
    }
  }

  pub fn char_is_bracket(chr: char) -> Option<BracketType> {
    match chr {
      '(' | '[' => Some(BracketType::Opening),
      ')' | ']' => Some(BracketType::Closing),
      _ => None,
    }
  }
}

/// The primary function for tokenization.
fn tokenize_part(
  chars: &Vec<char>,
  start_idx: usize,
  end_idx: usize,
) -> Result<Vec<Token>, TokenizeError> {
  let mut idx = start_idx;
  let mut tokens = vec![];

  while idx < end_idx && idx < chars.len() {
    let chr = chars[idx];

    match chr {
      _ if chr.is_ascii_digit() => {
        if let Ok((num, end_num_idx)) = util::parse_number(chars, idx) {
          tokens.push(Token::Number(num));
          idx = end_num_idx;
          continue;
        } else {
          return Err(TokenizeError::new(TokenizeErrorType::NumberParseError, idx));
        }
      }

      _ if let Some(op) = Operation::from_char(chr) => {
        tokens.push(Token::Operator(op));
        idx += 1;
        continue;
      }

      _ if chr.is_alphabetic() => {
        let (word, end_word_idx) = util::seek_word(chars, idx);

        if let Some(constant) = Constant::from_string(&word) {
          tokens.push(Token::Const(constant));
          idx = end_word_idx;
          continue;
        } else {
          let (found_consts, c_idx) = util::extract_consts_from_word(word.as_bytes());

          if !found_consts.is_empty() {
            for ele in found_consts {
              tokens.push(Token::Const(ele));
            }
            idx += c_idx + 1;
            continue;
          } else {
            if let Some(ftype) = FunctionType::from_string(&word) {
              #[cfg(not(feature = "rand"))]
              if ftype == FunctionType::Rand {
                return Err(TokenizeError::new(TokenizeErrorType::RandNotSupported, idx));
              }

              let mut start_seek_idx = end_word_idx;
              let mut base: Option<f64> = None;
              if chars[end_word_idx] == '_' {
                if !ftype.supports_base() {
                  return Err(TokenizeError::new(
                    TokenizeErrorType::FunctionDoesNotSupportBases(ftype),
                    idx,
                  ));
                }

                let (base_res, base_end_idx) =
                  util::extract_base(chars, ftype.clone(), &word, idx, end_word_idx)?;
                base = Some(base_res);

                start_seek_idx = base_end_idx;
              }

              if let Some(next_char_idx) = seek_next_non_whitespace_char(chars, start_seek_idx) {
                if util::char_is_bracket(chars[next_char_idx]).is_none() {
                  return Err(TokenizeError::new(
                    TokenizeErrorType::MalformedFunction(word.clone()),
                    idx,
                  ));
                }
              } else {
                return Err(TokenizeError::new(
                  TokenizeErrorType::MalformedFunction(word.clone()),
                  idx,
                ));
              }

              let func = Function::new(ftype, base);
              tokens.push(Token::Function(func));
              idx = start_seek_idx;
              continue;
            } else {
              return Err(TokenizeError::new(
                TokenizeErrorType::InvalidSymbol(word),
                idx,
              ));
            };
          }
        }
      }

      _ if let Some(bracket_type) = util::char_is_bracket(chr) => {
        tokens.push(bracket_type.to_token());
      }

      ',' => {
        tokens.push(Token::Comma);
      }

      _ => {}
    }

    idx += 1;
  }

  Ok(tokens)
}

/// Entry point for tokenization.
fn tokenize(input_str: &str) -> Result<Vec<Token>, TokenizeError> {
  let trim = input_str.trim();

  if trim.is_empty() {
    return Err(TokenizeError::new(TokenizeErrorType::EmptyString, 0));
  }

  let chars: Vec<char> = trim.chars().collect();
  let parsed = tokenize_part(&chars, 0, chars.len())?;

  Ok(parsed)
}

// TODO: finish test cases
#[cfg(test)]
mod tests {
  use crate::tokenize::Tokens;

  use super::{Constant, Function, FunctionType, Operation, Token, TokenizeErrorType, tokenize};

  fn print_tokens(tokens: &Tokens) {
    println!("{} ", tokens);
  }

  fn compare_input_to_tokens(input: &str, expected: Vec<Token>) -> bool {
    let parsed = Tokens::new(input).expect("parse error in test");
    let expected_as_tokens = Tokens::from_existing_vec(expected);
    let res = expected_as_tokens == parsed;

    if !res {
      print!("Expected tokens: ");
      print_tokens(&expected_as_tokens);
      print!("Parsed tokens: ");
      print_tokens(&parsed);
    }

    res
  }

  #[test]
  fn test_basic_ops() {
    let input = "12*5 + 8/2 - 12";
    let expected = vec![
      Token::Number(12f64),
      Token::Operator(Operation::Mul),
      Token::Number(5f64),
      Token::Operator(Operation::Add),
      Token::Number(8f64),
      Token::Operator(Operation::Div),
      Token::Number(2f64),
      Token::Operator(Operation::Sub),
      Token::Number(12f64),
    ];

    assert!(compare_input_to_tokens(input, expected))
  }

  #[test]
  fn test_invalid_function() {
    let input = "84 / sqrt(12) + 3 - lig(3 + 3)";
    let parsed = tokenize(input);

    assert!(parsed.is_err());
    let perr = parsed.expect_err("unreachable");
    assert!(matches!(
      perr.err_type(),
      TokenizeErrorType::InvalidSymbol(..)
    ));
  }

  #[test]
  fn test_malformed_function() {
    let input = "1 / log(100) + log 3 + 3)";
    let parsed = tokenize(input);

    assert!(parsed.is_err());

    let perr = parsed.expect_err("unreachable");
    assert!(matches!(
      perr.err_type(),
      TokenizeErrorType::MalformedFunction(..)
    ));
  }

  #[test]
  fn test_basic_functions() {
    let input = "18 * sqrt(12/2) + ln(8) - sin(45 + 30)";
    let expected = vec![
      Token::Number(18f64),
      Token::Operator(Operation::Mul),
      Token::Function(Function::with_no_base(FunctionType::Sqrt)),
      Token::OpenBracket,
      Token::Number(12f64),
      Token::Operator(Operation::Div),
      Token::Number(2f64),
      Token::CloseBracket,
      Token::Operator(Operation::Add),
      Token::Function(Function::with_no_base(FunctionType::Ln)),
      Token::OpenBracket,
      Token::Number(8f64),
      Token::CloseBracket,
      Token::Operator(Operation::Sub),
      Token::Function(Function::with_no_base(FunctionType::Sin)),
      Token::OpenBracket,
      Token::Number(45f64),
      Token::Operator(Operation::Add),
      Token::Number(30f64),
      Token::CloseBracket,
    ];

    assert!(compare_input_to_tokens(input, expected))
  }

  #[test]
  fn test_function_with_cases() {
    let input = "Sqrt(12+2) + CBRT(60) - sIn(20)";
    let expected = vec![
      Token::Function(Function::with_no_base(FunctionType::Sqrt)),
      Token::OpenBracket,
      Token::Number(12f64),
      Token::Operator(Operation::Add),
      Token::Number(2f64),
      Token::CloseBracket,
      Token::Operator(Operation::Add),
      Token::Function(Function::with_no_base(FunctionType::Cbrt)),
      Token::OpenBracket,
      Token::Number(60f64),
      Token::CloseBracket,
      Token::Operator(Operation::Sub),
      Token::Function(Function::with_no_base(FunctionType::Sin)),
      Token::OpenBracket,
      Token::Number(20f64),
      Token::CloseBracket,
    ];

    assert!(compare_input_to_tokens(input, expected))
  }

  #[test]
  fn test_nested_function() {
    let input = "log(sqrt(2/3) + 3)";
    let expected = vec![
      Token::Function(Function::with_no_base(FunctionType::Log)),
      Token::OpenBracket,
      Token::Function(Function::with_no_base(FunctionType::Sqrt)),
      Token::OpenBracket,
      Token::Number(2f64),
      Token::Operator(Operation::Div),
      Token::Number(3f64),
      Token::CloseBracket,
      Token::Operator(Operation::Add),
      Token::Number(3f64),
      Token::CloseBracket,
    ];

    assert!(compare_input_to_tokens(input, expected))
  }

  #[test]
  fn test_basic_consts() {
    let input = "pi/12 + pi/6 * e - 5";
    let expected = vec![
      Token::Const(Constant::Pi),
      Token::Operator(Operation::Div),
      Token::Number(12f64),
      Token::Operator(Operation::Add),
      Token::Const(Constant::Pi),
      Token::Operator(Operation::Div),
      Token::Number(6f64),
      Token::Operator(Operation::Mul),
      Token::Const(Constant::Euler),
      Token::Operator(Operation::Sub),
      Token::Number(5f64),
    ];

    assert!(compare_input_to_tokens(input, expected))
  }

  #[test]
  fn test_group() {
    let input = "(5^2) + (8/2 + 2)";
    let expected = vec![
      Token::OpenBracket,
      Token::Number(5f64),
      Token::Operator(Operation::Exp),
      Token::Number(2f64),
      Token::CloseBracket,
      Token::Operator(Operation::Add),
      Token::OpenBracket,
      Token::Number(8f64),
      Token::Operator(Operation::Div),
      Token::Number(2f64),
      Token::Operator(Operation::Add),
      Token::Number(2f64),
      Token::CloseBracket,
    ];

    assert!(compare_input_to_tokens(input, expected))
  }

  #[test]
  fn test_nested_group() {
    let input = "8/(5^2 + (5+ 5))";
    let expected = vec![
      Token::Number(8f64),
      Token::Operator(Operation::Div),
      Token::OpenBracket,
      Token::Number(5f64),
      Token::Operator(Operation::Exp),
      Token::Number(2f64),
      Token::Operator(Operation::Add),
      Token::OpenBracket,
      Token::Number(5f64),
      Token::Operator(Operation::Add),
      Token::Number(5f64),
      Token::CloseBracket,
      Token::CloseBracket,
    ];

    assert!(compare_input_to_tokens(input, expected))
  }

  #[test]
  fn test_err_combine_const_and_function() {
    // when i write sine i mean sin * e, we should not interpret sine as sin anyways for this context
    let input = "2sine(8)";
    let parsed = tokenize(input);

    assert!(parsed.is_err());
    let perr = parsed.expect_err("unreachable");
    assert!(matches!(
      perr.err_type(),
      TokenizeErrorType::InvalidSymbol(..)
    ));
  }

  #[test]
  fn test_non_numerical_base() {
    let input = "4log_5pi(12)";
    let parsed = tokenize(input);

    assert!(parsed.is_err());
    let perr = parsed.expect_err("unreachable");
    assert!(matches!(
      perr.err_type(),
      TokenizeErrorType::NonNumericalBase(..)
    ));
  }

  #[test]
  fn test_unsupported_base() {
    let input = "sin_10(2)";
    let parsed = tokenize(input);

    assert!(parsed.is_err());
    let perr = parsed.expect_err("unreachable");
    assert!(matches!(
      perr.err_type(),
      TokenizeErrorType::FunctionDoesNotSupportBases(..)
    ));
  }

  #[test]
  fn test_basic_base() {
    let input = "10root_4(256)+3";
    let expected = vec![
      Token::Number(10f64),
      Token::Function(Function::new(FunctionType::Root, Some(4f64))),
      Token::OpenBracket,
      Token::Number(256f64),
      Token::CloseBracket,
      Token::Operator(Operation::Add),
      Token::Number(3f64),
    ];

    assert!(compare_input_to_tokens(input, expected))
  }

  #[test]
  fn test_negative_base() {
    let input = "10root_-4(256)+3";
    let expected = vec![
      Token::Number(10f64),
      Token::Function(Function::new(FunctionType::Root, Some(-4f64))),
      Token::OpenBracket,
      Token::Number(256f64),
      Token::CloseBracket,
      Token::Operator(Operation::Add),
      Token::Number(3f64),
    ];

    assert!(compare_input_to_tokens(input, expected))
  }

  #[cfg(not(feature = "rand"))]
  #[test]
  fn test_unsupported_rand() {
    let input = "rand()";
    let parsed = tokenize(input);

    assert!(parsed.is_err());
    assert!(matches!(
      parsed.expect_err("unreachable").err_type(),
      TokenizeErrorType::RandNotSupported
    ))
  }
}
