use super::Grammar;
use super::{Expression, ExpressionKind, Rule, Span};
use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NonTerminal {
    pub span: Span,
    pub id: String,
}

impl Grammar {
    pub fn non_terminals(&self) -> HashMap<String, Vec<String>> {
        let mut map: HashMap<String, Vec<String>> = HashMap::new();
        for rule in &self.get_rules() {
            for non_terminal in rule.non_terminals() {
                match map.get_mut(&non_terminal.id) {
                    Some(values) => {
                        if !values.contains(&rule.id) {
                            values.push(rule.id.clone());
                        }
                    }
                    None => {
                        map.insert(non_terminal.id, vec![rule.id.clone()]);
                    }
                }
            }
        }
        map
    }
}

impl Rule {
    pub(crate) fn non_terminals(&self) -> Vec<NonTerminal> {
        self.expression.non_terminals()
    }
}

impl Expression {
    fn non_terminals(&self) -> Vec<NonTerminal> {
        match self.kind.clone() {
            ExpressionKind::Choice(expressions) | ExpressionKind::Sequence(expressions) => {
                expressions
                    .iter()
                    .flat_map(|e| e.non_terminals())
                    .collect::<Vec<NonTerminal>>()
            }
            ExpressionKind::Negate(expression)
            | ExpressionKind::Quantifier(expression, _)
            | ExpressionKind::Group(expression) => expression.non_terminals(),
            ExpressionKind::Literal(_) => vec![],
            ExpressionKind::Regex(_) => vec![],
            ExpressionKind::NonTerminal(id) => vec![NonTerminal {
                span: self.span.clone(),
                id,
            }],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::{Comment, RuleSet};
    use super::*;

    #[test]
    pub fn test_expression() {
        assert_eq!(
            Expression {
                span: Span { start: 0, end: 0 },
                kind: ExpressionKind::Literal("Hello".to_string()),
            }
            .non_terminals(),
            vec![]
        );
        assert_eq!(
            Expression {
                span: Span { start: 10, end: 14 },
                kind: ExpressionKind::NonTerminal("name".to_string()),
            }
            .non_terminals(),
            vec![NonTerminal {
                span: Span { start: 10, end: 14 },
                id: "name".to_string(),
            }]
        );
    }

    #[test]
    pub fn test_rule() {
        assert_eq!(
            Rule {
                span: Span { start: 4, end: 9 },
                id: "rule1".to_string(),
                expression: Expression {
                    span: Span { start: 10, end: 14 },
                    kind: ExpressionKind::NonTerminal("name".to_string()),
                },
            }
            .non_terminals(),
            vec![NonTerminal {
                span: Span { start: 10, end: 14 },
                id: "name".to_string(),
            }]
        );
    }

    #[test]
    pub fn test_grammar() {
        // rule1: a | b
        // a: "A" | a
        let g = Grammar {
            rulesets: vec![RuleSet {
                comment: Comment {
                    span: Span { start: 0, end: 5 },
                    value: "".to_string(),
                },
                rules: vec![
                    Rule {
                        span: Span { start: 0, end: 5 },
                        id: "rule1".to_string(),
                        expression: Expression {
                            span: Span { start: 7, end: 12 },
                            kind: ExpressionKind::Choice(vec![
                                Expression {
                                    span: Span { start: 7, end: 8 },
                                    kind: ExpressionKind::NonTerminal("a".to_string()),
                                },
                                Expression {
                                    span: Span { start: 11, end: 12 },
                                    kind: ExpressionKind::NonTerminal("b".to_string()),
                                },
                            ]),
                        },
                    },
                    Rule {
                        span: Span { start: 13, end: 23 },
                        id: "a".to_string(),
                        expression: Expression {
                            span: Span { start: 16, end: 23 },
                            kind: ExpressionKind::Choice(vec![
                                Expression {
                                    span: Span { start: 16, end: 19 },
                                    kind: ExpressionKind::Literal("A".to_string()),
                                },
                                Expression {
                                    span: Span { start: 23, end: 23 },
                                    kind: ExpressionKind::NonTerminal("a".to_string()),
                                },
                            ]),
                        },
                    },
                ],
            }],
        };
        let mut map = HashMap::new();
        map.insert("a".to_string(), vec!["rule1".to_string(), "a".to_string()]);
        map.insert("b".to_string(), vec!["rule1".to_string()]);
        assert_eq!(g.non_terminals(), map);
    }
}
