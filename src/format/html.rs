use super::Token;

impl Token {
    pub fn to_html(&self) -> String {
        match self {
            Token::Rule(tokens) => {
                let mut s = r#"<div class="grammar-rule">"#.to_string();
                s.push_str(
                    tokens
                        .iter()
                        .map(|token| token.to_html())
                        .collect::<Vec<String>>()
                        .join("")
                        .as_str(),
                );
                s.push_str("</div>\n");
                s.to_string()
            }
            Token::RuleId(s) => {
                format!("<span id=\"{}\" class=\"grammar-rule-id\">{}</span>", s, s)
            }
            Token::RuleExpression(tokens) => tokens
                .iter()
                .map(|token| token.to_html())
                .collect::<Vec<String>>()
                .join(""),
            Token::Whitespace(s) => s.to_string(),
            Token::Literal(s) => format!("<span class=\"grammar-literal\">{}</span>", s),
            Token::NonTerminal(s) => format!("<a href=\"#{}\">{}</a>", s, s),
            Token::Regex(s) => format!("<span class=\"grammar-regex\">{}</span>", s),
            Token::Symbol(s) => format!("<span class=\"grammar-symbol\">{}</span>", s),
            Token::UsedBy(tokens) => {
                let mut s = r#"<span class="grammar-usedby">"#.to_string();
                s.push_str(
                    tokens
                        .iter()
                        .map(|token| token.to_html())
                        .collect::<Vec<String>>()
                        .join("")
                        .as_str(),
                );
                s.push_str("</span>");
                s.to_string()
            }
            Token::Text(s) => s.to_string(),
            Token::Comment(s) => format!("\n<span class=\"title\">{}</span>\n", s),
            Token::RuleSet(tokens) => {
                let mut s = "<div class=\"ruleset\">".to_string();
                s.push_str(
                    tokens
                        .iter()
                        .map(|token| token.to_html())
                        .collect::<Vec<String>>()
                        .join("")
                        .as_str(),
                );
                s.push_str("</div>");
                s
            }
        }
    }
}
