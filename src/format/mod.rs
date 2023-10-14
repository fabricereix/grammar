use super::{Comment, Expression, ExpressionKind, Grammar, Rule, RuleSet};
use crate::Quantifier;
use std::collections::HashMap;


pub fn format_html(g: &Grammar, s: &str, section_header: &str, section_id: bool) -> String {
    let non_terminals = g.non_terminals();
    g.to_html(s, &non_terminals, section_header, section_id)
}

impl Grammar {
    pub fn to_html(&self, input: &str, used_by: &HashMap<String, Vec<String>>, section_header: &str, section_id: bool) -> String {
        let mut s = "".to_string();
        for ruleset in &self.rulesets {
            s.push_str(ruleset.to_html(input, used_by, section_header, section_id).as_str());
        }
        s
    }
}

impl RuleSet {
    pub fn to_html(&self, input: &str, used_by: &HashMap<String, Vec<String>>, section_header: &str, section_id: bool) -> String {
        let mut s = r#"<div class="grammar-ruleset">"#.to_string();
        let section_id = if section_id {
           format!(" id=\"{}\"", encode_html(&comment_to_id(&self.comment.value)))
        } else {
            "".to_string()
        };
        s.push_str(format!(r#"<{section_header}{section_id}>{}</{section_header}>"#, self.comment.to_html()).as_str());

        for rule in &self.rules {
            let used_by = match used_by.get(&rule.id) {
                Some(used_by) => used_by.clone(),
                None => vec![],
            };
            s.push_str(format!("{}\n", rule.to_html(input, &used_by)).as_str());
        }
        s.push_str("</div>");
        s
    }
}

impl Comment {
    pub fn to_html(&self) -> String {
        encode_html(&self.value)
    }
}

fn comment_to_id(value: &str) -> String {
   value
       .to_lowercase()
       .replace('/', "-")
       .replace(' ', "-")
       .replace("----", "-")
       .replace("---", "-")
       .replace("--", "-")
}


impl Rule {
    pub fn to_html(&self, input: &str, used_by: &[String]) -> String {
        let mut s = r#"<div class="grammar-rule">"#.to_string();
        s.push_str(html_rule_declaration(&self.id, used_by).as_str());
        s.push_str(html_rule_expression(&self.expression, input).as_str());
        s.push_str("</div>");
        s
    }
}

fn html_rule_declaration(id: &str, used_by: &[String]) -> String {
    let mut s = r#"<div class="grammar-rule-declaration">"#.to_string();
    s.push_str(
        format!(
            r#"<span class="grammar-rule-id" id="{id}">{id}</span>"#,
            id = id
        )
        .as_str(),
    );
    if !used_by.is_empty() {
        s.push_str(html_used_by(used_by).as_str());
    }
    s.push_str("</div>");
    s
}

fn html_used_by(used_by: &[String]) -> String {
    let used_by = used_by
        .iter()
        .map(|id| format!(r##"<a href="#{id}">{id}</a>"##, id = id))
        .collect::<Vec<String>>();
    format!(
        r#"<span class="grammar-usedby">(used by {})</span>"#,
        used_by.join(",&nbsp;")
    )
}

fn html_rule_expression(expr: &Expression, input: &str) -> String {
    let mut s = r#"<div class="grammar-rule-expression">"#.to_string();
    s.push_str(expr.to_html(0, input).as_str());
    s.push_str("</div>");
    s
}

impl Expression {
    pub fn to_html(&self, indent: usize, input: &str) -> String {
        self.kind.to_html(indent, input)
    }
}

impl ExpressionKind {
    pub fn to_html(&self, indent: usize, input: &str) -> String {
        match self {
            ExpressionKind::Choice(expressions) => html_choice(expressions.clone(), indent, input),
            ExpressionKind::Sequence(expressions) => {
                html_sequence(expressions.clone(), indent, input)
            }
            ExpressionKind::Group(expression) => html_group(*expression.clone(), indent, input),
            ExpressionKind::Negate(expression) => html_negate(*expression.clone(), indent, input),
            ExpressionKind::Quantifier(expression, quantifier) => {
                html_quantifier(*expression.clone(), quantifier, indent, input)
            }
            ExpressionKind::Literal(s) => html_literal(s),
            ExpressionKind::Regex(s) => html_regex(s),
            ExpressionKind::NonTerminal(s) => html_non_terminal(s),
        }
    }
}

fn html_choice(expressions: Vec<Expression>, indent: usize, input: &str) -> String {
    let mut expressions = expressions;
    let first_expression = expressions.remove(0);
    let mut s = first_expression.to_html(indent + 1, input);
    let mut previous = first_expression.span.end;
    for expression in expressions {
        if indent == 0 && input[previous..expression.span.start].contains('\n') {
            s.push_str("<br>\n");
        }
        s.push_str(r#"<span class="grammar-symbol">|</span>"#);
        s.push_str(expression.to_html(indent + 1, input).as_str());
        previous = expression.span.end;
    }
    if s.contains("<br>") {
        s = format!("&nbsp;{}", s);
    }
    s
}

fn html_sequence(expressions: Vec<Expression>, indent: usize, input: &str) -> String {
    let mut expressions = expressions;
    let first_expression = expressions.remove(0);
    let mut s = first_expression.to_html(indent + 1, input);
    let mut previous = first_expression.span.end;
    for expression in expressions {
        if input[previous..expression.span.start].contains('\n') {
            s.push_str("<br>\n");
        } else {
            s.push_str("&nbsp;");
        }
        s.push_str(expression.to_html(indent + 1, input).as_str());
        previous = expression.span.end;
    }
    s
}

fn html_group(expression: Expression, indent: usize, input: &str) -> String {
    format!(
        r#"<span class="grammar-symbol">(</span>{}<span class="grammar-symbol">)</span>"#,
        expression.to_html(indent, input)
    )
}

fn html_negate(expression: Expression, indent: usize, input: &str) -> String {
    format!(
        r#"<span class="grammar-symbol">~</span>{}"#,
        expression.to_html(indent, input)
    )
}

fn html_quantifier(
    expression: Expression,
    quantifier: &Quantifier,
    indent: usize,
    input: &str,
) -> String {
    format!(
        r#"{}{}"#,
        expression.to_html(indent, input),
        quantifier.to_html()
    )
}

fn html_literal(s: &str) -> String {
    format!(r#"<span class="grammar-literal">{}</span>"#, encode_html(s))
}

fn html_regex(s: &str) -> String {
    format!(r#"<span class="grammar-regex">{}</span>"#, encode_html(s))
}

fn html_non_terminal(s: &str) -> String {
    format!(r##"<a href="#{name}">{name}</a>"##, name = encode_html(s))
}

fn encode_html(s: &str) -> String {
    s.replace('>', "&gt;").replace('<', "&lt;")
}

impl Quantifier {
    pub fn to_html(&self) -> String {
        let s = match self {
            Quantifier::ZeroOrOne => "?",
            Quantifier::OneOrMany => "+",
            Quantifier::Many => "*",
        };
        format!(r#"<span class="grammar-symbol">{}</span>"#, s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Span;

    #[test]
    fn test_rule() {
        let input = "name: a";
        let used_by = vec!["other".to_string()];
        assert_eq!(
            Rule {
                span: Span { start: 0, end: 1 },
                id: "name".to_string(),
                expression:
                        Expression {
                            span: Span { start: 2, end: 3 },
                            kind: ExpressionKind::NonTerminal("a".to_string()),
                        }
            }.to_html(input, &used_by),
            "<div class=\"grammar-rule\"><div class=\"grammar-rule-declaration\"><span class=\"grammar-rule-id\" id=\"name\">name</span><span class=\"grammar-usedby\">(used by <a href=\"#other\">other</a>)</span></div><div class=\"grammar-rule-expression\"><a href=\"#a\">a</a></div></div>",
        );
    }

    #[test]
    fn test_choice() {
        // one line
        let input = "a|b";
        assert_eq!(
            ExpressionKind::Choice(vec![
                Expression {
                    span: Span { start: 0, end: 1 },
                    kind: ExpressionKind::NonTerminal("a".to_string()),
                },
                Expression {
                    span: Span { start: 2, end: 3 },
                    kind: ExpressionKind::NonTerminal("b".to_string()),
                },
            ])
            .to_html(0, input),
            "<a href=\"#a\">a</a><span class=\"grammar-symbol\">|</span><a href=\"#b\">b</a>",
        );

        // several lines
        let input = "a\n|b";
        assert_eq!(
            ExpressionKind::Choice(vec![
                Expression {
                    span: Span { start: 0, end: 1 },
                    kind: ExpressionKind::NonTerminal("a".to_string()),
                },
                Expression {
                    span: Span { start: 3, end: 4 },
                    kind: ExpressionKind::NonTerminal("b".to_string()),
                },
            ]).to_html(0, input),
            "&nbsp;<a href=\"#a\">a</a><br>\n<span class=\"grammar-symbol\">|</span><a href=\"#b\">b</a>",
        );
    }

    #[test]
    fn test_sequence() {
        // one line
        let input = "a b";
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
            .to_html(0, input),
            "<a href=\"#a\">a</a>&nbsp;<a href=\"#b\">b</a>",
        );
    }

    #[test]
    fn test_group() {
        // one line
        let input = "(a)";
        assert_eq!(
            Expression {
                span: Span { start: 0, end: 3 },
                kind: ExpressionKind::Group(
                    Box::new(Expression {
                        span: Span { start: 1, end: 2 },
                        kind: ExpressionKind::NonTerminal("a".to_string()),
                    }
                )),
            }.to_html(0, input),
            "<span class=\"grammar-symbol\">(</span><a href=\"#a\">a</a><span class=\"grammar-symbol\">)</span>",
        );

        // several lines
        let input = "(a\nb)";
        assert_eq!(
            Expression {
                span: Span { start: 0, end: 3 },
                kind: ExpressionKind::Group(
                    Box::new(Expression {
                        span: Span { start: 1, end: 2 },
                        kind: ExpressionKind::Sequence(vec![
                            Expression {
                                span: Span { start: 1, end: 2 },
                                kind: ExpressionKind::NonTerminal("a".to_string())
                            },
                            Expression {
                                span: Span { start: 3, end: 4 },
                                kind: ExpressionKind::NonTerminal("b".to_string())
                            }
                        ]),
                    }
                    )),
            }.to_html(0, input),
            "<span class=\"grammar-symbol\">(</span><a href=\"#a\">a</a><br>\n<a href=\"#b\">b</a><span class=\"grammar-symbol\">)</span>",
        );
    }

    #[test]
    fn test_nonterminal() {
        assert_eq!(
            ExpressionKind::NonTerminal("name".to_string()).to_html(0, ""),
            "<a href=\"#name\">name</a>".to_string()
        )
    }

    #[test]
    fn test_literal() {
        assert_eq!(
            ExpressionKind::Literal("GET".to_string()).to_html(0, ""),
            r#"<span class="grammar-literal">GET</span>"#.to_string()
        );
    }

    #[test]
    fn test_regex() {
        assert_eq!(
            ExpressionKind::Regex("[a-z]+".to_string()).to_html(0, ""),
            r#"<span class="grammar-regex">[a-z]+</span>"#.to_string()
        );
        assert_eq!(
            ExpressionKind::Regex("[<]".to_string()).to_html(0, ""),
            r#"<span class="grammar-regex">[&lt;]</span>"#.to_string()
        );
    }

    #[test]
    fn test_comment_to_id() {
        assert_eq!(comment_to_id("Template / Expression"), "template-expression");
        assert_eq!(comment_to_id("Lexical Grammar"), "lexical-grammar");
    }
}
