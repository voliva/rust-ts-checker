mod lexer;
mod tokens;

use crate::lexer::Lexer;

fn main() {
  let lexer = Lexer::from_file("./program.tsx").unwrap();
  for token in lexer {
    match token {
      Ok(t) => println!("{:?}", t),
      Err(e) => println!("Error! {}", e),
    }
  }
}
