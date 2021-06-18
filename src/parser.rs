use std::collections::HashMap;
use std::marker::PhantomData;

/**
 * Idea I have is that you can define a parser as with a set of chained rules:
 *
 * import statement:
 * - sequence // <- one-by-one
 *  - keyword("import")
 *  - oneOf // <-
 *   - sequence
 *    - identifier
 *    - optional
 *     - sequence
 *      - symbol(",")
 *      - {namedImports}
 *   - {namedImports}
 *   - sequence
 *    - symbol("*")
 *    - keyword("as")
 *    - identifier
 *  - keyword("from")
 *  - literal(str)
 *
 * namedImports:
 * - sequence
 *  - symbol("{")
 *  - optional
 *   - sequence
 *    - identifier
 *    - optional
 *     - symbol(":")
 *     - identifier
 *   - loop (loops can also take 0) <=== Q: how does a loop decide to begin a new loop? On the first value? but then it won't work with optional below! and if not, how does the outer loop(loop(...)) behave?
 *    - sequence
 *     - symbol(",")
 *     - identifier
 *     - optional <=== TODO this one is hard, because it effects sequence, or not straight forward. Otherwise transform to oneOf(sequence(",", identifier), sequence(",", identifier, ":", identifier))
 *      - symbol(":")
 *      - identifier
 *  - symbol("}")
 *
 * And then give you whatever has been captured in each step, or None if it didn't match
 *
 * Maybe it would be nice to go step-by-step. So you "feed" it a token, and it can give you:
 * - Rejected -> end state, it didn't match anything
 * - End([value]) -> end state, it matched something
 * - Accepted -> accepted
 * - Value([value]) -> accepted and it has a value, but can take more tokens (loops or optionals)
 *
 * To backtrack, maybe have an utility "split" which feeds each token to every posibility, and keeps track of the aggregated state.
 */

#[derive(Debug)]
pub enum MatcherResult<Token> {
  Rejected,
  End(MatchResultValue<Token>),
  Accepted,
  Value(MatchResultValue<Token>),
}

impl<Token> MatcherResult<Token> {
  pub fn value(self) -> Option<MatchResultValue<Token>> {
    match self {
      MatcherResult::End(v) => Some(v),
      MatcherResult::Value(v) => Some(v),
      _ => None,
    }
  }
}

#[derive(Debug)]
pub enum MatchResultValue<Token> {
  String(String),
  Number(i32),
  Token(Token),
  Map(HashMap<String, MatchResultValue<Token>>),
  Vector(Vec<MatchResultValue<Token>>),
}

impl<Token: Clone> Clone for MatchResultValue<Token> {
  fn clone(&self) -> MatchResultValue<Token> {
    match self {
      MatchResultValue::String(s) => MatchResultValue::String(String::from(s)),
      MatchResultValue::Number(v) => MatchResultValue::Number(*v),
      MatchResultValue::Token(v) => MatchResultValue::Token(v.clone()),
      MatchResultValue::Map(v) => MatchResultValue::Map(v.clone()),
      MatchResultValue::Vector(v) => MatchResultValue::Vector(v.clone()),
    }
  }
}

/// Possiblity 1 ///

// enum MatcherDef<Token> {
//   Sequence(Vec<MatcherDef<Token>>),
//   Optional(Box<MatcherDef<Token>>),
//   Loop(Box<MatcherDef<Token>>),
//   OneOf(Vec<MatcherDef<Token>>),
//   MatchFn(fn(Token) -> Option<MatchResult<Token>>),
//   Matcher(Box<Matcher<Token>>),
// }

// struct Matcher<Token> {
//   matcherDef: MatcherDef<Token>,
// }

/// Possiblity 2 ///

pub enum MatcherType<Token> {
  OneOf(OneOf<Token>),
  Sequence(Sequence<Token>),
  Terminal(Terminal<Token>),
  _Marker(PhantomData<Token>),
}

impl<Token: Clone> MatcherType<Token> {
  fn reset(&mut self) {
    match self {
      // MatcherType::OneOf(v) => v.reset(),
      MatcherType::Sequence(v) => v.reset(),
      MatcherType::Terminal(v) => v.reset(),
      _ => {}
    }
  }
  fn next(&mut self, token: &Token) -> MatcherResult<Token> {
    match self {
      // MatcherType::OneOf(v) => v.next(token),
      MatcherType::Sequence(v) => v.next(token),
      MatcherType::Terminal(v) => v.next(token),
      _ => MatcherResult::Rejected,
    }
  }
}

trait Matcher<Token> {
  fn next(&mut self, token: &Token) -> MatcherResult<Token>;
  fn reset(&mut self);
}

struct OneOf<Token> {
  matchers: Vec<MatcherType<Token>>,
}
impl<Token: Clone> Matcher<Token> for OneOf<Token> {
  fn next(&mut self, token: &Token) -> MatcherResult<Token> {
    let mut result: Option<MatchResultValue<Token>> = None;
    let mut has_accepted = false;

    for matcher in &mut self.matchers {
      match (matcher.next(&token), &result) {
        (MatcherResult::Accepted, _) => {
          has_accepted = true;
        }
        (MatcherResult::End(r), None) => {
          result = Some(r.clone());
        }
        (MatcherResult::Value(v), None) => {
          has_accepted = true;
          result = Some(v.clone());
        }
        (MatcherResult::Value(_), Some(_)) => {
          has_accepted = true;
        }
        _ => {}
      }
    }

    match &mut result {
      Some(v) => {
        if has_accepted {
          MatcherResult::Value(v.clone())
        } else {
          MatcherResult::End(v.clone())
        }
      }
      None => {
        if has_accepted {
          MatcherResult::Accepted
        } else {
          MatcherResult::Rejected
        }
      }
    }
  }
  fn reset(&mut self) {
    for matcher in &mut self.matchers {
      matcher.reset();
    }
  }
}

pub struct Sequence<Token> {
  sequence_matchers: Vec<SequenceMatcher<Token>>,
}
struct SequenceMatcher<Token> {
  matcher: MatcherType<Token>,
  result: Option<MatchResultValue<Token>>,
  is_head: bool,
}

impl<Token: Clone> Sequence<Token> {
  pub fn new(matchers: Vec<MatcherType<Token>>) -> Self {
    let mut result = Sequence {
      sequence_matchers: matchers
        .into_iter()
        .map(|matcher| SequenceMatcher {
          matcher,
          result: None,
          is_head: false,
        })
        .collect(),
    };
    result.sequence_matchers[0].is_head = true;
    return result;
  }

  pub fn next2(&mut self, token: &Token) -> MatcherResult<Token> {
    self.next(token)
  }
}

impl<Token: Clone> SequenceMatcher<Token> {
  fn reset(&mut self) {
    self.is_head = false;
    self.result = None;
    self.matcher.reset();
  }
}

impl<Token: Clone> Matcher<Token> for Sequence<Token> {
  fn next(&mut self, token: &Token) -> MatcherResult<Token> {
    let has_head = self.sequence_matchers.iter().any(|m| m.is_head);
    if !has_head {
      return MatcherResult::Rejected;
    }

    for i in (0..self.sequence_matchers.len()).rev() {
      let sequence_matcher = &mut self.sequence_matchers[i];
      if !sequence_matcher.is_head {
        continue;
      }

      match sequence_matcher.matcher.next(token) {
        MatcherResult::Rejected => {
          sequence_matcher.is_head = false;
        }
        MatcherResult::End(r) => {
          sequence_matcher.result = Some(r);
          sequence_matcher.is_head = false;
          for j in (i + 1)..self.sequence_matchers.len() {
            self.sequence_matchers[j].reset();
          }
          if i + 1 < self.sequence_matchers.len() {
            self.sequence_matchers[i + 1].is_head = true;
          }
        }
        MatcherResult::Value(v) => {
          sequence_matcher.result = Some(v);
          for j in (i + 1)..self.sequence_matchers.len() {
            self.sequence_matchers[j].reset();
          }
          if i + 1 < self.sequence_matchers.len() {
            self.sequence_matchers[i + 1].is_head = true;
          }
        }
        _ => {}
      }
    }

    let has_head = self.sequence_matchers.iter().any(|m| m.is_head);
    let last_matcher = &mut self.sequence_matchers.last().unwrap();
    match &last_matcher.result {
      Some(_) => {
        let result: MatchResultValue<Token> = MatchResultValue::Vector(
          self
            .sequence_matchers
            .iter()
            .map(|m| (&m.result).as_ref().unwrap().clone())
            .collect(),
        );
        if has_head {
          MatcherResult::Value(result)
        } else {
          MatcherResult::End(result)
        }
      }
      None => {
        if has_head {
          MatcherResult::Accepted
        } else {
          MatcherResult::Rejected
        }
      }
    }
  }
  fn reset(&mut self) {
    for sequence_matcher in &mut self.sequence_matchers {
      sequence_matcher.reset();
    }
    self.sequence_matchers[0].is_head = true;
  }
}

pub struct Terminal<Token> {
  match_fn: fn(&Token) -> bool,
  executed: bool,
}

impl<Token> Terminal<Token> {
  pub fn new(match_fn: fn(&Token) -> bool) -> Self {
    Self {
      match_fn,
      executed: false,
    }
  }
  pub fn matcher(match_fn: fn(&Token) -> bool) -> MatcherType<Token> {
    MatcherType::Terminal(Terminal::new(match_fn))
  }
}

impl<Token: Clone> Matcher<Token> for Terminal<Token> {
  fn next(&mut self, token: &Token) -> MatcherResult<Token> {
    if self.executed {
      return MatcherResult::Rejected;
    }
    self.executed = true;
    if (self.match_fn)(token) {
      MatcherResult::End(MatchResultValue::Token(token.clone()))
    } else {
      MatcherResult::Rejected
    }
  }
  fn reset(&mut self) {
    self.executed = false
  }
}

#[macro_export]
macro_rules! unwrap_enum {
  ( $r:expr, $m:path ) => {{
    (match $r {
      $m(v) => Some(v),
      _ => None,
    })
    .unwrap()
  }};
}
