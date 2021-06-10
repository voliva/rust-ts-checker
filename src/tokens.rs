#[derive(Debug, Clone, PartialEq)]
pub enum Token {
  Identifier(String),
  Literal(Literal),
  Symbol(String),
  Keyword(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
  Integer(i32),
  Str(String),
  BigInt(String),
  Boolean(bool),
  Decimal(f64),
  Regex(String, String), // pattern, flags
  Undefined,
  Null,
}

pub const VALID_SYMBOLS: &[&str] = &[
  "=", "+", "-", "*", "/", "==", "!=", "===", "!==", "<", ">", "<=", ">=", ";", "=>", ",", ".",
  "{", "}", "[", "]", "(", ")", "//", "/*", "*/", "`", "${", "?", ":", "&&", "||", "!", "</", "/>",
  "?.", "??",
];

pub const KNOWN_KEYWORDS: &[&str] = &[
  "import",
  "from",
  "as",
  "function",
  "return",
  "while",
  "if",
  "do",
  "typeof",
  "delete",
  "switch",
  "break",
  "continue",
  "export",
  "const",
  "let",
  "var",
  "interface",
  "extends",
];
