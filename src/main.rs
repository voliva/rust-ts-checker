mod ast;
mod lexer;
mod lexer_tests;
mod parser;
mod parser_tests;
mod tokens;

use crate::ast::source_file::SourceFile;
use crate::lexer::Lexer;
use std::time::Instant;

fn main() {
  let now = Instant::now();

  // let lexer = Lexer::from_file("./program.tsx").unwrap();
  let lexer = Lexer::from_text(
    "
    import foo, { foo: foo2, bar } from 'react';
  ",
  );

  println!("Opened in {}ys", now.elapsed().as_micros());

  let source_file = SourceFile::from(lexer);

  println!("Complete in {}ys", now.elapsed().as_micros());

  println!("{:?}", source_file);
}
