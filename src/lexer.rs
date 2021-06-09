use crate::tokens::*;
use std::iter::Peekable;
use std::vec::IntoIter;
use std::{fs, io};

pub struct Lexer {
  raw_data: Peekable<IntoIter<char>>,
  state: LexerState,
}

enum LexerState {
  Typescript,
  TemplateString,
  /**
   * Case we've found `<identifier`. Only valid possibilities are:
   * if (a < identifier) => Typescript
   * const myFunction<identifier extends any>() => Typescript
   * const myFunction<identifier, T2>() => Typescript
   * <identifier> => JSX
   * <identifier /> => JSX
   * <identifier foo=""> => JSX
   *
   * Within JSX I can have a typescript insert, which can have jsx etc. - When
   * getting out from typescript, how do I make it to go back to jsx?
   * stack... but how?
   */
  MaybeJsx,
  Jsx(i32),
}

impl Lexer {
  pub fn from_text(text: &str) -> Self {
    Lexer {
      raw_data: text.chars().collect::<Vec<_>>().into_iter().peekable(),
      state: LexerState::Typescript,
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
}

type LexerItem = std::result::Result<Token, String>;

impl Iterator for Lexer {
  type Item = LexerItem;

  fn next(&mut self) -> Option<LexerItem> {
    match self.state {
      LexerState::Typescript => next_typescript(self),
      _ => None,
    }
  }
}

fn next_typescript(lexer: &mut Lexer) -> Option<LexerItem> {
  let token: LexerItem;

  let first_char: char;
  loop {
    match lexer.raw_data.next() {
      Some(c) if c.is_whitespace() => continue,
      Some(c) => {
        first_char = c;
        break;
      }
      None => return None,
    }
  }

  if first_char.is_numeric() {
    let mut value = first_char.to_string();
    lexer.get_next_char_while(&mut value, |c| c.is_numeric());

    token = match value.parse() {
      Ok(i) => Ok(Token::Literal(Literal::Integer(i))),
      Err(_) => Err(format!("Integer literal {} is invalid", value)),
    }
  } else if is_identifier(first_char) {
    let mut name = first_char.to_string();
    lexer.get_next_char_while(&mut name, is_identifier);

    if KNOWN_KEYWORDS.contains(&&name[..]) {
      token = Ok(Token::Keyword(name))
    } else {
      token = Ok(Token::Identifier(name))
    }
  } else if first_char == '"' || first_char == '\'' {
    let mut value = String::new();
    lexer.get_next_char_while(&mut value, |c| c != first_char);
    // We need to exclude the last closing character
    lexer.raw_data.next();
    token = Ok(Token::Literal(Literal::Str(value)))
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

    token = match &raw[..] {
      // Ignore comments until newline
      s if s == "//" => {
        lexer.get_next_char_while(&mut String::new(), |c| c != '\n');
        lexer.next()?
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
        lexer.next()?
      }
      s if VALID_SYMBOLS.contains(&s) => Ok(Token::Symbol(raw)),
      _ => Err(format!("Unknown token: {}", raw)),
    }
  }

  return Some(token);
}

fn is_identifier(c: char) -> bool {
  c.is_ascii_alphanumeric() || c == '_' || c == '$'
}
