use super::{Token, TokenKind};
use crate::core::*;

pub struct Scanner {
    offset: usize,
    buffer: Vec<char>,
    eof: bool,
}

impl Scanner {
    pub fn init(s: &str) -> Scanner {
        let offset = 0;
        let buffer = s.chars().collect();
        Scanner {
            offset,
            buffer,
            eof: false,
        }
    }
}

impl Iterator for Scanner {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        if self.eof {
            return None;
        }

        let start = self.offset;
        if let Some(c) = self.read() {
            let token = match c {
                '~' => self.tilde(),
                ':' => Token {
                    kind: TokenKind::Colon,
                    span: Span {
                        start,
                        end: start + 1,
                    },
                },
                '(' => Token {
                    kind: TokenKind::LeftParenthesis,
                    span: Span {
                        start,
                        end: start + 1,
                    },
                },
                ')' => Token {
                    kind: TokenKind::RightParenthesis,
                    span: Span {
                        start,
                        end: start + 1,
                    },
                },
                '|' => Token {
                    kind: TokenKind::Pipe,
                    span: Span {
                        start,
                        end: start + 1,
                    },
                },
                '?' => Token {
                    kind: TokenKind::Quantifier(Quantifier::ZeroOrOne),
                    span: Span {
                        start,
                        end: start + 1,
                    },
                },
                '+' => Token {
                    kind: TokenKind::Quantifier(Quantifier::OneOrMany),
                    span: Span {
                        start,
                        end: start + 1,
                    },
                },
                '*' => Token {
                    kind: TokenKind::Quantifier(Quantifier::Many),
                    span: Span {
                        start,
                        end: start + 1,
                    },
                },
                '#' => self.comment(),
                '"' => self.literal_string(),
                '[' => self.character_class(false),
                _ => {
                    if c.is_whitespace() {
                        self.whitespace()
                    } else if c.is_alphabetic() {
                        self.identifier()
                    } else {
                        let message = format!("Unexpected character '{}'", c);
                        Token {
                            kind: TokenKind::Error(message),
                            span: Span { start, end: 0 },
                        }
                    }
                }
            };
            Some(token)
        } else {
            None
        }
    }
}

impl Scanner {
    fn identifier(&mut self) -> Token {
        let start = self.offset - 1;
        while let Some(c) = self.peek() {
            if c.is_alphanumeric() || c == '-' {
                self.read();
            } else {
                break;
            }
        }
        let text: String = self.buffer[start..self.offset].iter().collect();
        let end = self.offset;
        Token {
            kind: TokenKind::Identifier(text),
            span: Span { start, end },
        }
    }

    fn whitespace(&mut self) -> Token {
        let start = self.offset - 1;

        while let Some(c) = self.peek() {
            if !c.is_whitespace() {
                break;
            }
            self.next();
        }
        let text: String = self.buffer[start..self.offset].iter().collect();
        let end = self.offset - start;
        if text.ends_with('\n') {
            Token {
                kind: TokenKind::Newline(text),
                span: Span { start, end },
            }
        } else {
            Token {
                kind: TokenKind::Whitespace(text),
                span: Span { start, end },
            }
        }
    }

    fn literal_string(&mut self) -> Token {
        let start = self.offset - 1;
        let mut text = "".to_string();
        loop {
            match self.read() {
                None => {
                    let start = self.offset - 1;
                    let text = "Expected a closing quote".to_string();
                    return Token {
                        kind: TokenKind::Error(text),
                        span: Span { start, end: start },
                    };
                }
                Some('\n') => {
                    let start = self.offset - 1;
                    let text = "Expected a closing quote".to_string();
                    return Token {
                        kind: TokenKind::Error(text),
                        span: Span { start, end: start },
                    };
                }
                Some('"') => {
                    break;
                }
                Some('\\') => match self.read() {
                    Some('"') => {
                        text.push('"');
                    }
                    // Some('n') => {
                    //     text.push('\\');
                    //     text.push('n');
                    // }
                    // Some('r') => {
                    //     text.push('\\');
                    //     text.push('r');
                    // }
                    // Some('t') => {
                    //     text.push('\\');
                    //     text.push('t');
                    // }
                    Some('\\') => {
                        text.push('\\');
                    }
                    Some(c) => {
                        text.push('\\');
                        text.push(c);
                    }
                    None => {
                        let offset = self.offset;
                        let text = "Unexpected End of file".to_string();
                        return Token {
                            kind: TokenKind::Error(text),
                            span: Span {
                                start: offset,
                                end: 0,
                            },
                        };
                    }
                },
                Some(c) => text.push(c),
            }
        }
        let end = self.offset;
        Token {
            kind: TokenKind::LiteralString(text.to_string()),
            span: Span { start, end },
        }
    }

    fn comment(&mut self) -> Token {
        let start = self.offset - 1;
        let mut text = "".to_string();
        loop {
            match self.peek() {
                None | Some('\n') => {
                    break;
                }
                Some(c) => {
                    self.read();
                    text.push(c)
                }
            }
        }
        let end = self.offset - start;
        Token {
            kind: TokenKind::Comment(text),
            span: Span { start, end },
        }
    }

    fn tilde(&mut self) -> Token {
        let start = self.offset - 1;
        match self.peek() {
            Some('[') => {
                self.offset += 1;
                self.character_class(true)
            }
            _ => Token {
                kind: TokenKind::Tilde,
                span: Span {
                    start,
                    end: start + 1,
                },
            },
        }
    }

    fn character_class(&mut self, negative: bool) -> Token {
        let start = if negative {
            self.offset - 2
        } else {
            self.offset - 1
        };
        let mut text = if negative {
            "~[".to_string()
        } else {
            "[".to_string()
        };
        loop {
            match self.read() {
                None => {
                    let start = self.offset;
                    let text = "Expected a closing bracket".to_string();
                    return Token {
                        kind: TokenKind::Error(text),
                        span: Span { start, end: start },
                    };
                }
                Some('\n') => {
                    let start = self.offset - 1;
                    let text = "Expected a closing bracket".to_string();
                    return Token {
                        kind: TokenKind::Error(text),
                        span: Span { start, end: start },
                    };
                }
                Some(']') => {
                    break;
                }
                Some(c) => text.push(c),
            }
        }
        text.push(']');
        if let Some(c) = self.peek() {
            if c == '?' || c == '+' || c == '*' {
                self.offset += 1;
                text.push(c);
            }
        }
        let end = self.offset;
        Token {
            kind: TokenKind::Regex(text),
            span: Span { start, end },
        }
    }

    fn read(&mut self) -> Option<char> {
        match self.buffer.get(self.offset) {
            None => {
                self.eof = true;
                None
            }
            Some(c) => {
                self.offset += 1;
                Some(*c)
            }
        }
    }
    fn peek(&mut self) -> Option<char> {
        self.buffer.get(self.offset).copied()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_eof() {
        let mut scanner = Scanner::init("");
        // assert_eq!(
        //     scanner.next().unwrap(),
        //     Token {
        //         kind: TokenKind::Eof,
        //         span: Span { start: 0, end: 0 }
        //     }
        // );
        assert!(scanner.next().is_none());
    }

    #[test]
    fn test_whitespace() {
        let mut scanner = Scanner::init("   abc");
        assert_eq!(
            scanner.next().unwrap(),
            Token {
                kind: TokenKind::Whitespace("   ".to_string()),
                span: Span { start: 0, end: 3 }
            }
        );
        assert_eq!(scanner.offset, 3);

        let mut scanner = Scanner::init(" \n  abc");
        assert_eq!(
            scanner.next().unwrap(),
            Token {
                kind: TokenKind::Whitespace(" \n  ".to_string()),
                span: Span { start: 0, end: 4 }
            }
        );
        assert_eq!(scanner.offset, 4);

        let mut scanner = Scanner::init(" \nabc");
        assert_eq!(
            scanner.next().unwrap(),
            Token {
                kind: TokenKind::Newline(" \n".to_string()),
                span: Span { start: 0, end: 2 }
            }
        );
        assert_eq!(scanner.offset, 2);
    }

    #[test]
    fn test_identifier() {
        let mut scanner = Scanner::init("abc|");
        assert_eq!(
            scanner.next().unwrap(),
            Token {
                kind: TokenKind::Identifier("abc".to_string()),
                span: Span { start: 0, end: 3 }
            }
        );
        assert_eq!(scanner.offset, 3);
    }

    #[test]
    fn test_literal_string() {
        let mut scanner = Scanner::init("\"abc\"");
        scanner.read();
        assert_eq!(
            scanner.literal_string(),
            Token {
                kind: TokenKind::LiteralString("abc".to_string()),
                span: Span { start: 0, end: 5 }
            }
        );
        assert_eq!(scanner.offset, 5);

        // double quote
        let mut scanner = Scanner::init(r#""\"""#);
        assert_eq!(
            scanner.next().unwrap(),
            Token {
                kind: TokenKind::LiteralString("\"".to_string()),
                span: Span { start: 0, end: 4 }
            }
        );
        assert_eq!(scanner.offset, 4);

        // newline
        let mut scanner = Scanner::init("\"\\n\"");
        assert_eq!(
            scanner.next().unwrap(),
            Token {
                kind: TokenKind::LiteralString("\\n".to_string()),
                span: Span { start: 0, end: 4 }
            }
        );
        assert_eq!(scanner.offset, 4);
    }
}
