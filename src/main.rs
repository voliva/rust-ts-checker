mod lexer;
mod lexer_tests;
mod parser;
mod tokens;

use crate::lexer::Lexer;
use crate::parser::{MatchResultValue, Sequence, Terminal};
use crate::tokens::{Literal, Token};
use std::time::Instant;

fn main() {
  let mut matcher = Sequence::new(vec![
    Terminal::<Token>::matcher(|_| true),
    Terminal::<Token>::matcher(|_| false),
  ]);

  let result = matcher.next2(&Token::Literal(Literal::Str(String::from("asdf"))));
  println!("{:?}", result);
  let result = matcher.next2(&Token::Literal(Literal::Str(String::from("fdsa"))));
  println!("{:?}", result);
  let result = matcher.next2(&Token::Literal(Literal::Str(String::from("haha"))));
  println!("{:?}", result);

  // let unwrapped = (match result.value() {
  //   Some(MatchResultValue::Vector(v)) => Some(v),
  //   _ => None,
  // })
  // .unwrap();

  // let res = unwrap_enum!(result.value().unwrap(), MatchResultValue::Vector);

  /*
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
  */
}
