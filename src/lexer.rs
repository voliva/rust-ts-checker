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
  Jsx(JSXState),
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

#[derive(Copy, Clone, Debug)]
enum JSXState {
  Element(i32), // <something<generic<T>> something="whatever"
  Children,     // <something>...
  Closing,      // </something
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

pub type LexerItem = std::result::Result<Token, String>;

impl Iterator for Lexer {
  type Item = LexerItem;

  fn next(&mut self) -> Option<LexerItem> {
    loop {
      let last = self.state.len() - 1;
      // println!("{:?}", self.state[last]);
      let maybe_result = match self.state[last] {
        LexerState::Typescript(n) => next_typescript(self, n),
        LexerState::Jsx(n) => next_jsx(self, n),
        // v => panic!("No parser for state {:?}", v),
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
      // not covered
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
          // not covered
          lexer.state.push(LexerState::Jsx(JSXState::Element(1)))
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
      // not covered
      lexer.replace_state(LexerState::Typescript(TypescriptState {
        bracket_stack: state.bracket_stack,
        jsx_transition: JSXTransition::None,
      }))
    }
  } else {
    let raw = read_symbol(lexer, &first_char);

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
        JSXTransition::None if raw == "<>" => {
          // not covered
          lexer.state.push(LexerState::Jsx(JSXState::Children))
        }
        JSXTransition::Identifier if raw == ">" => {
          lexer.state.push(LexerState::Jsx(JSXState::Children))
        }
        JSXTransition::Identifier if raw == "<" => {
          // not covered
          lexer.state.push(LexerState::Jsx(JSXState::Element(2)))
        }
        JSXTransition::Identifier if raw == "/>" => {
          // not covered
          lexer.replace_state(LexerState::Typescript(TypescriptState {
            bracket_stack: state.bracket_stack,
            jsx_transition: JSXTransition::None,
          }))
        }
        _ => {
          if raw != "." {
            // not covered
            lexer.replace_state(LexerState::Typescript(TypescriptState {
              bracket_stack: state.bracket_stack,
              jsx_transition: JSXTransition::None,
            }))
          }
        }
      };
    }

    token = match &raw[..] {
      // Ignore comments until newline
      // not covered
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

fn next_jsx(lexer: &mut Lexer, state: JSXState) -> Option<Option<LexerItem>> {
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

  let token: LexerItem = match state {
    JSXState::Element(element_stack) => {
      /* Valid tokens are just a few:
       * <element value="asdf" typescript={123}>
       * - identifier
       * - =
       * - " => string literal
       * - { => go typescript
       * - > => push children
       * - /> => pop state
       */

      if is_identifier(first_char) {
        let mut name = first_char.to_string();
        lexer.get_next_char_while(&mut name, is_identifier);
    
        Ok(Token::Identifier(name))
      } else if first_char == '"' || first_char == '\'' {
        let mut value = String::new();
        lexer.get_next_char_while(&mut value, |c| c != first_char);
        lexer.raw_data.next();

        Ok(Token::Literal(Literal::Str(value)))
      } else {
        let symbol = read_symbol(lexer, &first_char);

        // <Component.Element<Pick<State, 'foo'> value="1" js={1} />
        if symbol == "=" || symbol == "-" || symbol == "." || symbol == "," {
          Ok(Token::Symbol(symbol))
        } else if symbol == "{" {
          lexer.state.push(LexerState::Typescript(TypescriptState {
            bracket_stack: 1,
            jsx_transition: JSXTransition::None,
          }));
  
          Ok(Token::Symbol(symbol))
        } else if symbol == ">" {
          if element_stack == 1 {
            // not covered
            lexer.replace_state(LexerState::Jsx(JSXState::Children));
          } else {
            // not covered
            lexer.replace_state(LexerState::Jsx(JSXState::Element(element_stack-1)));
          }
          Ok(Token::Symbol(symbol))
        } else if symbol == "<" {
          lexer.replace_state(LexerState::Jsx(JSXState::Element(element_stack+1)));

          Ok(Token::Symbol(symbol))
        } else if symbol == "/>" {
          // not covered
          lexer.state.pop();
  
          Ok(Token::Symbol(symbol))
        } else {
          Err(format!("Unkown token {}", symbol))
        }
      }
    },
    JSXState::Children => {
      /* Valid tokens are just a few:
       * some long text {123} <element />
       * - { => go typescript
       * - < => go Element
       * - anything else => string literal
       */

      if first_char == '{' {
        lexer.state.push(LexerState::Typescript(TypescriptState {
          bracket_stack: 1,
          jsx_transition: JSXTransition::None,
        }));

        Ok(Token::Symbol(String::from("{")))
      } else if first_char == '<' {
        let symbol = read_symbol(lexer, &first_char);

        if symbol == "<" {
          // not covered
          lexer.state.push(LexerState::Jsx(JSXState::Element(1)));
  
          Ok(Token::Symbol(String::from(symbol)))
        } else if symbol == "</" {
          lexer.replace_state(LexerState::Jsx(JSXState::Closing));
  
          Ok(Token::Symbol(String::from(symbol)))
        } else if symbol == "<>" {
          // not covered
          lexer.state.push(LexerState::Jsx(JSXState::Children));

          Ok(Token::Symbol(String::from(symbol)))
        } else if symbol == "</>" {
          // not covered
          lexer.state.pop();

          Ok(Token::Symbol(String::from(symbol)))
        } else {
          Err(format!("Unkown token {}", symbol))
        }
      } else {
        let mut value = String::from(first_char);
        lexer.get_next_char_while(&mut value, |c| c != '{' && c != '<');

        Ok(Token::Literal(Literal::Str(value)))
      }
    }
    JSXState::Closing => {
      /* We're just expecting to close:
       * </element.subelement>
       */

      if is_identifier(first_char) {
        let mut name = first_char.to_string();
        lexer.get_next_char_while(&mut name, is_identifier);
    
        Ok(Token::Identifier(name))
      } else if first_char == '.' {
        Ok(Token::Symbol(String::from(".")))
      } else if first_char == '>' {
        lexer.state.pop();
        Ok(Token::Symbol(String::from(">")))
      } else {
        panic!("Unknown token starting with {}", first_char)
      }
    }
  };

  return Some(Some(token));
}

fn read_symbol(lexer: &mut Lexer, first_char: &char) -> String {
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

  return raw;
}

fn is_identifier(c: char) -> bool {
  c.is_ascii_alphanumeric() || c == '_' || c == '$'
}
