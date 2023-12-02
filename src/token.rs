#[derive(Debug, PartialEq, Clone)]
pub enum TokenType {
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Comma,
    Dot,
    Minus,
    Plus,
    Semicolon,
    Slash,
    Star,
    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,
    // literals
    Identifier(String),
    Number(f64),
    Str(String),
    // keywords
    And,
    Not,
    Struct,
    Else,
    False,
    Fun,
    For,
    If,
    Null,
    Or,
    Return,
    Super,
    Slf, // Self is a reserved keyword
    True,
    Let,
    While,
    // EOF
    Eof,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Token {
    pub typ: TokenType,
    pub lexeme: String,
    pub line: u16,
}
