#[derive(Clone, PartialEq, Debug)]
pub enum TokenKind {
    EndOfFile,

    Error(String),
    Identifier(String),
    Integer(u64),
    Float(f64),

    Colon,
    Semicolon,
    LParen,
    RParen,
    LBrace,
    RBrace,
    Comma,
    RightArrow,

    Plus,
    Minus,
    Asterisk,
    Slash,
    Percent,
    Equals,
    ExclamationMark,

    PlusEquals,
    MinusEquals,
    AsteriskEquals,
    SlashEquals,
    PercentEquals,
    EqualsEquals,
    ExclamationMarkEquals,
}

#[derive(Clone, PartialEq, Debug)]
pub struct Token {
    pub kind: TokenKind,
    pub position: usize,
    pub line: usize,
    pub column: usize,
    pub length: usize,
}

impl Token {
    pub fn new(kind: TokenKind, position: usize, line: usize, column: usize, length: usize) -> Token {
        Token {
            kind,
            position,
            line,
            column,
            length,
        }
    }
}
