use core::fmt::Debug;
use std::marker::PhantomData;
use std::ops::Deref;

/// Global ///

#[derive(Debug, PartialEq)]
pub enum MatcherResult<Token> {
  Rejected,
  End(MatchResultValue<Token>),
  Accepted,
  Value(MatchResultValue<Token>),
}

// impl<Token> MatcherResult<Token> {
//   pub fn value(self) -> Option<MatchResultValue<Token>> {
//     match self {
//       MatcherResult::End(v) => Some(v),
//       MatcherResult::Value(v) => Some(v),
//       _ => None,
//     }
//   }
// }

#[derive(Clone, Debug, PartialEq)]
pub enum MatchResultValue<Token> {
  Token(Token),
  Vector(Vec<MatchResultValue<Token>>),         // Loops
  Option(Option<Box<MatchResultValue<Token>>>), // Optionals
  Branch(usize, Box<MatchResultValue<Token>>),  // OneOf
}

#[derive(Clone)]
pub enum MatcherType<Token> {
  OneOf(OneOf<Token>),
  Sequence(Sequence<Token>),
  Loop(Loop<Token>),
  Terminal(Terminal<Token>),
  Optional(Optional<Token>),
  _Marker(PhantomData<Token>),
}

impl<Token: Clone + Debug> MatcherType<Token> {
  pub fn reset(&mut self) {
    match self {
      MatcherType::OneOf(v) => v.reset(),
      MatcherType::Sequence(v) => v.reset(),
      MatcherType::Loop(v) => v.reset(),
      MatcherType::Terminal(v) => v.reset(),
      MatcherType::Optional(v) => v.reset(),
      _ => {}
    }
  }
  pub fn next(&mut self, token: &Token) -> MatcherResult<Token> {
    match self {
      MatcherType::OneOf(v) => v.next(token),
      MatcherType::Sequence(v) => v.next(token),
      MatcherType::Loop(v) => v.next(token),
      MatcherType::Terminal(v) => v.next(token),
      MatcherType::Optional(v) => v.next(token),
      _ => MatcherResult::Rejected,
    }
  }
}

pub trait Matcher<Token> {
  fn next(&mut self, token: &Token) -> MatcherResult<Token>;
  fn reset(&mut self);
}

/// OneOf ///
#[derive(Clone)]
pub struct OneOf<Token> {
  matchers: Vec<MatcherType<Token>>,
}

impl<Token> OneOf<Token> {
  pub fn new(matchers: Vec<MatcherType<Token>>) -> Self {
    Self { matchers }
  }
  pub fn matcher(matchers: Vec<MatcherType<Token>>) -> MatcherType<Token> {
    MatcherType::OneOf(OneOf::new(matchers))
  }
}

impl<Token: Clone + Debug> Matcher<Token> for OneOf<Token> {
  fn next(&mut self, token: &Token) -> MatcherResult<Token> {
    let mut result: Option<(usize, MatchResultValue<Token>)> = None;
    let mut has_accepted = false;

    let length = self.matchers.len();
    for i in 0..length {
      let matcher = &mut self.matchers[i];
      match (matcher.next(&token), &result) {
        (MatcherResult::Accepted, _) => {
          has_accepted = true;
        }
        (MatcherResult::End(r), None) => {
          result = Some((i, r.clone()));
        }
        (MatcherResult::Value(v), None) => {
          has_accepted = true;
          result = Some((i, v.clone()));
        }
        (MatcherResult::Value(_), Some(_)) => {
          has_accepted = true;
        }
        _ => {}
      }
    }

    match &mut result {
      Some((i, v)) => {
        let value = MatchResultValue::Branch(*i, Box::new(v.clone()));
        if has_accepted {
          MatcherResult::Value(value)
        } else {
          MatcherResult::End(value)
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

/// Sequence ///
#[derive(Clone)]
pub struct Sequence<Token> {
  sequence_matchers: Vec<SequenceMatcher<Token>>,
}
#[derive(Clone)]
struct SequenceMatcher<Token> {
  matcher: MatcherType<Token>,
  result: Option<MatchResultValue<Token>>,
  is_head: bool,
}

impl<Token: Clone + Debug> Sequence<Token> {
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
    result.propagate_optional_heads();
    return result;
  }

  pub fn matcher(matchers: Vec<MatcherType<Token>>) -> MatcherType<Token> {
    MatcherType::Sequence(Self::new(matchers))
  }

  fn propagate_optional_heads(&mut self) {
    for i in 1..self.sequence_matchers.len() {
      let prev = &self.sequence_matchers[i - 1];
      if prev.is_head && matches!(prev.matcher, MatcherType::Optional(_)) {
        self.sequence_matchers[i].is_head = true;
      }
    }
  }
}

impl<Token: Clone + Debug> SequenceMatcher<Token> {
  fn reset(&mut self) {
    self.is_head = false;
    self.matcher.reset();
    self.result = if matches!(self.matcher, MatcherType::Optional(_)) {
      Some(MatchResultValue::Option(None))
    } else {
      None
    }
  }
}

impl<Token: Clone + Debug> Matcher<Token> for Sequence<Token> {
  fn next(&mut self, token: &Token) -> MatcherResult<Token> {
    let has_head = self.sequence_matchers.iter().any(|m| m.is_head);
    if !has_head {
      return MatcherResult::Rejected;
    }

    let mut has_updated = false;
    for i in (0..self.sequence_matchers.len()).rev() {
      let sequence_matcher = &mut self.sequence_matchers[i];
      if !sequence_matcher.is_head {
        continue;
      }

      match sequence_matcher.matcher.next(token) {
        MatcherResult::Rejected => {
          sequence_matcher.is_head = false;
        }
        MatcherResult::End(r)
          if matches!(sequence_matcher.matcher, MatcherType::Optional(_))
            && matches!(r, MatchResultValue::Option(None)) =>
        {
          sequence_matcher.is_head = false;
        }
        MatcherResult::End(r) => {
          has_updated = true;
          sequence_matcher.result = Some(r);
          sequence_matcher.is_head = false;
          // We have a new value: Reset all following matchers
          for j in (i + 1)..self.sequence_matchers.len() {
            self.sequence_matchers[j].reset();
          }
          // Then set head the next one
          if i + 1 < self.sequence_matchers.len() {
            self.sequence_matchers[i + 1].is_head = true;
          }
        }
        MatcherResult::Value(v) => {
          has_updated = true;
          sequence_matcher.result = Some(v);
          // Same as before, but keeping this as head.
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

    self.propagate_optional_heads();

    let has_head = self.sequence_matchers.iter().any(|m| m.is_head);
    let is_complete = !self
      .sequence_matchers
      .iter()
      .any(|m| matches!(m.result, None));
    // println!("has_head: {}, is_complete: {}", has_head, is_complete);
    if is_complete && has_updated {
      let result: MatchResultValue<Token> = MatchResultValue::Vector(
        self
          .sequence_matchers
          .iter()
          .map(|m| (&m.result).as_ref().unwrap().clone())
          .collect(),
      );
      if has_head {
        // println!("value {:?}", result);
        MatcherResult::Value(result)
      } else {
        // println!("end {:?}", result);
        MatcherResult::End(result)
      }
    } else {
      if has_head {
        // println!("accepted");
        MatcherResult::Accepted
      } else {
        // println!("rejected");
        MatcherResult::Rejected
      }
    }
  }
  fn reset(&mut self) {
    for sequence_matcher in &mut self.sequence_matchers {
      sequence_matcher.reset();
    }
    self.sequence_matchers[0].is_head = true;
    self.propagate_optional_heads()
  }
}

/// Loop ///
#[derive(Clone)]
pub struct Loop<Token> {
  original: Box<MatcherType<Token>>,
  matchers: Vec<SequenceMatcher<Token>>,
}

impl<Token: Clone> Loop<Token> {
  pub fn new(matcher: MatcherType<Token>) -> Self {
    Self {
      original: Box::new(matcher.clone()),
      matchers: vec![SequenceMatcher {
        matcher,
        is_head: true,
        result: None,
      }],
    }
  }
  pub fn matcher(matcher: MatcherType<Token>) -> MatcherType<Token> {
    MatcherType::Loop(Loop::new(matcher))
  }
}

impl<Token: Clone + Debug> Matcher<Token> for Loop<Token> {
  fn next(&mut self, token: &Token) -> MatcherResult<Token> {
    for i in 0..self.matchers.len() {
      let matcher_state = &mut (self.matchers[i]);
      let matcher = &mut (matcher_state.matcher);
      let is_head = matcher_state.is_head;

      if !is_head {
        continue;
      }

      match matcher.next(token) {
        MatcherResult::Rejected => {
          matcher_state.is_head = false;
        }
        MatcherResult::Accepted => {}
        MatcherResult::Value(v) => {
          matcher_state.result = Some(v);
          self.matchers.truncate(i + 1);
          let result = MatcherResult::Value(MatchResultValue::Vector(
            (&self.matchers)
              .into_iter()
              .map(|matcher_state| matcher_state.result.as_ref().unwrap().clone())
              .collect(),
          ));

          self.matchers.push(SequenceMatcher {
            matcher: self.original.deref().clone(),
            is_head: true,
            result: None,
          });

          return result;
        }
        MatcherResult::End(v) => {
          matcher_state.is_head = false;
          matcher_state.result = Some(v.clone());
          self.matchers.truncate(i + 1);
          let result = MatcherResult::Value(MatchResultValue::Vector(
            (&self.matchers)
              .into_iter()
              .map(|matcher_state| matcher_state.result.as_ref().unwrap().clone())
              .collect(),
          ));

          self.matchers.push(SequenceMatcher {
            matcher: self.original.deref().clone(),
            is_head: true,
            result: None,
          });

          return result;
        }
      }
    }

    let has_head = self.matchers.iter().any(|m| m.is_head);

    if has_head {
      MatcherResult::Accepted
    } else {
      MatcherResult::Rejected
    }
  }
  fn reset(&mut self) {
    self.matchers = vec![SequenceMatcher {
      matcher: self.original.deref().clone(),
      is_head: true,
      result: None,
    }]
  }
}

/// Terminal ///
#[derive(Clone)]
pub struct Terminal<Token> {
  match_fn: fn(&Token) -> bool,
  executed: bool,
}

impl<Token: Clone + Debug> Terminal<Token> {
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

impl<Token: Clone + Debug> Matcher<Token> for Terminal<Token> {
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

// impl<'a, Token: Clone + Debug> Matcher<Token> for &'a mut Terminal<Token> {
//   fn next(&mut self, token: &Token) -> MatcherResult<Token> {
//     Terminal::next(*self, token)
//   }
//   fn reset(&mut self) {
//     Terminal::reset(*self)
//   }
// }

/// Optional ///
#[derive(Clone)]
pub struct Optional<Token> {
  matcher: Box<MatcherType<Token>>,
  has_emitted: bool,
}

impl<Token: Clone + Debug> Optional<Token> {
  pub fn new(matcher: MatcherType<Token>) -> Self {
    Self {
      matcher: Box::new(matcher),
      has_emitted: false,
    }
  }
  pub fn matcher(matcher: MatcherType<Token>) -> MatcherType<Token> {
    MatcherType::Optional(Optional::new(matcher))
  }
}

impl<Token: Clone + Debug> Matcher<Token> for Optional<Token> {
  fn next(&mut self, token: &Token) -> MatcherResult<Token> {
    let result = self.matcher.next(token);

    match result {
      MatcherResult::Rejected => {
        if self.has_emitted {
          MatcherResult::Rejected
        } else {
          MatcherResult::End(MatchResultValue::Option(None))
        }
      }
      MatcherResult::Accepted => MatcherResult::Accepted,
      MatcherResult::Value(v) => {
        self.has_emitted = true;
        MatcherResult::Value(MatchResultValue::Option(Some(Box::new(v))))
      }
      MatcherResult::End(v) => {
        self.has_emitted = true;
        MatcherResult::End(MatchResultValue::Option(Some(Box::new(v))))
      }
    }
  }
  fn reset(&mut self) {
    self.matcher.reset();
    self.has_emitted = false;
  }
}

#[macro_export]
macro_rules! unwrap_enum {
  ( $r:expr, $m:path ) => {{
    (match &$r {
      $m(v) => Some(v),
      _ => None,
    })
    .unwrap()
  }};
}

#[macro_export]
macro_rules! unwrap_branch {
  ( $r:expr ) => {{
    (match &$r {
      MatchResultValue::Branch(v, t) => Some((v, t)),
      _ => None,
    })
    .unwrap()
  }};
}

#[macro_export]
macro_rules! unwrap_match {
  ( $r:expr, $p: pat => $v: expr ) => {{
    (match &$r {
      $p => Some($v),
      _ => None,
    })
    .unwrap()
  }};
}
