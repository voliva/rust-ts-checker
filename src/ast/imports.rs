use super::utils::{peek_token, read_token};
use crate::lexer::Lexer;
use crate::parser::{
  Loop, MatchResultValue, MatcherResult, MatcherType, OneOf, Optional, Sequence, Terminal,
};
use crate::tokens::{Literal, Token};
use crate::{unwrap_branch, unwrap_enum, unwrap_match};
use core::iter::Peekable;
use std::ops::Deref;

#[derive(Debug)]
pub struct ImportDeclaration {
  target: String,
  default: Option<String>,
  clause: Option<ImportClause>,
}

#[derive(Debug)]
enum ImportClause {
  NamespaceImport(String),
  NamedImports(Vec<NamedImport>),
}

impl ImportDeclaration {
  pub fn create(lexer: &mut Peekable<Lexer>) -> Option<Result<Self, String>> {
    let (token, ..) = peek_token(lexer).ok()?;

    let mut parser = import_statement();
    let parser_result = parser.next(&token);

    if matches!(parser_result, MatcherResult::Rejected) {
      return None;
    }
    lexer.next();

    let result = loop {
      let (token, line, col) = match read_token(lexer) {
        Ok(v) => v,
        Err(r) => break Err(r),
      };

      match parser.next(&token) {
        MatcherResult::Rejected => {
          break Err(format!(
            "line: {} col: {} unexpected token {:?}",
            line, col, token
          ))
        }
        MatcherResult::End(v) => break Ok(parse_import(&v)),
        _ => {}
      }
    };
    Some(result)
  }
}

// NamedImport
#[derive(Debug)]
struct NamedImport {
  original: String,
  alias: Option<String>,
}

fn import_statement() -> MatcherType<Token> {
  Sequence::matcher(vec![
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
  ])
}
fn parse_import(value: &MatchResultValue<Token>) -> ImportDeclaration {
  let result = unwrap_enum!(value, MatchResultValue::Vector);
  let target =
    unwrap_match!(result[3], MatchResultValue::Token(Token::Literal(Literal::Str(v))) => v);

  let (clause, default) = match unwrap_branch!(result[1]) {
    (0, r) => {
      let sequence = unwrap_enum!(r.deref(), MatchResultValue::Vector);
      let default = unwrap_match!(sequence[0], MatchResultValue::Token(Token::Identifier(i)) => i);
      let clause = match unwrap_enum!(sequence[1], MatchResultValue::Option) {
        Some(v) => Some(ImportClause::NamedImports(parse_named_imports(
          &unwrap_enum!(v.deref(), MatchResultValue::Vector)[1],
        ))),
        None => None,
      };
      (clause, Some(default.clone()))
    }
    (1, r) => (
      Some(ImportClause::NamedImports(parse_named_imports(r))),
      None,
    ),
    (2, r) => {
      let v = unwrap_enum!(r.deref(), MatchResultValue::Vector);

      (
        Some(ImportClause::NamespaceImport(
          unwrap_match!(v[2], MatchResultValue::Token(Token::Identifier(i)) => i).clone(),
        )),
        None,
      )
    }
    (_, _) => (None, None),
  };

  ImportDeclaration {
    target: target.clone(),
    clause,
    default,
  }
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
fn parse_named_imports(value: &MatchResultValue<Token>) -> Vec<NamedImport> {
  let source = unwrap_enum!(value, MatchResultValue::Vector);

  match unwrap_enum!(source[1], MatchResultValue::Option) {
    None => vec![],
    Some(v) => {
      let seq = unwrap_enum!(v.deref(), MatchResultValue::Vector);
      let mut result = vec![parse_import_unit(&seq[0])];
      let tail = match unwrap_enum!(seq[1], MatchResultValue::Option) {
        Some(v) => unwrap_enum!(v.deref(), MatchResultValue::Vector).clone(),
        None => vec![],
      };

      for s in tail {
        let seq = unwrap_enum!(s, MatchResultValue::Vector);
        result.push(parse_import_unit(&seq[1]))
      }

      result
    }
  }
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
fn parse_import_unit(value: &MatchResultValue<Token>) -> NamedImport {
  let source = unwrap_enum!(value, MatchResultValue::Vector);
  let original =
    unwrap_match!(source[0], MatchResultValue::Token(Token::Identifier(i)) => i.clone());

  match unwrap_enum!(source[1], MatchResultValue::Option) {
    None => NamedImport {
      original,
      alias: None,
    },
    Some(v) => {
      let seq = unwrap_enum!(v.deref(), MatchResultValue::Vector);
      let alias = unwrap_match!(seq[1], MatchResultValue::Token(Token::Identifier(i)) => i.clone());
      NamedImport {
        original,
        alias: Some(alias),
      }
    }
  }
}
