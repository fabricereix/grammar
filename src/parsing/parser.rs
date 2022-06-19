use crate::core::*;

use super::{ParseError, Token, TokenKind};

// easier use vec of tokens
pub struct Parser {
    pub current: usize,
    pub tokens: Vec<Token>,
}

impl Parser {
    pub fn init(tokens: Vec<Token>) -> Parser {
        let current = 0;
        Parser { current, tokens }
    }

    //
    // Utils methods
    //

    fn next(&mut self) -> Option<Token> {
        match self.tokens.get(self.current) {
            None => None,
            Some(token) => {
                self.current += 1;
                Some(token.clone())
            }
        }
    }

    fn peek(&self) -> Option<Token> {
        self.tokens.get(self.current).cloned()
    }

    fn is_eof(&self) -> bool {
        self.peek().is_none()
    }

    fn offset(&self) -> usize {
        match self.tokens.get(self.current) {
            None => self.tokens.get(self.current - 1).unwrap().span.end,
            Some(token) => token.span.start,
        }
    }

    // synchronize to the next newline
    fn synchronize(&mut self) {
        loop {
            match self.next() {
                None => {
                    break;
                }
                Some(token) => {
                    if matches!(token.kind, TokenKind::Newline(_)) {
                        break;
                    }
                }
            };
        }
    }

    fn skip_newlines(&mut self) {
        while let Some(Token {
            kind: TokenKind::Newline(_),
            ..
        }) = self.peek()
        {
            self.next();
        }
    }

    fn parse_error(&self, message: &str) -> ParseError {
        let start = self.offset();
        let end = start;
        let span = Span { start, end };
        let message = message.to_string();
        ParseError { span, message }
    }
    //
    // Grammar
    //

    pub fn grammar_file(&mut self) -> Result<Grammar, Vec<ParseError>> {
        let mut rulesets = vec![];
        let mut errors = vec![];
        loop {
            // consume newline
            self.skip_newlines();

            if self.is_eof() {
                break;
            }

            match self.rule_set() {
                Ok(None) => {
                    let span = Span {
                        start: self.offset(),
                        end: self.offset(),
                    };
                    let message = format!("Unexpected character {:?}", self.peek());
                    let error = ParseError { span, message };
                    errors.push(error);
                    self.synchronize();
                }
                Ok(Some(ruleset)) => rulesets.push(ruleset),
                Err(mut es) => {
                    errors.append(&mut es);
                    self.synchronize();
                }
            }
        }

        if errors.is_empty() {
            Ok(Grammar { rulesets })
        } else {
            Err(errors)
        }
    }

    fn rule_set(&mut self) -> Result<Option<RuleSet>, Vec<ParseError>> {
        let comment = if let Some(comment) = self.comment() {
            comment
        } else {
            return Ok(None);
        };
        let mut rules = vec![];
        let mut errors = vec![];
        loop {
            // consume newline
            self.skip_newlines();
            match self.rule() {
                Ok(None) => {
                    // if self.peek().kind == TokenKind::Eof {
                    //     break;
                    // } else {
                    //     // let span = Span { start: self.offset(), end: self.offset() };
                    //     // let message = format!("Unexpected character {:?}", self.peek());
                    //     // let error = ParseError { span, message };
                    //     // errors.push(error);
                    //     // self.synchronize();
                    //
                    // }
                    break;
                }
                Ok(Some(rule)) => rules.push(rule),
                Err(error) => {
                    errors.push(error);
                    self.synchronize();
                }
            }
        }
        if errors.is_empty() {
            Ok(Some(RuleSet { comment, rules }))
        } else {
            Err(errors)
        }
    }

    fn comment(&mut self) -> Option<Comment> {
        if self.is_eof() {
            return None;
        }
        if let Some(Token {
            kind: TokenKind::Comment(value),
            span,
        }) = self.peek()
        {
            self.next();
            Some(Comment { span, value })
        } else {
            None
        }
    }

    fn rule(&mut self) -> Result<Option<Rule>, ParseError> {
        let start = self.offset();
        let id = match self.identifier_expression() {
            Some(value) => value,
            None => return Ok(None),
        };
        if self.match_token(TokenKind::Colon).is_none() {
            return Err(ParseError {
                span: Span {
                    start: self.offset(),
                    end: 0,
                },
                message: "Expecting a colon".to_string(),
            });
        };
        let expression = match self.choice_expression()? {
            None => {
                return Err(ParseError {
                    span: Span {
                        start: self.offset(),
                        end: 0,
                    },
                    message: "Expecting an expression".to_string(),
                });
            }
            Some(value) => value,
        };

        let end = self.offset();
        let span = Span { start, end };
        Ok(Some(Rule {
            span,
            id,
            expression,
        }))
    }

    fn choice_expression(&mut self) -> Result<Option<Expression>, ParseError> {
        let start = self.offset();
        if self.is_eof() {
            return Ok(None);
        }
        let first_expression = match self.sequence_expression()? {
            None => return Ok(None),
            Some(expression) => expression,
        };
        let mut terms = vec![first_expression.clone()];
        while self.match_token(TokenKind::Pipe).is_some() {
            match self.sequence_expression()? {
                Some(expression) => terms.push(expression),
                None => return Err(self.parse_error("Expecting an expression")),
            }
        }

        if terms.len() == 1 {
            Ok(Some(first_expression))
        } else {
            let end = self.offset();
            Ok(Some(Expression {
                span: Span { start, end },
                kind: ExpressionKind::Choice(terms),
            }))
        }
    }

    fn sequence_expression(&mut self) -> Result<Option<Expression>, ParseError> {
        let start = self.offset();
        let first_expression = match self.negate_expression()? {
            None => return Ok(None),
            Some(expression) => expression,
        };
        let mut terms = vec![first_expression.clone()];
        while let Some(expression) = self.negate_expression()? {
            terms.push(expression);
        }
        if terms.len() == 1 {
            Ok(Some(first_expression))
        } else {
            let end = self.offset();
            Ok(Some(Expression {
                span: Span { start, end },
                kind: ExpressionKind::Sequence(terms),
            }))
        }
    }

    fn negate_expression(&mut self) -> Result<Option<Expression>, ParseError> {
        if self.match_token(TokenKind::Tilde).is_none() {
            return self.quantified_expression();
        }
        let start = self.offset();
        if let Some(expression) = self.quantified_expression()? {
            let end = self.offset();
            Ok(Some(Expression {
                span: Span { start, end },
                kind: ExpressionKind::Negate(Box::new(expression)),
            }))
        } else {
            Err(self.parse_error("Expecting an expression"))
        }
    }

    fn quantified_expression(&mut self) -> Result<Option<Expression>, ParseError> {
        let start = self.offset();
        if let Some(expression) = self.group_expression()? {
            if let Some(quantifier) = self.quantifier() {
                let end = self.tokens.get(self.current - 1).unwrap().span.end;
                //let end = self.offset();
                let span = Span { start, end };
                let kind = ExpressionKind::Quantifier(Box::new(expression), quantifier);
                Ok(Some(Expression { span, kind }))
            } else {
                Ok(Some(expression))
            }
        } else {
            Ok(None)
        }
    }

    fn group_expression(&mut self) -> Result<Option<Expression>, ParseError> {
        let start = self.offset();
        if self.is_eof() {
            return Ok(None);
        }
        if self.match_token(TokenKind::LeftParenthesis).is_none() {
            return Ok(self.primary_expression());
        }

        match self.choice_expression()? {
            None => Err(self.parse_error("Expecting Expression")),
            Some(expression) => {
                if self.match_token(TokenKind::RightParenthesis).is_some() {
                    let end = self.offset();
                    Ok(Some(Expression {
                        span: Span { start, end },
                        kind: ExpressionKind::Group(Box::new(expression)),
                    }))
                } else {
                    Err(self.parse_error("Expecting a right parenthesis"))
                }
            }
        }
    }

    // fn zprimary_expression(&mut self) -> Result<Option<Expression>, ParseError> {
    //     let negate = self.match_token(TokenKind::Tilde);
    //     let core_expression = match self.core_expression() {
    //         None => {
    //             if negate.is_none() {
    //                 return Ok(None);
    //             } else {
    //                 return Err(ParseError {
    //                     span: Span {
    //                         start: self.offset(),
    //                         end: 0,
    //                     },
    //                     message: "Expecting Literal Expression, Character Class or Non-Terminal"
    //                         .to_string(),
    //                 });
    //             }
    //         }
    //         Some(expression) => expression,
    //     };
    //     let quantifier = self.quantifier_expression();
    //     if negate.is_none() && quantifier.is_none() {
    //         Ok(Some(core_expression))
    //     } else {
    //         Ok(Some(
    //             Expression {
    //                 span: Span { start: 0, end: 0 },
    //                 kind: ExpressionKind::PrimaryExpression(PrimaryExpression {
    //                     negate,
    //                     term: Box::new(core_expression),
    //                     quantifier,
    //                 }),
    //             }))
    //     }
    // }

    fn match_token(&mut self, token_kind: TokenKind) -> Option<Token> {
        if let Some(token) = self.peek() {
            if token.kind == token_kind {
                self.next()
            } else {
                None
            }
        } else {
            None
        }
    }

    fn quantifier(&mut self) -> Option<Quantifier> {
        if self.is_eof() {
            return None;
        }
        if let Some(Token {
            kind: TokenKind::Quantifier(quantifier),
            ..
        }) = self.peek()
        {
            self.next();
            Some(quantifier)
        } else {
            None
        }
    }

    fn primary_expression(&mut self) -> Option<Expression> {
        if let Some(Token { span, kind }) = self.peek() {
            let kind = match kind {
                TokenKind::Identifier(value) => ExpressionKind::NonTerminal(value),
                TokenKind::LiteralString(value) => ExpressionKind::Literal(value),
                TokenKind::Regex(value) => ExpressionKind::Regex(value),
                _ => return None,
            };
            self.next();
            Some(Expression { span, kind })
        } else {
            None
        }
    }

    fn identifier_expression(&mut self) -> Option<String> {
        if let Some(Token {
            kind: TokenKind::Identifier(value),
            ..
        }) = self.peek()
        {
            self.next();
            Some(value)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    //
    // Grammar
    //

    #[test]
    pub fn test_grammar() {
        let mut parser = Parser::init(vec![
            comment_token(0, " Comment1"),
            newline_token(9, "\n"),
            comment_token(10, " Comment2"),
            //eof_token(19),
        ]);
        let grammar_file = parser.grammar_file();
        eprintln!("{:#?}", grammar_file);
    }

    // #[test]
    // pub fn test_choice() {
    //     let mut parser = Parser::init(vec![
    //         identifier_token(0, "a"),
    //         pipe_token(0),
    //         whitespace_token(0, " "),
    //         identifier_token(0, "b"),
    //         eof_token(19),
    //     ]);
    //     let expr = parser.choice_expression();
    //     eprintln!("{:#?}", expr);
    // }

    //
    // Lexical Grammar
    //

    #[test]
    pub fn test_comment() {
        let mut parser = Parser::init(vec![comment_token(0, " Comment1")]);
        let grammar_file = parser.grammar_file();
        eprintln!("{:#?}", grammar_file);
    }

    #[test]
    pub fn test_rule() {
        let mut parser = Parser::init(vec![
            identifier_token(10, "rule1"),
            colon_token(15),
            identifier_token(16, "aaa"),
        ]);
        assert_eq!(
            parser.rule().unwrap().unwrap(),
            Rule {
                span: Span { start: 10, end: 19 },
                id: "rule1".to_string(),
                expression: Expression {
                    span: Span { start: 16, end: 19 },
                    kind: ExpressionKind::NonTerminal("aaa".to_string()),
                },
            }
        );
    }

    #[test]
    pub fn test_choice_expression() {
        let mut parser = Parser::init(vec![identifier_token(10, "item")]);
        assert_eq!(
            parser.choice_expression().unwrap().unwrap(),
            Expression {
                span: Span { start: 10, end: 14 },
                kind: ExpressionKind::NonTerminal("item".to_string()),
            }
        );

        let mut parser = Parser::init(vec![
            identifier_token(10, "item1"),
            pipe_token(15),
            identifier_token(16, "item2"),
        ]);
        assert_eq!(
            parser.choice_expression().unwrap().unwrap(),
            Expression {
                span: Span { start: 10, end: 21 },
                kind: ExpressionKind::Choice(vec![
                    Expression {
                        span: Span { start: 10, end: 15 },
                        kind: ExpressionKind::NonTerminal("item1".to_string()),
                    },
                    Expression {
                        span: Span { start: 16, end: 21 },
                        kind: ExpressionKind::NonTerminal("item2".to_string()),
                    }
                ],),
            }
        );
    }

    #[test]
    pub fn test_sequence_expression() {
        let mut parser = Parser::init(vec![identifier_token(10, "item")]);
        assert_eq!(
            parser.sequence_expression().unwrap().unwrap(),
            Expression {
                span: Span { start: 10, end: 14 },
                kind: ExpressionKind::NonTerminal("item".to_string()),
            }
        );

        let mut parser = Parser::init(vec![
            identifier_token(10, "item"),
            identifier_token(20, "other"),
        ]);
        assert_eq!(
            parser.sequence_expression().unwrap().unwrap(),
            Expression {
                span: Span { start: 10, end: 25 },
                kind: ExpressionKind::Sequence(vec![
                    Expression {
                        span: Span { start: 10, end: 14 },
                        kind: ExpressionKind::NonTerminal("item".to_string()),
                    },
                    Expression {
                        span: Span { start: 20, end: 25 },
                        kind: ExpressionKind::NonTerminal("other".to_string()),
                    },
                ]),
            }
        );
    }

    #[test]
    pub fn test_group_expression() {
        let mut parser = Parser::init(vec![identifier_token(10, "item")]);
        assert_eq!(
            parser.group_expression().unwrap().unwrap(),
            Expression {
                span: Span { start: 10, end: 14 },
                kind: ExpressionKind::NonTerminal("item".to_string()),
            }
        );

        let mut parser = Parser::init(vec![
            left_parenthesis_token(9),
            identifier_token(10, "item"),
            right_parenthesis_token(14),
        ]);
        assert_eq!(
            parser.group_expression().unwrap().unwrap(),
            Expression {
                span: Span { start: 9, end: 15 },
                kind: ExpressionKind::Group(Box::new(Expression {
                    span: Span { start: 10, end: 14 },
                    kind: ExpressionKind::NonTerminal("item".to_string()),
                })),
            },
        );
        assert_eq!(parser.current, 3);

        let mut parser = Parser::init(vec![
            left_parenthesis_token(9),
            identifier_token(10, "item"),
            colon_token(14),
        ]);
        assert_eq!(
            parser.group_expression().err().unwrap(),
            ParseError {
                span: Span { start: 14, end: 14 },
                message: "Expecting a right parenthesis".to_string(),
            }
        );
        assert_eq!(parser.current, 2);
    }

    #[test]
    pub fn test_primary_expression() {
        let mut parser = Parser::init(vec![]);
        assert!(parser.primary_expression().is_none());
        assert_eq!(parser.current, 0);

        let mut parser = Parser::init(vec![literal_string_token(10, "Hello")]);
        assert_eq!(
            parser.primary_expression().unwrap(),
            Expression {
                span: Span { start: 10, end: 15 },
                kind: ExpressionKind::Literal("Hello".to_string()),
            }
        );
        assert_eq!(parser.current, 1);

        let mut parser = Parser::init(vec![identifier_token(10, "name")]);
        assert_eq!(
            parser.primary_expression().unwrap(),
            Expression {
                span: Span { start: 10, end: 14 },
                kind: ExpressionKind::NonTerminal("name".to_string()),
            }
        );
        assert_eq!(parser.current, 1);
    }

    //
    // Helpers
    //

    fn colon_token(offset: usize) -> Token {
        Token {
            kind: TokenKind::Colon,
            span: Span {
                start: offset,
                end: offset + 1,
            },
        }
    }

    fn pipe_token(offset: usize) -> Token {
        Token {
            kind: TokenKind::Pipe,
            span: Span {
                start: offset,
                end: offset + 1,
            },
        }
    }

    fn newline_token(offset: usize, s: &str) -> Token {
        Token {
            kind: TokenKind::Newline(s.to_string()),
            span: Span {
                start: offset,
                end: offset + s.len(),
            },
        }
    }

    fn comment_token(offset: usize, s: &str) -> Token {
        Token {
            kind: TokenKind::Comment(s.to_string()),
            span: Span {
                start: offset,
                end: offset + s.len() + 1,
            },
        }
    }

    fn identifier_token(offset: usize, s: &str) -> Token {
        Token {
            kind: TokenKind::Identifier(s.to_string()),
            span: Span {
                start: offset,
                end: offset + s.len(),
            },
        }
    }

    fn left_parenthesis_token(offset: usize) -> Token {
        Token {
            kind: TokenKind::LeftParenthesis,
            span: Span {
                start: offset,
                end: offset + 1,
            },
        }
    }

    fn right_parenthesis_token(offset: usize) -> Token {
        Token {
            kind: TokenKind::RightParenthesis,
            span: Span {
                start: offset,
                end: offset + 1,
            },
        }
    }

    fn literal_string_token(offset: usize, s: &str) -> Token {
        Token {
            kind: TokenKind::LiteralString(s.to_string()),
            span: Span {
                start: offset,
                end: offset + s.len(),
            },
        }
    }

    // fn eof_token(offset: usize) -> Token {
    //     Token {
    //         kind: TokenKind::Eof,
    //         span: Span {
    //             start: offset,
    //             end: offset,
    //         },
    //     }
    // }
}
