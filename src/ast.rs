use crate::lexer::Lexer;
use crate::tokens::{Literal, Token};
use core::iter::Peekable;

/// SourceFile
pub struct SourceFile {
  children: Vec<SourceFileElement>,
}

enum SourceFileElement {
  ImportDeclaration(ImportDeclaration),
  FunctionDeclaration(FunctionDeclaration),
}

impl From<Lexer> for SourceFile {
  fn from(lexer: Lexer) -> Self {
    let peekable = lexer.peekable();

    SourceFile {}
  }
}

/// ImportDeclaration
pub struct ImportDeclaration {
  target: String,
  default: Option<String>,
  clause: Option<ImportClause>,
}

enum ImportClause {
  NamespaceImport(String),
  NamedImports(Vec<NamedImport>),
}

enum ImportDeclarationState {
  Starting(Option<String>),
  Namespace(Option<String>, Option<String>),
  Named(Option<String>, Vec<NamedImport>),
}

impl ImportDeclaration {
  fn create(lexer: &mut Peekable<Lexer>) -> Option<Result<Self, String>> {
    let token = peek_token(lexer);

    match token {
      Ok((Token::Keyword(keyword), ..)) if keyword == "import" => {}
      _ => {
        return None;
      }
    };
    lexer.next();

    let mut state: ImportDeclarationState = ImportDeclarationState::Starting(None);

    let result = loop {
      let (token, line, col) = match read_token(lexer) {
        Ok(r) => r,
        Err(r) => {
          return Some(Err(r));
        }
      };

      match (state, token) {
        (ImportDeclarationState::Starting(default), Token::Identifier(identifier)) => {
          if default == None {
            state = ImportDeclarationState::Starting(Some(identifier))
          } else {
            break Err(format!(
              "line: {} col: {} Can't have more than one default declaration",
              line, col
            ));
          }
        }
        (ImportDeclarationState::Starting(default), Token::Symbol(s)) => {
          if s == "*" {
            state = ImportDeclarationState::Namespace(default, None)
          } else if s == "{" {
            state = ImportDeclarationState::Named(default, Vec::new())
          } else {
            break Err(format!(
              "line: {} col: {} Unexpected symbol {}",
              line, col, s
            ));
          }
        }
        (ImportDeclarationState::Namespace(..), Token::Keyword(s)) if s == "as" => {}
        (ImportDeclarationState::Namespace(default, identifier), Token::Identifier(i)) => {
          if identifier == None {
            state = ImportDeclarationState::Namespace(default, Some(i));
          } else {
            break Err(format!(
              "line: {} col: {} Can't have more than one namespace import",
              line, col
            ));
          }
        }
        (ImportDeclarationState::Namespace(default, identifier), Token::Keyword(k))
          if k == "from" =>
        {
          if identifier == None {
            break Err(format!("line: {} col: {} Missing name", line, col));
          }
        }
        (
          ImportDeclarationState::Namespace(default, identifier),
          Token::Literal(Literal::Str(s)),
        ) => {
          // Funny thing is that this import will be valid:
          // import * as as as React from from from "react"
          break match identifier {
            None => Err(format!("line: {} col: {} Missing name", line, col)),
            Some(i) => Ok(ImportDeclaration {
              default,
              target: s,
              clause: Some(ImportClause::NamespaceImport(i)),
            }),
          };
        }
        _ => {
          break Err(format!(
            "line: {} col: {} Unexpected token {:?}",
            line, col, token
          ))
        }
      }
    };
    Some(result)

    // let mut target: String;
    // let mut default: Option<String> = None;
    // let mut clause: Option<ImportClause> = None;

    // let token = read_token(lexer);
    // match token {
    //   Token::Identifier(i) => {
    //     default = Some(i);
    //   }
    //   Token::Symbol(s) if s == "*" => {}
    //   Token::Symbol(s) if s == "{" => {}
    //   _ => panic!(
    //     "line: ? col: ? ImportDeclaration: Unexpected token {:?}",
    //     token
    //   ),
    // }

    // None
    // ImportDeclaration {
    //   target,
    //   default,
    //   clause,
    // }
  }
}

// NamedImport
struct NamedImport {
  original: String,
  alias: Option<String>,
}

pub struct FunctionDeclaration {}

/// Utils
fn peek_token(lexer: &Peekable<Lexer>) -> Result<(Token, i32, i32), String> {
  match lexer.peek() {
    Some(locatedToken) => match locatedToken.token {
      Ok(t) => Ok((t, locatedToken.line, locatedToken.col)),
      Err(t) => Err(format!(
        "line: {} col: {} ImportDeclaration: {}",
        locatedToken.line, locatedToken.col, t
      )),
    },
    _ => Err("Unexpected EOF".to_owned()),
  }
}

fn read_token(lexer: &Peekable<Lexer>) -> Result<(Token, i32, i32), String> {
  match lexer.next() {
    Some(locatedToken) => match locatedToken.token {
      Ok(t) => Ok((t, locatedToken.line, locatedToken.col)),
      Err(t) => Err(format!(
        "line: {} col: {} ImportDeclaration: {}",
        locatedToken.line, locatedToken.col, t
      )),
    },
    _ => Err("Unexpected EOF".to_owned()),
  }
}
