pub mod ast;
pub mod constants;
pub mod eval;
pub mod functions;
pub mod operators;
pub mod tokenize;

use std::process::exit;

use clap::Parser;

use crate::{ast::Expression, tokenize::Tokens};

/// A CLI wrapper around the mparse library for parsing and evaluating basic mathematical expressions from plaintext.
#[derive(Parser, Debug)]
struct Args {
  /// The expression to parse and evaluate.
  input: String,

  /// Displays the expression as the tokens it parsed.
  #[arg(short = 't', long)]
  print_tokens: bool,

  /// Displays the expression parsed into its AST form.
  #[arg(short = 'a', long)]
  print_ast: bool,

  /// Evaluates trigonometric functions in degrees instead of radians.
  #[arg(short = 'd', long = "deg")]
  deg_mode: bool,

  /// Do not attempt to apply any tolerance to the result.
  #[arg(short = 'n', long = "nt")]
  no_tolerance: bool,
}

const EPSILION: f64 = 1e-12;
fn apply_tolerance(flt: f64) -> f64 {
  if flt.abs() < EPSILION {
    return 0f64;
  }

  let as_int = flt.round();
  if (flt - as_int).abs() < EPSILION {
    return as_int;
  }

  flt
}

fn main() {
  let args = Args::parse();
  let tokens_res = Tokens::new(args.input.as_str());
  match tokens_res {
    Err(e) => {
      println!("tokenization error: {}", e);
      exit(0);
    }
    _ => {}
  }

  let tokens = tokens_res.expect("unreachable");
  if args.print_tokens {
    println!("Tokens: {}", tokens);
  }

  let ast_res = Expression::new(&tokens);
  match ast_res {
    Err(e) => {
      println!("ast parsing error: {}", e);
      exit(0);
    }
    _ => {}
  }

  let ast = ast_res.expect("unreachable");
  if args.print_ast {
    println!("AST: {}", ast);
  }

  let expr_res = eval::evaluate(&ast, args.deg_mode);
  match expr_res {
    Ok(res) => {
      let flt = if args.no_tolerance {
        res
      } else {
        apply_tolerance(res)
      };

      println!("{}", flt)
    }
    Err(e) => println!("evaluation error: {}", e),
  }
}
