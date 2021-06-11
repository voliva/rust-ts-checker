mod lexer;
mod lexer_tests;
mod tokens;

use crate::lexer::Lexer;
use std::time::Instant;

fn main() {
  let now = Instant::now();

  let lexer = Lexer::from_file("./program.tsx").unwrap();

  println!("Opened in {}ys", now.elapsed().as_micros());

  for token in lexer {
    match token {
      Ok(t) => println!("{:?}", t),
      Err(e) => println!("Error! {}", e),
    }
  }

  println!("Complete in {}ys", now.elapsed().as_micros());
}
