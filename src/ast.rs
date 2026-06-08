use std::fmt::Display;

use thiserror::Error;

use crate::{
  constants::Constant,
  functions::Function,
  operators::Operation,
  tokenize::{Token, Tokens},
};

#[derive(Debug, Error, PartialEq, PartialOrd)]
pub enum ParseErrorRepr {
  #[error("unexpected token while parsing, expected {expected:?}, found {tok:?}")]
  UnexpectedToken { tok: Token, expected: Token },

  #[error("token {second:?} cannot come after token {first:?}")]
  InvalidTokenSequence { first: Token, second: Token },

  #[error("unexpected operator {0}")]
  UnexpectedOperator(Operation),
}

impl ParseErrorRepr {
  pub fn make_unexpected(tok: Token, expected: Token) -> Self {
    ParseErrorRepr::UnexpectedToken { tok, expected }
  }

  pub fn make_invalid_seq(first: Token, second: Token) -> Self {
    ParseErrorRepr::InvalidTokenSequence { first, second }
  }
}

#[derive(Debug, PartialEq)]
pub enum Expression {
  Number(f64),
  Constant(Constant),

  Unary {
    op: Operation,
    expr: Box<Expression>,
  },

  Binary {
    op: Operation,
    left: Box<Expression>,
    right: Box<Expression>,
  },

  Function {
    func: Function,
    exprs: Vec<Expression>,
  },
}

impl Expression {
  pub fn new(tokens: &Tokens) -> Result<Expression, ParseErrorRepr> {
    expr(tokens, 0)
  }
}

impl Display for Expression {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Expression::Number(n) => write!(f, "{}", n),
      Expression::Constant(constant) => write!(f, "{}", constant.as_str()),
      Expression::Unary { op, expr } => write!(f, "{}({})", op.as_str(), expr),
      Expression::Binary { op, left, right } => write!(f, "({} {} {})", left, op.as_str(), right),
      Expression::Function { func, exprs } => {
        let ftype = func.get_function_type();
        write!(f, "{}", ftype.as_str())?;

        if func.has_base() && ftype.supports_base() {
          write!(f, "_{}", func.get_base_unwrap())?;
        }

        write!(f, "(")?;
        for i in 0..exprs.len() {
          let exp = exprs.get(i).unwrap();
          if i == exprs.len() - 1 {
            write!(f, "{}", exp)?;
          } else {
            write!(f, "{}, ", exp)?;
          }
        }

        write!(f, ")")?;

        Ok(())
      }
    }
  }
}

fn expr(tokens: &Tokens, min_bp: u8) -> Result<Expression, ParseErrorRepr> {
  let tok = tokens.next();

  let mut lhs = match tok {
    Token::Number(n) => Expression::Number(n),
    Token::Const(constant) => Expression::Constant(constant),
    Token::Operator(op) => match op {
      Operation::Sub => {
        let prefix_bp = op.get_prefix_bp().expect("unreachable");
        let rhs = expr(tokens, prefix_bp)?;
        Expression::Unary {
          op,
          expr: Box::new(rhs),
        }
      }

      _ => {
        return Err(ParseErrorRepr::UnexpectedOperator(op));
      }
    },

    Token::Function(function) => {
      tokens.expect(Token::OpenBracket)?;
      let mut exprs = Vec::<Expression>::new();

      if tokens.peek() != Token::CloseBracket {
        loop {
          exprs.push(expr(tokens, 0)?);

          if tokens.peek() == Token::Comma {
            tokens.next();
            continue;
          }

          break;
        }
      }

      let next = tokens.next();
      if next != Token::CloseBracket && next != Token::Eof {
        return Err(ParseErrorRepr::make_invalid_seq(tok, next));
      }

      Expression::Function {
        func: function,
        exprs,
      }
    }
    Token::OpenBracket => {
      let rhs = expr(tokens, 0)?;
      let next = tokens.next();
      if next != Token::CloseBracket {
        todo!("impl error")
      }

      rhs
    }

    _ => {
      return Err(ParseErrorRepr::make_invalid_seq(tok, tokens.peek()));
    }
  };

  loop {
    let op = match tokens.peek() {
      Token::Operator(op) => op,

      Token::Number(_) | Token::Const(_) | Token::Function(_) | Token::OpenBracket => {
        Operation::Mul
      }

      Token::Eof | Token::CloseBracket | Token::Comma => {
        break;
      }

      #[allow(unreachable_patterns)]
      _ => {
        return Err(ParseErrorRepr::make_invalid_seq(tok, tokens.peek()));
      }
    };

    if let Some((left_bp, right_bp)) = op.get_infix_bp() {
      if left_bp < min_bp {
        break;
      }

      if matches!(tokens.peek(), Token::Operator(..)) {
        tokens.next();
      }

      let rhs = expr(tokens, right_bp)?;

      lhs = Expression::Binary {
        op,
        left: Box::new(lhs),
        right: Box::new(rhs),
      }
    } else if let Some(postfix_bp) = op.get_postfix_bp() {
      if postfix_bp < min_bp {
        break;
      }

      tokens.next();

      lhs = Expression::Unary {
        op,
        expr: Box::new(lhs),
      };
      continue;
    } else {
      return Err(ParseErrorRepr::UnexpectedOperator(op));
    }
  }

  Ok(lhs)
}
