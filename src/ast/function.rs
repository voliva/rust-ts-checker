use crate::ast::utils::{peek_token, read_token};
use crate::parser::{
  Loop, MatchResultValue, MatcherResult, MatcherType, Optional, Sequence, Terminal,
};
use crate::tokens::Token;
use crate::Lexer;
use crate::{unwrap_enum, unwrap_match};
use core::iter::Peekable;
use std::ops::Deref;

#[derive(Debug)]
pub struct FunctionDeclaration {
  identifier: String,
  generics: Vec<FunctionGeneric>,
  parameters: Vec<FunctionParameter>,
  // body
}

impl FunctionDeclaration {
  pub fn create(lexer: &mut Peekable<Lexer>) -> Option<Result<Self, String>> {
    let (token, ..) = peek_token(lexer).ok()?;

    let mut parser = function_declaration();
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
        MatcherResult::End(v) => break Ok(parse_function_declaration(&v)),
        _ => {}
      }
    };
    Some(result)
  }
}

fn function_declaration() -> MatcherType<Token> {
  Sequence::matcher(vec![
    Terminal::matcher(|token| (matches!(token, Token::Keyword(x) if x == "function"))),
    Terminal::matcher(|token| (matches!(token, Token::Identifier(_)))),
    Optional::matcher(function_generics()),
    function_parameters(),
    Terminal::matcher(|token| (matches!(token, Token::Symbol(x) if x == "{"))),
    // TODO body
    Terminal::matcher(|token| (matches!(token, Token::Symbol(x) if x == "}"))),
  ])
}
fn parse_function_declaration(value: &MatchResultValue<Token>) -> FunctionDeclaration {
  let seq = unwrap_enum!(value, MatchResultValue::Vector);
  let identifier = unwrap_match!(seq[1], MatchResultValue::Token(Token::Identifier(i)) => i);
  let generics = match unwrap_enum!(seq[2], MatchResultValue::Option) {
    Some(v) => parse_function_generics(v.deref()),
    None => vec![],
  };
  let parameters = parse_function_parameters(&seq[3]);

  FunctionDeclaration {
    identifier: identifier.clone(),
    generics,
    parameters,
  }
}

#[derive(Debug)]
struct FunctionGeneric {
  identifier: String,
  extends: Option<TypeDefinition>,
}

fn function_generics() -> MatcherType<Token> {
  Sequence::matcher(vec![
    Terminal::matcher(|token| (matches!(token, Token::Symbol(x) if x == "<"))),
    Loop::matcher(Sequence::matcher(vec![
      Terminal::matcher(|token| (matches!(token, Token::Identifier(_)))),
      Optional::matcher(Sequence::matcher(vec![
        Terminal::matcher(|token| (matches!(token, Token::Keyword(x) if x == "extends"))),
        type_definition(),
      ])),
      Optional::matcher(Terminal::matcher(
        |token| (matches!(token, Token::Symbol(x) if x == ",")),
      )),
    ])),
    Terminal::matcher(|token| (matches!(token, Token::Symbol(x) if x == ">"))),
  ])
}
fn parse_function_generics(value: &MatchResultValue<Token>) -> Vec<FunctionGeneric> {
  let loop_match_result = &unwrap_enum!(value, MatchResultValue::Vector)[1];
  let loop_results = unwrap_enum!(loop_match_result, MatchResultValue::Vector);

  loop_results
    .into_iter()
    .map(|loop_match| {
      let seq = unwrap_enum!(loop_match, MatchResultValue::Vector);
      let identifier = unwrap_match!(seq[0], MatchResultValue::Token(Token::Identifier(i)) => i);
      let extends = match unwrap_enum!(seq[1], MatchResultValue::Option) {
        Some(v) => Some(parse_definition(
          &unwrap_enum!(v.deref(), MatchResultValue::Vector)[1],
        )),
        None => None,
      };

      FunctionGeneric {
        identifier: identifier.clone(),
        extends,
      }
    })
    .collect()
}

#[derive(Debug)]
struct FunctionParameter {
  identifier: String,
  definition: Option<TypeDefinition>,
  optional: bool,
  initializer: Option<Expression>,
}

fn function_parameters() -> MatcherType<Token> {
  Sequence::matcher(vec![
    Terminal::matcher(|token| (matches!(token, Token::Symbol(x) if x == "("))),
    Loop::matcher(Sequence::matcher(vec![
      Terminal::matcher(|token| (matches!(token, Token::Identifier(_)))),
      Optional::matcher(Terminal::matcher(
        |token| (matches!(token, Token::Symbol(x) if x == "?")),
      )),
      Optional::matcher(Sequence::matcher(vec![
        Terminal::matcher(|token| (matches!(token, Token::Symbol(x) if x == ":"))),
        type_definition(),
      ])),
      Optional::matcher(Sequence::matcher(vec![
        Terminal::matcher(|token| (matches!(token, Token::Symbol(x) if x == "="))),
        expression(),
      ])),
      Optional::matcher(Terminal::matcher(
        |token| (matches!(token, Token::Symbol(x) if x == ",")),
      )),
    ])),
    Terminal::matcher(|token| (matches!(token, Token::Symbol(x) if x == ")"))),
  ])
}

fn parse_function_parameters(value: &MatchResultValue<Token>) -> Vec<FunctionParameter> {
  let loop_match_result = &unwrap_enum!(value, MatchResultValue::Vector)[1];
  let loop_results = unwrap_enum!(loop_match_result, MatchResultValue::Vector);

  loop_results
    .into_iter()
    .map(|loop_match| {
      let seq = unwrap_enum!(loop_match, MatchResultValue::Vector);
      let identifier = unwrap_match!(seq[0], MatchResultValue::Token(Token::Identifier(i)) => i);
      let optional = unwrap_enum!(seq[1], MatchResultValue::Option);
      let definition = match unwrap_enum!(seq[2], MatchResultValue::Option) {
        Some(v) => Some(parse_definition(
          &unwrap_enum!(v.deref(), MatchResultValue::Vector)[1],
        )),
        None => None,
      };
      let initializer = match unwrap_enum!(seq[2], MatchResultValue::Option) {
        Some(v) => Some(parse_expression(
          &unwrap_enum!(v.deref(), MatchResultValue::Vector)[1],
        )),
        None => None,
      };

      FunctionParameter {
        identifier: identifier.clone(),
        optional: matches!(optional, Some(_)),
        definition,
        initializer,
      }
    })
    .collect()
}

// to be declared on external files
#[derive(Debug)]
struct TypeDefinition {
  // TODO
}

fn type_definition() -> MatcherType<Token> {
  // TODO
  Terminal::matcher(|token| (matches!(token, Token::Identifier(_))))
}
fn parse_definition(_: &MatchResultValue<Token>) -> TypeDefinition {
  TypeDefinition {}
}

#[derive(Debug)]
struct Expression {
  // TODO
}

fn expression() -> MatcherType<Token> {
  // TODO
  Terminal::matcher(|token| (matches!(token, Token::Identifier(_))))
}

fn parse_expression(_: &MatchResultValue<Token>) -> Expression {
  Expression {}
}
