mod lexer;
mod lexer_tests;
mod parser;
mod parser_tests;
mod tokens;

use crate::lexer::Lexer;
use crate::parser::MatcherType;
use crate::parser::{Loop, Matcher, MatcherResult, OneOf, Optional, Sequence, Terminal};
use crate::tokens::{Literal, Token};
use std::time::Instant;

fn main() {
  let mut import_statement = Sequence::new(vec![
    Terminal::matcher(|token| (matches!(token, Token::Keyword(x) if x == "import"))),
    OneOf::matcher(vec![
      // Default export
      Sequence::matcher(vec![
        // something
        Terminal::matcher(|token| (matches!(token, Token::Identifier(_)))),
        // , { namedImport }
        Optional::matcher(Sequence::matcher(vec![
          Terminal::matcher(|token| (matches!(token, Token::Symbol(x) if x == ","))),
          named_imports(),
        ])),
      ]),
      named_imports(),
      // * as something
      Sequence::matcher(vec![
        Terminal::matcher(|token| (matches!(token, Token::Symbol(x) if x == "*"))),
        Terminal::matcher(|token| (matches!(token, Token::Keyword(x) if x == "as"))),
        Terminal::matcher(|token| (matches!(token, Token::Identifier(_)))),
      ]),
    ]),
    Terminal::matcher(|token| (matches!(token, Token::Keyword(x) if x == "from"))),
    Terminal::matcher(|token| (matches!(token, Token::Literal(Literal::Str(_))))),
  ]);

  let now = Instant::now();

  // let lexer = Lexer::from_file("./program.tsx").unwrap();
  let lexer = Lexer::from_text("import { foo: foo2, bar } from 'react'");

  println!("Opened in {}ys", now.elapsed().as_micros());

  let mut result = MatcherResult::Rejected;
  for located_token in lexer {
    match located_token.token {
      Ok(t) => {
        result = import_statement.next(&t);
        println!("{:?} -> {:?}", t, result);
        //   println!(
        //   "line={} col={} {:?}",
        //   located_token.line, located_token.col, t
        // )
      }
      Err(e) => println!(
        "line={} col={} Error! {}",
        located_token.line, located_token.col, e,
      ),
    }
  }

  println!("Complete in {}ys", now.elapsed().as_micros());

  println!("{:?}", result);
}

fn import_unit() -> MatcherType<Token> {
  Sequence::matcher(vec![
    Terminal::matcher(|token| (matches!(token, Token::Identifier(_)))),
    Optional::matcher(Sequence::matcher(vec![
      Terminal::matcher(|token| (matches!(token, Token::Symbol(x) if x == ":"))),
      Terminal::matcher(|token| (matches!(token, Token::Identifier(_)))),
    ])),
  ])
}

fn named_imports() -> MatcherType<Token> {
  Sequence::matcher(vec![
    Terminal::matcher(|token| (matches!(token, Token::Symbol(x) if x == "{"))),
    Optional::matcher(Sequence::matcher(vec![
      import_unit(),
      Optional::matcher(Loop::matcher(Sequence::matcher(vec![
        Terminal::matcher(|token| (matches!(token, Token::Symbol(x) if x == ","))),
        import_unit(),
      ]))),
    ])),
    Terminal::matcher(|token| (matches!(token, Token::Symbol(x) if x == "}"))),
  ])
}
