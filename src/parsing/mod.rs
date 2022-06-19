mod parser;
mod scanner;
mod tokens;

pub use self::parser::Parser;
pub use self::scanner::Scanner;
pub use self::tokens::{Element, Node, NodeType, ParseError, Token, TokenKind};
use crate::core::Grammar;

pub fn parse(s: &str) -> Result<Grammar, Vec<ParseError>> {
    let scanner = Scanner::init(s);
    let tokens = scanner
        .collect::<Vec<Token>>()
        .iter()
        .filter(|token| !matches!(token.kind, TokenKind::Whitespace(_)))
        .cloned()
        .collect::<Vec<Token>>();
    let mut parser = Parser::init(tokens);
    parser.grammar_file()
}
