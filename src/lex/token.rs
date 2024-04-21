#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub enum TokenType {
    Space, // we will filter this out before the parse step.

    // Single char Tokens
    LeftParen,   // (
    RightParen,  // )
    LeftBrace,   // {
    RightBrace,  // }
    LeftSquare,  // [
    RightSquare, // ]
    Comma,       // ,
    Dot,         // .
    Minus,       // -
    Plus,        // +
    Slash,       // /
    Star,        // *
    Semicolon,   // ;

    // One or two char Tokens
    LessThan,      // <
    GreaterThan,   // >
    LessThanEq,    // <=
    GreaterThanEq, // >=
    EqEq,          // ==
    Colon,         // :
    Assign,        // :=
    Bang,          // !
    BangEq,        // !=

    // Literals
    Identifier(String),
    Char(char),
    Number(i32, String), // Number(value, repr)

    // Keywords
    Let,
    If,
    For,
    While,
    True,
    False,

    // End of file
    EOF,
}

impl TokenType {
    pub fn size(&self) -> usize {
        // the size the token would occupy in the source code
        match self {
            TokenType::Space => 1,
            TokenType::LeftParen => 1,
            TokenType::RightParen => 1,
            TokenType::LeftBrace => 1,
            TokenType::RightBrace => 1,
            TokenType::LeftSquare => 1,
            TokenType::RightSquare => 1,
            TokenType::Comma => 1,
            TokenType::Dot => 1,
            TokenType::Minus => 1,
            TokenType::Plus => 1,
            TokenType::Slash => 1,
            TokenType::Star => 1,
            TokenType::Semicolon => 1,
            TokenType::LessThan => 1,
            TokenType::GreaterThan => 1,
            TokenType::LessThanEq => 2,
            TokenType::GreaterThanEq => 2,
            TokenType::EqEq => 2,
            TokenType::Colon => 1,
            TokenType::Assign => 2,
            TokenType::Bang => 1,
            TokenType::BangEq => 2,
            TokenType::Identifier(string) => string.len(),
            TokenType::Char(_) => 3,
            TokenType::Number(_, repr) => repr.len(),
            TokenType::Let => 3,
            TokenType::If => 2,
            TokenType::For => 3,
            TokenType::While => 5,
            TokenType::True => 4,
            TokenType::False => 5,
            TokenType::EOF => 0,
        }
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub struct Token {
    token_type: TokenType,
    line: usize, // The offset in chars from the start of the file. Useful for meaningful error messages
    col: usize,
    length: usize, // The length of the token.
}

impl Token {
    pub fn new(token_type: TokenType, line: usize, col: usize, length: usize) -> Self {
        Self {
            token_type,
            line,
            col,
            length,
        }
    }

    pub fn token_type(&self) -> &TokenType {
        &self.token_type
    }
}

impl std::fmt::Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.token_type)
    }
}
