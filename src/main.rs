use std::io::{self, BufRead};

use clap::Parser;
use mparse::{Error, ast::Expression, eval, tokenize::Tokens};

/// A CLI wrapper around the mparse library for parsing and evaluating basic mathematical expressions from plaintext.
#[derive(Parser, Debug)]
struct Args {
  /// The expression to parse and evaluate.
  input: Option<String>,

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
  let mut args = Args::parse();

  if args.input.is_some() {
    let res = parse(&args);
    match res {
      Err(e) => {
        println!("{}", e);
      }
      Ok(_) => println!("{}", res.expect("unreachable")),
    }
  } else {
    let stdin = io::stdin();
    loop {
      let mut input_str = String::new();
      let mut lock = stdin.lock();
      let _ = lock.read_line(&mut input_str).expect("read line error");
      args.input = Some(input_str);

      let res = parse(&args);
      match res {
        Err(e) => {
          println!("{}", e);
        }
        Ok(_) => println!("{}", res.expect("unreachable")),
      }
    }
  };
}

fn parse(args: &Args) -> Result<f64, Error> {
  let tokens_res = Tokens::new(args.input.as_ref().expect("unreachable").as_str());
  match tokens_res {
    Err(e) => {
      return Err(Error::TokenizeError(e));
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
      return Err(Error::ParseError(e));
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

      Ok(flt)
    }
    Err(e) => Err(Error::EvaluationError(e)),
  }
}
