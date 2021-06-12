#[cfg(test)]
mod lexer_tests {
  // Note this useful idiom: importing names from outer (for mod tests) scope.
  use crate::lexer::{Lexer, LexerItem};
  use crate::tokens::{Literal, Token};
  use itertools::assert_equal;

  #[test]
  fn const_declaration() {
    let lexer = Lexer::from_text("const myVariable:MyType = 5;");
    let result = vec![
      keyword("const"),
      identifier("myVariable"),
      symbol(":"),
      identifier("MyType"),
      symbol("="),
      i_literal(5),
      symbol(";"),
    ]
    .into_iter();

    assert_equal(lexer, result);
  }

  #[test]
  fn literals() {
    let lexer = Lexer::from_text("\"double' quote\" 'single\" quote' 0 123");
    let result = vec![
      s_literal("double' quote"),
      s_literal("single\" quote"),
      i_literal(0),
      i_literal(123),
    ]
    .into_iter();

    assert_equal(lexer, result);
  }

  #[test]
  fn identifier_vs_keyword() {
    let lexer = Lexer::from_text("afunction function functiona");
    let result = vec![
      identifier("afunction"),
      keyword("function"),
      identifier("functiona"),
    ]
    .into_iter();

    assert_equal(lexer, result);
  }

  #[test]
  fn fake_jsx() {
    let lexer = Lexer::from_text("var fn = <T extends any>() => void 0");
    let result = vec![
      keyword("var"),
      identifier("fn"),
      symbol("="),
      symbol("<"),
      identifier("T"),
      keyword("extends"),
      identifier("any"),
      symbol(">"),
      symbol("("),
      symbol(")"),
      symbol("=>"),
      identifier("void"),
      i_literal(0),
    ]
    .into_iter();

    assert_equal(lexer, result);

    let lexer = Lexer::from_text("3 < value || value > 5");
    let result = vec![
      i_literal(3),
      symbol("<"),
      identifier("value"),
      symbol("||"),
      identifier("value"),
      symbol(">"),
      i_literal(5),
    ]
    .into_iter();

    assert_equal(lexer, result);
  }

  #[test]
  fn real_jsx() {
    let lexer = Lexer::from_text("let fn = <T>() => void 0</T>");
    let result = vec![
      keyword("let"),
      identifier("fn"),
      symbol("="),
      symbol("<"),
      identifier("T"),
      symbol(">"),
      s_literal("() => void 0"),
      symbol("</"),
      identifier("T"),
      symbol(">"),
    ]
    .into_iter();

    assert_equal(lexer, result);
  }

  #[test]
  fn complex_jsx_element() {
    let lexer = Lexer::from_text("<Elm.Sub<Generic<T>> prop1 prop2=\"2\" prop3={3} />");
    let result = vec![
      symbol("<"),
      identifier("Elm"),
      symbol("."),
      identifier("Sub"),
      symbol("<"),
      identifier("Generic"),
      symbol("<"),
      identifier("T"),
      symbol(">"),
      symbol(">"),
      identifier("prop1"),
      identifier("prop2"),
      symbol("="),
      s_literal("2"),
      identifier("prop3"),
      symbol("="),
      symbol("{"),
      i_literal(3),
      symbol("}"),
      symbol("/>"),
    ]
    .into_iter();

    assert_equal(lexer, result);

    let lexer = Lexer::from_text("<Elm<G>>body</Elm>");
    let result = vec![
      symbol("<"),
      identifier("Elm"),
      symbol("<"),
      identifier("G"), // TODO i_literal(3)
      symbol(">"),
      symbol(">"),
      s_literal("body"),
      symbol("</"),
      identifier("Elm"),
      symbol(">"),
    ]
    .into_iter();

    assert_equal(lexer, result);
  }

  #[test]
  fn jsx_short_element() {
    // TODO this is contrived - I want to verify that /> after <Identifier returns back to parsing TS
    let lexer = Lexer::from_text("<Elm /> > otherElement");
    let result = vec![
      symbol("<"),
      identifier("Elm"),
      symbol("/>"),
      symbol(">"),
      identifier("otherElement"),
    ]
    .into_iter();

    assert_equal(lexer, result);

    // TODO this is contrived - I want to verify that /> when in JSX returns back to parsing TS
    let lexer = Lexer::from_text("<Elm foo /> > otherElement");
    let result = vec![
      symbol("<"),
      identifier("Elm"),
      identifier("foo"),
      symbol("/>"),
      symbol(">"),
      identifier("otherElement"),
    ]
    .into_iter();

    assert_equal(lexer, result);
  }

  #[test]
  fn jsx_element_to_ts() {
    let lexer = Lexer::from_text("<Elm prop1={({a,b}) => {}} prop2={() => <Elm2>text</Elm2>}/>");
    let result = vec![
      symbol("<"),
      identifier("Elm"),
      identifier("prop1"),
      symbol("="),
      symbol("{"),
      symbol("("),
      symbol("{"),
      identifier("a"),
      symbol(","),
      identifier("b"),
      symbol("}"),
      symbol(")"),
      symbol("=>"),
      symbol("{"),
      symbol("}"),
      symbol("}"),
      identifier("prop2"),
      symbol("="),
      symbol("{"),
      symbol("("),
      symbol(")"),
      symbol("=>"),
      symbol("<"),
      identifier("Elm2"),
      symbol(">"),
      s_literal("text"),
      symbol("</"),
      identifier("Elm2"),
      symbol(">"),
      symbol("}"),
      symbol("/>"),
    ]
    .into_iter();

    assert_equal(lexer, result);
  }

  #[test]
  fn jsx_reserved_prop() {
    let lexer = Lexer::from_text("<Element prop interface=\"hello\" />");
    let result = vec![
      symbol("<"),
      identifier("Element"),
      identifier("prop"), // TODO removing this one makes the test fail
      identifier("interface"),
      symbol("="),
      s_literal("hello"),
      symbol("/>"),
    ]
    .into_iter();

    assert_equal(lexer, result);
  }

  #[test]
  fn jsx_children_to_ts() {
    let lexer = Lexer::from_text(
      "<Elm>Some text{
      render_element(<Elm>text</Elm>, {})
    }More text</Elm>",
    );
    let result = vec![
      symbol("<"),
      identifier("Elm"),
      symbol(">"),
      s_literal("Some text"),
      symbol("{"),
      identifier("render_element"),
      symbol("("),
      symbol("<"),
      identifier("Elm"),
      symbol(">"),
      s_literal("text"),
      symbol("</"),
      identifier("Elm"),
      symbol(">"),
      symbol(","),
      symbol("{"),
      symbol("}"),
      symbol(")"),
      symbol("}"),
      s_literal("More text"),
      symbol("</"),
      identifier("Elm"),
      symbol(">"),
    ]
    .into_iter();

    assert_equal(lexer, result);
  }

  #[test]
  fn jsx_nested() {
    let lexer = Lexer::from_text(
      "<Parent>
        <Child foo bar>foo</Child>
        foo bar<>yo</>foo</Parent>",
    );
    let result = vec![
      symbol("<"),
      identifier("Parent"),
      symbol(">"),
      symbol("<"),
      identifier("Child"),
      identifier("foo"),
      identifier("bar"),
      symbol(">"),
      s_literal("foo"),
      symbol("</"),
      identifier("Child"),
      symbol(">"),
      s_literal("foo bar"),
      symbol("<>"),
      s_literal("yo"),
      symbol("</>"),
      s_literal("foo"),
      symbol("</"),
      identifier("Parent"),
      symbol(">"),
    ]
    .into_iter();

    assert_equal(lexer, result);
  }

  #[test]
  fn jsx_fragment() {
    let lexer = Lexer::from_text("<>body {child}</> === element");
    let result = vec![
      symbol("<>"),
      s_literal("body "),
      symbol("{"),
      identifier("child"),
      symbol("}"),
      symbol("</>"),
      symbol("==="),
      identifier("element"),
    ]
    .into_iter();

    assert_equal(lexer, result);
  }

  #[test]
  fn comments() {
    let lexer = Lexer::from_text(
      "
      foo() // ignore these
      /* also these */
      not_this/* this yes */()
      /*
      and this
      */
      <>/* not this */{/* but this */}</>
    ",
    );
    let result = vec![
      identifier("foo"),
      symbol("("),
      symbol(")"),
      identifier("not_this"),
      symbol("("),
      symbol(")"),
      symbol("<>"),
      s_literal("/* not this */"),
      symbol("{"),
      symbol("}"),
      symbol("</>"),
    ]
    .into_iter();

    assert_equal(lexer, result);
  }

  #[test]
  fn void() {
    let lexer = Lexer::from_text("");
    let result = vec![].into_iter();

    assert_equal(lexer, result);
  }

  // TODO spread props <Element {...props} /> <Element lol {...props} />

  fn keyword(string: &str) -> LexerItem {
    Ok(Token::Keyword(string.to_string()))
  }
  fn identifier(string: &str) -> LexerItem {
    Ok(Token::Identifier(string.to_string()))
  }
  fn symbol(string: &str) -> LexerItem {
    Ok(Token::Symbol(string.to_string()))
  }
  fn i_literal(value: i32) -> LexerItem {
    Ok(Token::Literal(Literal::Integer(value)))
  }
  fn s_literal(string: &str) -> LexerItem {
    Ok(Token::Literal(Literal::Str(string.to_string())))
  }
}
