use crate::lex::token::{Token,TokenType};
trait Expression {

}



struct Operator {
    token: Token
}

struct Literal {
    token: Token
}

struct UnaryExpression {
    left: Token,
    right: Box<dyn Expression>
}

struct BinaryExpression {

}

impl Operator {
    fn new(token: Token) -> Self {
        assert!(Self::is_valid_operator(&token));
        Operator {token}
    }

    fn is_valid_operator(token: &Token) -> bool {
        match token.token_type() {
            TokenType::LessThan |
            TokenType::GreaterThan |
            TokenType::LessThanEq |
            TokenType::GreaterThanEq|
            TokenType::EqEq |
            TokenType::BangEq |
            TokenType::Minus |
            TokenType::Plus |
            TokenType::Slash |
            TokenType::Star => true,
            _ => false
        }
    }
}

impl Literal {
    fn new(token: Token) -> Self {
        assert!(Self::is_valid_literal(&token));
        Literal { token }
    }

    fn is_valid_literal(token: &Token) -> bool {
        match token.token_type() {
            TokenType::Identifier(_) |
            TokenType::Char(_) |
            TokenType::Number(_, _) |
            TokenType::True |
            TokenType::False => true,
            _ => false,
        }
    }
}