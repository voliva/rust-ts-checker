#[cfg(test)]
mod parser_tests {
  // Note this useful idiom: importing names from outer (for mod tests) scope.
  use crate::parser::{
    Loop, MatchResultValue, MatcherResult, MatcherType, OneOf, Optional, Sequence, Terminal,
  };

  /// Terminal
  #[test]
  fn terminal_accepts_value_then_rejects() {
    let mut parser = Terminal::matcher(|token: &char| *token == 'a');

    run_test(
      &mut parser,
      "aaa",
      vec![
        MatcherResult::End(MatchResultValue::Token('a')),
        MatcherResult::Rejected,
        MatcherResult::Rejected,
      ],
    );
  }

  #[test]
  fn terminal_doesnt_accept_after_rejecting() {
    let mut parser = Terminal::matcher(|token: &char| *token == 'a');

    run_test(
      &mut parser,
      "ba",
      vec![MatcherResult::Rejected, MatcherResult::Rejected],
    );
  }

  #[test]
  fn terminal_resets_its_state() {
    let mut parser = Terminal::matcher(|token: &char| *token == 'a');

    run_test(
      &mut parser,
      "aa",
      vec![
        MatcherResult::End(MatchResultValue::Token('a')),
        MatcherResult::Rejected,
      ],
    );

    parser.reset();

    run_test(
      &mut parser,
      "aa",
      vec![
        MatcherResult::End(MatchResultValue::Token('a')),
        MatcherResult::Rejected,
      ],
    );
  }

  /// Sequence
  #[test]
  fn sequence_accepts_values_one_by_one() {
    let mut parser = Sequence::matcher(vec![
      Terminal::matcher(|token: &char| *token == 'a'),
      Terminal::matcher(|token: &char| *token == 'b'),
      Terminal::matcher(|token: &char| *token == 'c'),
    ]);

    run_test(
      &mut parser,
      "abcd",
      vec![
        MatcherResult::Accepted,
        MatcherResult::Accepted,
        MatcherResult::End(MatchResultValue::Vector(vec![
          MatchResultValue::Token('a'),
          MatchResultValue::Token('b'),
          MatchResultValue::Token('c'),
        ])),
        MatcherResult::Rejected,
      ],
    );
  }

  #[test]
  fn sequence_doesnt_accept_after_rejecting() {
    let mut parser = Sequence::matcher(vec![
      Terminal::matcher(|token: &char| *token == 'a'),
      Terminal::matcher(|token: &char| *token == 'b'),
      Terminal::matcher(|token: &char| *token == 'c'),
    ]);

    run_test(
      &mut parser,
      "adbc",
      vec![
        MatcherResult::Accepted,
        MatcherResult::Rejected,
        MatcherResult::Rejected,
        MatcherResult::Rejected,
      ],
    );
  }

  #[test]
  fn sequence_resets_its_state() {
    let mut parser = Sequence::matcher(vec![
      Terminal::matcher(|token: &char| *token == 'a'),
      Terminal::matcher(|token: &char| *token == 'b'),
      Terminal::matcher(|token: &char| *token == 'c'),
    ]);

    run_test(&mut parser, "a", vec![MatcherResult::Accepted]);

    parser.reset();

    run_test(
      &mut parser,
      "abc",
      vec![
        MatcherResult::Accepted,
        MatcherResult::Accepted,
        MatcherResult::End(MatchResultValue::Vector(vec![
          MatchResultValue::Token('a'),
          MatchResultValue::Token('b'),
          MatchResultValue::Token('c'),
        ])),
      ],
    );
  }

  /// OneOf
  #[test]
  fn one_of_ends_when_one_of_inners_matches() {
    let mut parser = OneOf::matcher(vec![
      Terminal::matcher(|token: &char| *token == 'a'),
      Terminal::matcher(|token: &char| *token == 'b'),
    ]);

    run_test(
      &mut parser,
      "aba",
      vec![
        MatcherResult::End(MatchResultValue::Branch(
          0,
          Box::new(MatchResultValue::Token('a')),
        )),
        MatcherResult::Rejected,
        MatcherResult::Rejected,
      ],
    );
  }

  #[test]
  fn one_of_doesnt_accept_after_rejecting() {
    let mut parser = OneOf::matcher(vec![
      Terminal::matcher(|token: &char| *token == 'a'),
      Terminal::matcher(|token: &char| *token == 'b'),
    ]);

    run_test(
      &mut parser,
      "ca",
      vec![MatcherResult::Rejected, MatcherResult::Rejected],
    );
  }

  #[test]
  fn one_of_resets_its_state() {
    let mut parser = OneOf::matcher(vec![
      Terminal::matcher(|token: &char| *token == 'a'),
      Terminal::matcher(|token: &char| *token == 'b'),
    ]);

    run_test(
      &mut parser,
      "a",
      vec![MatcherResult::End(MatchResultValue::Branch(
        0,
        Box::new(MatchResultValue::Token('a')),
      ))],
    );

    parser.reset();

    run_test(
      &mut parser,
      "b",
      vec![MatcherResult::End(MatchResultValue::Branch(
        1,
        Box::new(MatchResultValue::Token('b')),
      ))],
    );
  }

  #[test]
  fn one_of_doesnt_change_branch_after_reject() {
    let mut parser = OneOf::matcher(vec![
      Sequence::matcher(vec![
        Terminal::matcher(|token: &char| *token == 'a'),
        Terminal::matcher(|token: &char| *token == 'b'),
        Terminal::matcher(|token: &char| *token == 'c'),
      ]),
      Terminal::matcher(|token: &char| *token == 'd'),
    ]);

    run_test(
      &mut parser,
      "abdc",
      vec![
        MatcherResult::Accepted,
        MatcherResult::Accepted,
        MatcherResult::Rejected,
        MatcherResult::Rejected,
      ],
    );
  }

  #[test]
  fn one_of_changes_branch_when_overlap() {
    let mut parser = OneOf::matcher(vec![
      Terminal::matcher(|token: &char| *token == 'a'),
      Sequence::matcher(vec![
        Terminal::matcher(|token: &char| *token == 'a'),
        Terminal::matcher(|token: &char| *token == 'b'),
      ]),
    ]);

    run_test(
      &mut parser,
      "ab",
      vec![
        MatcherResult::Value(MatchResultValue::Branch(
          0,
          Box::new(MatchResultValue::Token('a')),
        )),
        MatcherResult::End(MatchResultValue::Branch(
          1,
          Box::new(MatchResultValue::Vector(vec![
            MatchResultValue::Token('a'),
            MatchResultValue::Token('b'),
          ])),
        )),
      ],
    );
  }

  /// Loop
  #[test]
  fn loop_keeps_adding_values_until_reject() {
    let mut parser = Loop::matcher(Sequence::matcher(vec![
      Terminal::matcher(|token: &char| *token == 'a'),
      Terminal::matcher(|token: &char| *token == 'b'),
    ]));

    run_test(
      &mut parser,
      "ababcab",
      vec![
        MatcherResult::Accepted,
        MatcherResult::Value(MatchResultValue::Vector(vec![MatchResultValue::Vector(
          vec![MatchResultValue::Token('a'), MatchResultValue::Token('b')],
        )])),
        MatcherResult::Accepted,
        MatcherResult::Value(MatchResultValue::Vector(vec![
          MatchResultValue::Vector(vec![
            MatchResultValue::Token('a'),
            MatchResultValue::Token('b'),
          ]),
          MatchResultValue::Vector(vec![
            MatchResultValue::Token('a'),
            MatchResultValue::Token('b'),
          ]),
        ])),
        MatcherResult::Rejected,
        MatcherResult::Rejected,
        MatcherResult::Rejected,
      ],
    );
  }

  #[test]
  fn loop_accepts_only_first_value_from_inner() {
    let mut parser = Loop::matcher(Sequence::matcher(vec![
      Loop::matcher(Sequence::matcher(vec![
        Terminal::matcher(|token: &char| *token == 'a'),
        Terminal::matcher(|token: &char| *token == 'b'),
      ])),
      Terminal::matcher(|token: &char| *token == 'a'),
    ]));

    /*
    This test documents that the inner loop is useless here, it can't be used.
    As soon as a new loop would start, the parent sequence completes, and the
    outer loop restarts the inner one.

    The regex of this one would be `((ab)+a)+`, and Loop can't solve it as of now.
    Making a Loop that can solve this would mean that it needs to add multiple
    heads, and it gets quite complex to wrap around this concept.

    It is possible to write a matcher that has the same behavior: if we look at
    an example of what would be accepted:
    - ababaababaaba...
    and not accepted:
    - _babaababaaba... => Must start with a
    - abaaababbabab... => Can't have 3 consecutive a's, can't have 2 consecutive b's
    => Must end with a

    then probably something like this would work:

    a(baa?)+

    Sequence(
      Terminal(a),
      Loop(Sequence(
        Terminal(b),
        Terminal(a),
        Optional(Terminal(a))
      ))
    )
    or
    Sequence(
      Terminal(a),
      Loop(OneOf(
        Sequence(
          Terminal(b),
          Terminal(a)
        ),
        Sequence(
          Terminal(b),
          Terminal(a),
          Terminal(a)
        )
      )
    ) // All Optionals can be unwrapped with OneOf
    */

    run_test(
      &mut parser,
      "ababa",
      vec![
        MatcherResult::Accepted,
        MatcherResult::Accepted,
        MatcherResult::Value(MatchResultValue::Vector(vec![MatchResultValue::Vector(
          vec![
            MatchResultValue::Vector(vec![MatchResultValue::Vector(vec![
              MatchResultValue::Token('a'),
              MatchResultValue::Token('b'),
            ])]),
            MatchResultValue::Token('a'),
          ],
        )])),
        MatcherResult::Rejected,
        MatcherResult::Rejected,
      ],
    );

    parser.reset();

    run_test(
      &mut parser,
      "abaaba",
      vec![
        MatcherResult::Accepted,
        MatcherResult::Accepted,
        MatcherResult::Value(MatchResultValue::Vector(vec![MatchResultValue::Vector(
          vec![
            MatchResultValue::Vector(vec![MatchResultValue::Vector(vec![
              MatchResultValue::Token('a'),
              MatchResultValue::Token('b'),
            ])]),
            MatchResultValue::Token('a'),
          ],
        )])),
        MatcherResult::Accepted,
        MatcherResult::Accepted,
        MatcherResult::Value(MatchResultValue::Vector(vec![
          MatchResultValue::Vector(vec![
            MatchResultValue::Vector(vec![MatchResultValue::Vector(vec![
              MatchResultValue::Token('a'),
              MatchResultValue::Token('b'),
            ])]),
            MatchResultValue::Token('a'),
          ]),
          MatchResultValue::Vector(vec![
            MatchResultValue::Vector(vec![MatchResultValue::Vector(vec![
              MatchResultValue::Token('a'),
              MatchResultValue::Token('b'),
            ])]),
            MatchResultValue::Token('a'),
          ]),
        ])),
      ],
    );
  }

  /// Optional
  #[test]
  fn sequence_with_optional_accepts_two_outcomes() {
    let mut parser = Sequence::matcher(vec![
      Terminal::matcher(|token: &char| *token == 'a'),
      Optional::matcher(Terminal::matcher(|token: &char| *token == 'b')),
      Terminal::matcher(|token: &char| *token == 'c'),
    ]);

    run_test(
      &mut parser,
      "acbc",
      vec![
        MatcherResult::Accepted,
        MatcherResult::End(MatchResultValue::Vector(vec![
          MatchResultValue::Token('a'),
          MatchResultValue::Option(None),
          MatchResultValue::Token('c'),
        ])),
        MatcherResult::Rejected,
        MatcherResult::Rejected,
      ],
    );

    parser.reset();

    run_test(
      &mut parser,
      "abca",
      vec![
        MatcherResult::Accepted,
        MatcherResult::Accepted,
        MatcherResult::End(MatchResultValue::Vector(vec![
          MatchResultValue::Token('a'),
          MatchResultValue::Option(Some(Box::new(MatchResultValue::Token('b')))),
          MatchResultValue::Token('c'),
        ])),
        MatcherResult::Rejected,
      ],
    );
  }

  #[test]
  fn optional_takes_preference_over_next_sequence() {
    let mut parser = Sequence::matcher(vec![
      Terminal::matcher(|token: &char| *token == 'a'),
      Optional::matcher(Sequence::matcher(vec![
        Terminal::matcher(|token: &char| *token == 'b'),
        Terminal::matcher(|token: &char| *token == 'c'),
      ])),
      Terminal::matcher(|token: &char| *token == 'b'),
      Terminal::matcher(|token: &char| *token == 'c'),
    ]);

    run_test(
      &mut parser,
      "abcbcd",
      vec![
        MatcherResult::Accepted,
        MatcherResult::Accepted,
        MatcherResult::Accepted,
        MatcherResult::Accepted,
        MatcherResult::End(MatchResultValue::Vector(vec![
          MatchResultValue::Token('a'),
          MatchResultValue::Option(Some(Box::new(MatchResultValue::Vector(vec![
            MatchResultValue::Token('b'),
            MatchResultValue::Token('c'),
          ])))),
          MatchResultValue::Token('b'),
          MatchResultValue::Token('c'),
        ])),
        MatcherResult::Rejected,
      ],
    );
  }

  #[test]
  fn sequence_emits_value_only_when_option_complete() {
    let mut parser = Sequence::matcher(vec![
      Terminal::matcher(|token: &char| *token == 'a'),
      Optional::matcher(Sequence::matcher(vec![
        Terminal::matcher(|token: &char| *token == 'b'),
        Terminal::matcher(|token: &char| *token == 'c'),
      ])),
    ]);

    run_test(
      &mut parser,
      "abc",
      vec![
        MatcherResult::Value(MatchResultValue::Vector(vec![
          MatchResultValue::Token('a'),
          MatchResultValue::Option(None),
        ])),
        MatcherResult::Accepted, // Failing: sends `Value` with same value as previous
        MatcherResult::End(MatchResultValue::Vector(vec![
          MatchResultValue::Token('a'),
          MatchResultValue::Option(Some(Box::new(MatchResultValue::Vector(vec![
            MatchResultValue::Token('b'),
            MatchResultValue::Token('c'),
          ])))),
        ])),
      ],
    );
  }

  #[test]
  fn sequence_emits_rejected_after_option_rejects() {
    let mut parser = Sequence::matcher(vec![
      Terminal::matcher(|token: &char| *token == 'a'),
      Optional::matcher(Sequence::matcher(vec![Terminal::matcher(
        |token: &char| *token == 'b',
      )])),
    ]);

    run_test(
      &mut parser,
      "ac",
      vec![
        MatcherResult::Value(MatchResultValue::Vector(vec![
          MatchResultValue::Token('a'),
          MatchResultValue::Option(None),
        ])),
        MatcherResult::Rejected, // Failing: sends `End` with same value as previous
      ],
    );
  }

  #[test]
  fn sequence_emits_value_only_when_loop_emits_new_value() {
    let mut parser = Sequence::matcher(vec![
      Terminal::matcher(|token: &char| *token == 'a'),
      Loop::matcher(Sequence::matcher(vec![
        Terminal::matcher(|token: &char| *token == 'b'),
        Terminal::matcher(|token: &char| *token == 'c'),
      ])),
    ]);

    run_test(
      &mut parser,
      "abcbc",
      vec![
        MatcherResult::Accepted,
        MatcherResult::Accepted,
        MatcherResult::Value(MatchResultValue::Vector(vec![
          MatchResultValue::Token('a'),
          MatchResultValue::Vector(vec![MatchResultValue::Vector(vec![
            MatchResultValue::Token('b'),
            MatchResultValue::Token('c'),
          ])]),
        ])),
        MatcherResult::Accepted, // Fails: emits Value with the same as previous
        MatcherResult::Value(MatchResultValue::Vector(vec![
          MatchResultValue::Token('a'),
          MatchResultValue::Vector(vec![
            MatchResultValue::Vector(vec![
              MatchResultValue::Token('b'),
              MatchResultValue::Token('c'),
            ]),
            MatchResultValue::Vector(vec![
              MatchResultValue::Token('b'),
              MatchResultValue::Token('c'),
            ]),
          ]),
        ])),
      ],
    );
  }

  #[test]
  fn sequence_emits_rejected_after_loop_rejects() {
    let mut parser = Sequence::matcher(vec![
      Terminal::matcher(|token: &char| *token == 'a'),
      Loop::matcher(Terminal::matcher(|token: &char| *token == 'b')),
    ]);

    run_test(
      &mut parser,
      "abbc",
      vec![
        MatcherResult::Accepted,
        MatcherResult::Value(MatchResultValue::Vector(vec![
          MatchResultValue::Token('a'),
          MatchResultValue::Vector(vec![MatchResultValue::Token('b')]),
        ])),
        MatcherResult::Value(MatchResultValue::Vector(vec![
          MatchResultValue::Token('a'),
          MatchResultValue::Vector(vec![
            MatchResultValue::Token('b'),
            MatchResultValue::Token('b'),
          ]),
        ])),
        MatcherResult::Rejected, // Failing: sends `End` with same value as previous
      ],
    );
  }

  /// Utils
  fn run_test(matcher: &mut MatcherType<char>, sequence: &str, expect: Vec<MatcherResult<char>>) {
    assert_eq!(sequence.len(), expect.len());
    for (i, c) in sequence.char_indices() {
      assert_eq!(matcher.next(&c), expect[i], "failed on index {}", i);
    }
  }
}
