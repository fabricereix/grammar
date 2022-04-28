mod html;

use super::{Comment, Expression, ExpressionKind, Grammar, Rule, RuleSet};
use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Token {
    RuleSet(Vec<Token>),
    Rule(Vec<Token>),
    RuleId(String),
    RuleExpression(Vec<Token>),
    Whitespace(String),
    Literal(String),
    NonTerminal(String),
    Regex(String),
    Symbol(String),
    Text(String),
    Comment(String),
    UsedBy(Vec<Token>),
}

pub fn format_html(g: &Grammar, s: &str) -> String {
    let non_terminals = g.non_terminals();
    let tokens = g.tokenize_usedby(s, &non_terminals);
    let mut s = "<pre><code>\n".to_string();
    s.push_str(
        tokens
            .iter()
            .map(|token| token.to_html())
            .collect::<Vec<String>>()
            .join("")
            .as_str(),
    );
    s.push_str("</code></pre>");
    s.to_string()
}

impl Grammar {
    pub fn tokenize(&self, s: &str) -> Vec<Token> {
        let mut tokens = vec![];
        for ruleset in self.rulesets.clone() {
            tokens.push(ruleset.tokenize(s));
            tokens.push(Token::Whitespace("\n\n".to_string()));
        }
        tokens
    }

    pub fn tokenize_usedby(
        &self,
        s: &str,
        non_terminals: &HashMap<String, Vec<String>>,
    ) -> Vec<Token> {
        let mut tokens = vec![];
        for ruleset in self.rulesets.clone() {
            tokens.push(ruleset.tokenize_used_by(s, non_terminals));
            tokens.push(Token::Whitespace("\n\n".to_string()));
        }
        tokens
    }
}

impl RuleSet {
    pub fn tokenize(&self, s: &str) -> Token {
        let mut tokens = vec![];
        tokens.push(self.comment.tokenize());
        for rule in &self.rules {
            tokens.push(rule.tokenize(s));
        }
        tokens.push(Token::Whitespace("\n\n".to_string()));
        Token::RuleSet(tokens)
    }

    pub fn tokenize_used_by(&self, s: &str, non_terminals: &HashMap<String, Vec<String>>) -> Token {
        let mut tokens = vec![];
        tokens.push(self.comment.tokenize());
        for rule in &self.rules {
            let used_by = match non_terminals.get(&rule.id) {
                None => vec![],
                Some(values) => values.to_vec(),
            };
            tokens.push(rule.tokenize_usedby(s, &used_by));
        }
        tokens.push(Token::Whitespace("\n\n".to_string()));
        Token::RuleSet(tokens)
    }
}

impl Comment {
    pub fn tokenize(&self) -> Token {
        Token::Comment(self.value.clone().trim().to_string())
    }
}

impl Rule {
    pub fn tokenize(&self, s: &str) -> Token {
        let mut tokens = vec![];
        tokens.push(Token::RuleId(self.id.clone()));
        tokens.push(Token::Symbol(":".to_string()));
        tokens.push(Token::Whitespace("\n  ".to_string()));
        tokens.push(Token::RuleExpression(self.expression.format(s)));
        Token::Rule(tokens)
    }

    pub fn tokenize_usedby(&self, s: &str, used_by: &Vec<String>) -> Token {
        let mut tokens = vec![];
        tokens.push(Token::RuleId(self.id.clone()));
        tokens.push(Token::Symbol(":".to_string()));
        if !used_by.is_empty() {
            tokens.push(Token::Whitespace(" ".to_string()));

            let mut usedby_tokens = vec![];
            usedby_tokens.push(Token::Text("(used by ".to_string()));
            usedby_tokens.push(Token::NonTerminal(used_by.get(0).unwrap().clone()));
            for s in &used_by[1..] {
                usedby_tokens.push(Token::Whitespace(", ".to_string()));
                usedby_tokens.push(Token::NonTerminal(s.clone()));
            }
            usedby_tokens.push(Token::Text(")".to_string()));
            tokens.push(Token::UsedBy(usedby_tokens));
        }
        tokens.push(Token::Whitespace("\n  ".to_string()));
        tokens.push(Token::RuleExpression(self.expression.format(s)));
        Token::Rule(tokens)
    }
}

fn indent(input_tokens: Vec<Token>) -> Vec<Token> {
    let mut tokens = vec![];
    for token in input_tokens {
        let token = if let Token::Whitespace(s) = token {
            Token::Whitespace(s.replace("\n", "\n  "))
        } else {
            token
        };
        tokens.push(token);
    }
    tokens
}

impl Expression {
    pub fn format(&self, s: &str) -> Vec<Token> {
        match self.kind.clone() {
            ExpressionKind::Choice(expressions) => {
                let first_expression = expressions.get(0).expect("at least one element");
                let second_expression = expressions.get(1).expect("at least two element");
                let mut tokens = vec![];

                let start = first_expression.span.end;
                let end = second_expression.span.end;
                first_expression.format(s);
                if s[start..end].contains("\n") {
                    tokens.push(Token::Whitespace("  ".to_string()));
                }
                tokens.append(&mut first_expression.format(s));
                let start = first_expression.span.end;
                for expression in &expressions[1..] {
                    let end = expression.span.start;
                    if s[start..end].contains("\n") {
                        tokens.push(Token::Whitespace("\n  ".to_string()));
                    } else {
                        tokens.push(Token::Whitespace(" ".to_string()));
                    }
                    tokens.push(Token::Symbol("|".to_string()));
                    tokens.push(Token::Whitespace(" ".to_string()));
                    tokens.append(&mut expression.format(s));
                }
                tokens
            }

            ExpressionKind::Sequence(expressions) => {
                let first_expression = expressions.get(0).expect("at least one element");
                let mut tokens = first_expression.format(s);
                let mut start = first_expression.span.end;
                for expression in &expressions[1..] {
                    let end = expression.span.start;
                    if s[start..end].contains("\n") {
                        tokens.push(Token::Whitespace("\n  ".to_string()));
                    } else {
                        tokens.push(Token::Whitespace(" ".to_string()));
                    }
                    tokens.append(&mut expression.format(s));
                    start = expression.span.start;
                }
                tokens
            }
            ExpressionKind::Group(expression) => {
                let mut tokens = vec![Token::Symbol("(".to_string())];

                let start = self.span.start;
                let end = expression.span.start;
                if s[start..end].contains("\n") {
                    tokens.push(Token::Whitespace("\n  ".to_string()));
                    tokens.append(&mut indent(expression.format(s)));
                } else {
                    tokens.push(Token::Whitespace(" ".to_string()));
                    tokens.append(&mut expression.format(s));
                }

                let start = expression.span.end;
                let end = self.span.end;
                if s[start..end].contains("\n") {
                    tokens.push(Token::Whitespace("\n".to_string()));
                } else {
                    tokens.push(Token::Whitespace(" ".to_string()));
                }

                tokens.push(Token::Symbol(")".to_string()));
                tokens
            }
            ExpressionKind::Literal(s) => vec![Token::Literal(s)],
            ExpressionKind::Regex(s) => vec![Token::Regex(s)],
            ExpressionKind::NonTerminal(s) => vec![Token::NonTerminal(s)],

            ExpressionKind::Negate(expression) => {
                let mut tokens = vec![Token::Symbol("~".to_string())];
                tokens.append(&mut expression.format(s));
                tokens
            }
            ExpressionKind::Quantifier(expression, quantifier) => {
                let mut tokens = expression.format(s);
                tokens.push(Token::Symbol(quantifier.to_string()));
                tokens
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::Span;
    use super::*;

    #[test]
    fn test_choice() {
        // one line
        assert_eq!(
            Expression {
                span: Span { start: 0, end: 3 },
                kind: ExpressionKind::Choice(vec![
                    Expression {
                        span: Span { start: 0, end: 1 },
                        kind: ExpressionKind::NonTerminal("a".to_string()),
                    },
                    Expression {
                        span: Span { start: 2, end: 3 },
                        kind: ExpressionKind::NonTerminal("b".to_string()),
                    },
                ]),
            }
            .format("a|b"),
            vec![
                Token::NonTerminal("a".to_string()),
                Token::Whitespace(" ".to_string()),
                Token::Symbol("|".to_string()),
                Token::Whitespace(" ".to_string()),
                Token::NonTerminal("b".to_string()),
            ]
        );

        // several lines
        assert_eq!(
            Expression {
                span: Span { start: 0, end: 5 },
                kind: ExpressionKind::Choice(vec![
                    Expression {
                        span: Span { start: 0, end: 1 },
                        kind: ExpressionKind::NonTerminal("a".to_string()),
                    },
                    Expression {
                        span: Span { start: 4, end: 5 },
                        kind: ExpressionKind::NonTerminal("b".to_string()),
                    },
                ]),
            }
            .format("a\n |b"),
            vec![
                Token::Whitespace("  ".to_string()),
                Token::NonTerminal("a".to_string()),
                Token::Whitespace("\n  ".to_string()),
                Token::Symbol("|".to_string()),
                Token::Whitespace(" ".to_string()),
                Token::NonTerminal("b".to_string()),
            ]
        );
    }

    #[test]
    fn test_sequence() {
        // one line
        assert_eq!(
            Expression {
                span: Span { start: 0, end: 3 },
                kind: ExpressionKind::Sequence(vec![
                    Expression {
                        span: Span { start: 0, end: 1 },
                        kind: ExpressionKind::NonTerminal("a".to_string()),
                    },
                    Expression {
                        span: Span { start: 2, end: 3 },
                        kind: ExpressionKind::NonTerminal("b".to_string()),
                    },
                ]),
            }
            .format("a b"),
            vec![
                Token::NonTerminal("a".to_string()),
                Token::Whitespace(" ".to_string()),
                Token::NonTerminal("b".to_string()),
            ]
        );

        // several lines
        assert_eq!(
            Expression {
                span: Span { start: 0, end: 6 },
                kind: ExpressionKind::Sequence(vec![
                    Expression {
                        span: Span { start: 0, end: 1 },
                        kind: ExpressionKind::NonTerminal("a".to_string()),
                    },
                    Expression {
                        span: Span { start: 2, end: 3 },
                        kind: ExpressionKind::NonTerminal("b".to_string()),
                    },
                    Expression {
                        span: Span { start: 5, end: 6 },
                        kind: ExpressionKind::NonTerminal("c".to_string()),
                    }
                ]),
            }
            .format("a b\n c"),
            vec![
                Token::NonTerminal("a".to_string()),
                Token::Whitespace(" ".to_string()),
                Token::NonTerminal("b".to_string()),
                Token::Whitespace("\n  ".to_string()),
                Token::NonTerminal("c".to_string()),
            ]
        );
    }

    #[test]
    fn test_group() {
        // one line
        assert_eq!(
            Expression {
                span: Span { start: 0, end: 3 },
                kind: ExpressionKind::Group(Box::new(Expression {
                    span: Span { start: 1, end: 2 },
                    kind: ExpressionKind::NonTerminal("a".to_string()),
                })),
            }
            .format("(a)"),
            vec![
                Token::Symbol("(".to_string()),
                Token::Whitespace(" ".to_string()),
                Token::NonTerminal("a".to_string()),
                Token::Whitespace(" ".to_string()),
                Token::Symbol(")".to_string()),
            ]
        );

        // several lines
        assert_eq!(
            Expression {
                span: Span { start: 0, end: 4 },
                kind: ExpressionKind::Group(Box::new(Expression {
                    span: Span { start: 2, end: 3 },
                    kind: ExpressionKind::NonTerminal("a".to_string()),
                })),
            }
            .format("(\na)"),
            vec![
                Token::Symbol("(".to_string()),
                Token::Whitespace("\n  ".to_string()),
                Token::NonTerminal("a".to_string()),
                Token::Whitespace(" ".to_string()),
                Token::Symbol(")".to_string()),
            ]
        )
    }

    #[test]
    fn test_literal() {
        assert_eq!(
            Expression {
                span: Span { start: 0, end: 0 },
                kind: ExpressionKind::Literal("GET".to_string()),
            }
            .format(""),
            vec![Token::Literal("GET".to_string())]
        )
    }

    #[test]
    fn test_regex() {
        assert_eq!(
            Expression {
                span: Span { start: 0, end: 6 },
                kind: ExpressionKind::Regex("[a-z]+".to_string()),
            }
            .format(""),
            vec![Token::Regex("[a-z]+".to_string())]
        )
    }

    #[test]
    fn test_nonterminal() {
        assert_eq!(
            Expression {
                span: Span { start: 0, end: 4 },
                kind: ExpressionKind::NonTerminal("name".to_string()),
            }
            .format(""),
            vec![Token::NonTerminal("name".to_string())]
        )
    }
}
