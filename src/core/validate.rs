use super::Grammar;
use super::{Rule, Span};
use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ValidateError {
    pub span: Span,
    pub message: String,
}

impl Grammar {
    pub fn validate(&self) -> Vec<ValidateError> {
        let mut errors = vec![];

        let non_terminals = self.non_terminals();
        for rule in &self.get_rules()[1..] {
            if let Some(error) = rule.unused_error(&non_terminals) {
                errors.push(error);
            }
        }
        for rule in &self.get_rules() {
            errors.append(&mut rule.undefined_error(&self.get_rules()));
        }
        for rule in &self.get_rules() {
            let defined_rules = self.rule_by_id(&rule.id);
            if let Some(error) = rule.already_defined_error(&defined_rules) {
                errors.push(error);
            }
        }
        errors
    }

    fn rule_by_id(&self, id: &str) -> Vec<Rule> {
        let mut rules = vec![];
        for ruleset in &self.rulesets {
            for rule in &ruleset.rules {
                if rule.id == id {
                    rules.push(rule.clone());
                }
            }
        }
        rules
    }

    pub fn get_rules(&self) -> Vec<Rule> {
        let mut rules = vec![];
        for ruleset in &self.rulesets {
            for rule in &ruleset.rules {
                rules.push(rule.clone());
            }
        }
        rules
    }
}

impl Rule {
    fn unused_error(&self, non_terminals: &HashMap<String, Vec<String>>) -> Option<ValidateError> {
        if non_terminals.contains_key(&self.id) {
            None
        } else {
            let span = self.span.clone();
            let message = format!("rule <{}> is not used", self.id);
            Some(ValidateError { span, message })
        }
    }

    fn undefined_error(&self, rules: &Vec<Rule>) -> Vec<ValidateError> {
        let mut errors = vec![];
        let rules = rules
            .iter()
            .map(|rule| rule.id.clone())
            .collect::<Vec<String>>();
        for non_terminal in self.non_terminals() {
            if !rules.contains(&non_terminal.id) {
                let span = non_terminal.span.clone();
                let message = format!("rule <{}> is not defined", non_terminal.id);
                let error = ValidateError { span, message };
                errors.push(error);
            }
        }
        errors
    }

    fn already_defined_error(&self, rules: &Vec<Rule>) -> Option<ValidateError> {
        if rules.len() > 1 {
            let span = self.span.clone();
            let message = format!("rule <{}> is defined several times", self.id);
            Some(ValidateError { span, message })
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::{Expression, ExpressionKind};

    fn rule() -> Rule {
        Rule {
            span: Span { start: 10, end: 14 },
            id: "name".to_string(),
            expression: Expression {
                span: Span { start: 15, end: 16 },
                kind: ExpressionKind::NonTerminal("a".to_string()),
            },
        }
    }

    #[test]
    pub fn test_unused_error() {
        let mut terminals = HashMap::new();
        assert_eq!(
            rule().unused_error(&terminals).unwrap(),
            ValidateError {
                span: Span { start: 10, end: 14 },
                message: "rule <name> is not used".to_string()
            }
        );
        terminals.insert(
            "name".to_string(),
            vec!["rule1".to_string(), "a".to_string()],
        );
        assert!(rule().unused_error(&terminals).is_none());
    }

    #[test]
    pub fn test_undefined_error() {
        let mut rules = vec![];

        rules.push(Rule {
            span: Span { start: 1, end: 10 },
            id: "name".to_string(),
            expression: Expression {
                span: Span { start: 5, end: 10 },
                kind: ExpressionKind::Literal("a".to_string()),
            },
        });
        assert_eq!(
            rule().undefined_error(&rules),
            vec![ValidateError {
                span: Span { start: 15, end: 16 },
                message: "rule <a> is not defined".to_string()
            }]
        );
        rules.push(Rule {
            span: Span { start: 1, end: 10 },
            id: "x".to_string(),
            expression: Expression {
                span: Span { start: 5, end: 10 },
                kind: ExpressionKind::Literal("a".to_string()),
            },
        });
        //assert!(rule().undefined_error(&rules).is_empty());
    }
}
