#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Quantifier {
    ZeroOrOne, // ?
    OneOrMany, // +
    Many,      // *
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Grammar {
    pub rulesets: Vec<RuleSet>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuleSet {
    pub comment: Comment,
    pub rules: Vec<Rule>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Comment {
    pub span: Span,
    pub value: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Rule {
    pub span: Span,
    pub id: String,
    pub expression: Expression,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Expression {
    pub span: Span,
    pub kind: ExpressionKind,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ExpressionKind {
    Choice(Vec<Expression>),
    Sequence(Vec<Expression>),
    Group(Box<Expression>),
    Negate(Box<Expression>),
    Quantifier(Box<Expression>, Quantifier),
    Literal(String),
    Regex(String),
    NonTerminal(String),
}

impl std::fmt::Display for Quantifier {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Quantifier::ZeroOrOne => write!(f, "?"),
            Quantifier::OneOrMany => write!(f, "+"),
            Quantifier::Many => write!(f, "*"),
        }
    }
}
