use super::function::FunctionDeclaration;
use super::imports::ImportDeclaration;
use crate::lexer::Lexer;
use crate::tokens::Token;
use core::iter::Peekable;

/// SourceFile
#[derive(Debug)]
pub struct SourceFile {
  children: Vec<SourceFileElement>,
}

#[derive(Debug)]
enum SourceFileElement {
  ImportDeclaration(ImportDeclaration),
  FunctionDeclaration(FunctionDeclaration),
}

impl From<Lexer> for SourceFile {
  fn from(lexer: Lexer) -> Self {
    let mut peekable = lexer.peekable();

    let mut children = vec![];

    loop {
      while next_is_semicolon(&mut peekable) {
        peekable.next();
      }

      let result = ImportDeclaration::create(&mut peekable);
      match result {
        Some(Ok(v)) => {
          children.push(SourceFileElement::ImportDeclaration(v));
          continue;
        }
        Some(Err(r)) => {
          panic!("Error parsing import: {}", r);
        }
        _ => {}
      };

      let result = FunctionDeclaration::create(&mut peekable);
      match result {
        Some(Ok(v)) => {
          children.push(SourceFileElement::FunctionDeclaration(v));
          continue;
        }
        Some(Err(r)) => {
          panic!("Error parsing function: {}", r);
        }
        _ => {}
      };

      break;
    }

    SourceFile { children }
  }
}

fn next_is_semicolon(peekable: &mut Peekable<Lexer>) -> bool {
  matches!(peekable.peek(), Some(located_token) if matches!(&located_token.token, Ok(Token::Symbol(s)) if s == ";"))
}
