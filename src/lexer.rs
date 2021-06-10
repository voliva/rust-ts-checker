use crate::tokens::*;
use std::iter::Peekable;
use std::vec::{IntoIter, Vec};
use std::{fs, io};

pub struct Lexer {
  raw_data: Peekable<IntoIter<char>>,
  state: Vec<LexerState>,
}

#[derive(Copy, Clone, Debug)]
enum LexerState {
  Typescript(TypescriptState),
  Jsx(i32),
}

#[derive(Copy, Clone, Debug)]
struct TypescriptState {
  bracket_stack: i32,
  jsx_transition: JSXTransition,
}

/**
 * Case we've found `<identifier`. Only valid possibilities are:
 * if (a < identifier) => Typescript
 * const myFunction = <identifier extends any>() => Typescript
 * const myFunction = <identifier, T2>() => Typescript
 * <identifier> => JSX
 * <identifier /> => JSX
 * if (a < identifier / 3) => Typescript
 * <identifier foo=""> => JSX
 */
#[derive(Copy, Clone, Debug)]
enum JSXTransition {
  None,       // '<' => Bracket, _ => None
  Bracket,    // identifier => Identifier, _ => None
  Identifier, // 'extends' => None, (>,/>,identifier) => JSX, _ => None
}

impl Lexer {
  pub fn from_text(text: &str) -> Self {
    Lexer {
      raw_data: text.chars().collect::<Vec<_>>().into_iter().peekable(),
      state: vec![LexerState::Typescript(TypescriptState {
        bracket_stack: 1,
        jsx_transition: JSXTransition::None,
      })],
    }
  }

  pub fn from_file(file_path: &str) -> io::Result<Self> {
    Ok(Self::from_text(&fs::read_to_string(file_path)?))
  }

  fn get_next_char_while<F>(&mut self, raw_token: &mut String, cond: F)
  where
    F: Fn(char) -> bool,
  {
    loop {
      match self.raw_data.peek() {
        Some(c) if cond(*c) => {
          raw_token.push(*c);
          self.raw_data.next();
        }
        _ => break,
      }
    }
  }

  fn replace_state(&mut self, state: LexerState) {
    let position = self.state.len() - 1;
    self.state[position] = state;
  }
}

type LexerItem = std::result::Result<Token, String>;

impl Iterator for Lexer {
  type Item = LexerItem;

  fn next(&mut self) -> Option<LexerItem> {
    loop {
      let length = self.state.len() - 1;
      let maybe_result = match self.state[length] {
        LexerState::Typescript(n) => next_typescript(self, n),
        v => panic!("No parser for state {:?}", v),
      };
      match maybe_result {
        None => {}
        Some(r) => {
          break r;
        }
      }
    }
  }
}

fn next_typescript(lexer: &mut Lexer, state: TypescriptState) -> Option<Option<LexerItem>> {
  let token: LexerItem;

  let first_char: char;
  loop {
    match lexer.raw_data.next() {
      Some(c) if c.is_whitespace() => continue,
      Some(c) => {
        first_char = c;
        break;
      }
      None => return Some(None),
    }
  }

  if first_char.is_numeric() {
    let mut value = first_char.to_string();
    lexer.get_next_char_while(&mut value, |c| c.is_numeric());

    token = match value.parse() {
      Ok(i) => Ok(Token::Literal(Literal::Integer(i))),
      Err(_) => Err(format!("Integer literal {} is invalid", value)),
    };

    if !matches!(state.jsx_transition, JSXTransition::None) {
      lexer.replace_state(LexerState::Typescript(TypescriptState {
        bracket_stack: state.bracket_stack,
        jsx_transition: JSXTransition::None,
      }))
    }
  } else if is_identifier(first_char) {
    let mut name = first_char.to_string();
    lexer.get_next_char_while(&mut name, is_identifier);

    match state.jsx_transition {
      JSXTransition::Bracket => lexer.replace_state(LexerState::Typescript(TypescriptState {
        bracket_stack: state.bracket_stack,
        jsx_transition: JSXTransition::Identifier,
      })),
      JSXTransition::Identifier => {
        if name == "extends" {
          lexer.replace_state(LexerState::Typescript(TypescriptState {
            bracket_stack: state.bracket_stack,
            jsx_transition: JSXTransition::None,
          }))
        } else {
          lexer.state.push(LexerState::Jsx(1))
        }
      }
      _ => {}
    }
    if KNOWN_KEYWORDS.contains(&&name[..]) {
      token = Ok(Token::Keyword(name))
    } else {
      token = Ok(Token::Identifier(name))
    };
  } else if first_char == '"' || first_char == '\'' {
    let mut value = String::new();
    lexer.get_next_char_while(&mut value, |c| c != first_char);
    // We need to exclude the last closing character
    lexer.raw_data.next();
    token = Ok(Token::Literal(Literal::Str(value)));

    if !matches!(state.jsx_transition, JSXTransition::None) {
      lexer.replace_state(LexerState::Typescript(TypescriptState {
        bracket_stack: state.bracket_stack,
        jsx_transition: JSXTransition::None,
      }))
    }
  } else {
    let mut raw = first_char.to_string();
    loop {
      if let Some(peek) = lexer.raw_data.peek() {
        raw.push(*peek);
      } else {
        // We reached the end of the program.
        break;
      }

      if VALID_SYMBOLS.contains(&&raw[..]) {
        lexer.raw_data.next();
      } else {
        raw.pop();
        break;
      }
    }

    // Change state
    if raw == "{" {
      lexer.replace_state(LexerState::Typescript(TypescriptState {
        bracket_stack: state.bracket_stack + 1,
        jsx_transition: state.jsx_transition,
      }))
    } else if raw == "}" {
      if state.bracket_stack == 1 {
        lexer.state.pop();
      } else {
        lexer.replace_state(LexerState::Typescript(TypescriptState {
          bracket_stack: state.bracket_stack - 1,
          jsx_transition: state.jsx_transition,
        }))
      }
    } else {
      match state.jsx_transition {
        JSXTransition::None if raw == "<" => {
          lexer.replace_state(LexerState::Typescript(TypescriptState {
            bracket_stack: state.bracket_stack,
            jsx_transition: JSXTransition::Bracket,
          }))
        }
        JSXTransition::Identifier if raw == ">" || raw == "/>" => {
          lexer.state.push(LexerState::Jsx(1))
        }
        _ => lexer.replace_state(LexerState::Typescript(TypescriptState {
          bracket_stack: state.bracket_stack,
          jsx_transition: JSXTransition::None,
        })),
      };
    }

    token = match &raw[..] {
      // Ignore comments until newline
      s if s == "//" => {
        lexer.get_next_char_while(&mut String::new(), |c| c != '\n');
        return None;
      }
      s if s == "/*" => {
        let mut prev = '_';

        loop {
          match lexer.raw_data.peek() {
            Some('/') if prev == '*' => {
              lexer.raw_data.next();
              break;
            }
            Some(c) => {
              prev = *c;
              lexer.raw_data.next();
            }
            _ => break,
          }
        }
        return None;
      }
      s if VALID_SYMBOLS.contains(&s) => Ok(Token::Symbol(raw)),
      _ => Err(format!("Unknown token: {}", raw)),
    };
  }

  return Some(Some(token));
}

fn is_identifier(c: char) -> bool {
  c.is_ascii_alphanumeric() || c == '_' || c == '$'
}
