use crate::lexer::Lexer;
use crate::tokens::Token;
use core::iter::Peekable;

/// Utils
pub fn peek_token(lexer: &mut Peekable<Lexer>) -> Result<(Token, i32, i32), String> {
  match lexer.peek() {
    Some(located_token) => match &located_token.token {
      Ok(t) => Ok((t.clone(), located_token.line, located_token.col)),
      Err(t) => Err(format!(
        "line: {} col: {} ImportDeclaration: {}",
        located_token.line, located_token.col, t
      )),
    },
    _ => Err("Unexpected EOF".to_owned()),
  }
}

pub fn read_token(lexer: &mut Peekable<Lexer>) -> Result<(Token, i32, i32), String> {
  match lexer.next() {
    Some(located_token) => match &located_token.token {
      Ok(t) => Ok((t.clone(), located_token.line, located_token.col)),
      Err(t) => Err(format!(
        "line: {} col: {} ImportDeclaration: {}",
        located_token.line, located_token.col, t
      )),
    },
    _ => Err("Unexpected EOF".to_owned()),
  }
}
