mod lexer;
mod lexer_tests;
mod tokens;

use crate::lexer::Lexer;
use std::time::Instant;

fn main() {
  let now = Instant::now();

  let lexer = Lexer::from_file("./program.tsx").unwrap();

  println!("Opened in {}ys", now.elapsed().as_micros());

  for located_token in lexer {
    match located_token.token {
      Ok(t) => println!(
        "line={} col={} {:?}",
        located_token.line, located_token.col, t
      ),
      Err(e) => println!(
        "line={} col={} Error! {}",
        located_token.line, located_token.col, e,
      ),
    }
  }

  println!("Complete in {}ys", now.elapsed().as_micros());
}
