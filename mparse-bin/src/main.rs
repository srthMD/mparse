use std::{
  io::{self, BufRead},
  process::exit,
};

use clap::Parser;
use mparse::{
  Error,
  ast::Expression,
  eval,
  tokenize::Tokens,
  types::{
    object::Object,
    vector::{Vec2D, Vec3D},
  },
};

/// A CLI wrapper around the mparse library for parsing and evaluating basic mathematical expressions from plaintext.
#[derive(Parser, Debug)]
struct Args {
  /// The expression to parse and evaluate.
  input: Option<String>,

  /// Displays the expression as the tokens it parsed.
  #[arg(long)]
  print_tokens: bool,

  /// Displays the expression parsed into its AST form.
  #[arg(long)]
  print_ast: bool,

  /// Evaluates trigonometric functions in degrees instead of radians.
  #[arg(short = 'd', long = "deg")]
  deg_mode: bool,

  /// Do not attempt to apply any tolerance to the result.
  #[arg(short = 'n', long = "no-tolerance")]
  no_tolerance: bool,
}

fn apply_tolerance(obj: Object) -> Object {
  match obj {
    Object::Number(n) => apply_tolerance_f64(n).into(),
    Object::Vec2D(vec2_d) => apply_tolerance_vec2d(vec2_d).into(),
    Object::Vec3D(vec3_d) => apply_tolerance_vec3d(vec3_d).into(),
    _ => obj,
  }
}

fn apply_tolerance_vec2d(vec: Vec2D) -> Vec2D {
  Vec2D {
    x: apply_tolerance_f64(vec.x),
    y: apply_tolerance_f64(vec.y),
  }
}

fn apply_tolerance_vec3d(vec: Vec3D) -> Vec3D {
  Vec3D {
    x: apply_tolerance_f64(vec.x),
    y: apply_tolerance_f64(vec.y),
    z: apply_tolerance_f64(vec.z),
  }
}

const EPSILION: f64 = 1e-12;
fn apply_tolerance_f64(flt: f64) -> f64 {
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

      if input_str.trim() == "exit" {
        exit(0)
      }

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

fn parse(args: &Args) -> Result<Object, Error> {
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
