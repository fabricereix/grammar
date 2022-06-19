use crate::core::*;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TokenKind {
    Colon,
    Pipe,
    LeftParenthesis,
    RightParenthesis,
    Tilde,
    Quantifier(Quantifier),
    Whitespace(String),
    Newline(String),
    Comment(String),
    Identifier(String),
    LiteralString(String),
    Regex(String),
    //Eof,
    Error(String),
}

impl TokenKind {
    pub fn value(&self) -> String {
        match self {
            TokenKind::Colon => ":".to_string(),
            TokenKind::Pipe => "|".to_string(),
            TokenKind::LeftParenthesis => "(".to_string(),
            TokenKind::RightParenthesis => ")".to_string(),
            TokenKind::Tilde => "~".to_string(),
            TokenKind::Quantifier(quantifier) => quantifier.to_string(),
            TokenKind::Whitespace(s) => s.to_string(),
            TokenKind::Newline(s) => s.to_string(),
            TokenKind::Comment(s) => s.to_string(),
            TokenKind::Identifier(s) => s.to_string(),
            TokenKind::LiteralString(s) => s.to_string(),
            TokenKind::Regex(s) => s.to_string(),
            //TokenKind::Eof => "<eof>".to_string(),
            TokenKind::Error(message) => format!("Error: {}", message),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NodeType {
    GrammarFile,
    Rule,
    ChoiceExpression,
    SequenceExpression,
    GroupExpression,
    PrimaryExpression,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Node {
    pub node_type: NodeType,
    pub children: Vec<Element>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Element {
    Node(Node),
    Token(Token),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ParseError {
    pub span: Span,
    pub message: String,
}

impl Node {
    pub fn errors(&self) -> Vec<ParseError> {
        let mut errors = vec![];
        for element in self.children.clone() {
            errors.append(&mut element.errors());
        }
        errors
    }
}

impl Element {
    pub fn errors(&self) -> Vec<ParseError> {
        match self {
            Element::Node(node) => node.errors(),
            Element::Token(Token {
                kind: TokenKind::Error(message),
                span,
            }) => vec![ParseError {
                span: span.clone(),
                message: message.to_string(),
            }],
            _ => vec![],
        }
    }
}
