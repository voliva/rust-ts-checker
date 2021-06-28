use super::imports::ImportDeclaration;
use crate::lexer::Lexer;

/// SourceFile
#[derive(Debug)]
pub struct SourceFile {
  children: Vec<SourceFileElement>,
}

#[derive(Debug)]
enum SourceFileElement {
  ImportDeclaration(ImportDeclaration),
  // FunctionDeclaration(FunctionDeclaration),
}

impl From<Lexer> for SourceFile {
  fn from(lexer: Lexer) -> Self {
    let mut peekable = lexer.peekable();

    let mut children = vec![];

    loop {
      let mut has_result = false;

      let result = ImportDeclaration::create(&mut peekable);
      match result {
        Some(Ok(v)) => {
          has_result = true;
          children.push(SourceFileElement::ImportDeclaration(v));
        }
        Some(Err(r)) => {
          panic!("Error parsing import: {}", r);
        }
        _ => {}
      };

      if !has_result {
        break;
      }
    }

    SourceFile { children }
  }
}
