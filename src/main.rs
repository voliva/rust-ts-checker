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

    function myFunction<T, K extends string>(value: T, key?: K, option = false) {}
  ",
  );

  println!("Opened in {}ms", now.elapsed().as_micros() as f64 / 1000.0);

  let source_file = SourceFile::from(lexer);

  println!(
    "Complete in {}ms",
    now.elapsed().as_micros() as f64 / 1000.0
  );

  println!("{:?}", source_file);
}
