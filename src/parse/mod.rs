pub mod ast;

use std::collections::VecDeque;

use ast::*;
use crate::{error::{parse_error::{InvalidLiteral, InvalidUnaryOperatorErr, NoTokensOnParseErr}, LangError}, lex::token::{Token, TokenType}};

pub struct Parser {
    tokens: VecDeque<Token>
}

impl Parser {
    pub fn new(tokens: &[Token]) -> Self {
        let mut vec_deque = VecDeque::with_capacity(tokens.len());
        vec_deque.extend(tokens.iter().cloned());
        Parser { tokens: vec_deque}
    }
}

trait Parsable {
    fn parse(parser: &mut Parser) -> Result<Self, Box<dyn LangError>> where Self: Sized;
}

impl Parsable for Literal {
    fn parse(parser: &mut Parser) -> Result<Self, Box<dyn LangError>> {
        let token: Token = match &parser.tokens.front() {
            None => return Err(Box::new(NoTokensOnParseErr)),
            Some(_) => parser.tokens.pop_front().unwrap(), 
        };

        let ty = match token.token_type() {
            TokenType::True | TokenType::False => LangType::Boolean,
            TokenType::Number(_,_) => LangType::Number,
            TokenType::Char(_) => LangType::Char,
            _ => return Err(Box::new(InvalidLiteral::new(token)))
        };

        Ok(Literal::new(token, ty))
    }
}

impl Parsable for UnaryOperator {
    fn parse(parser: &mut Parser) -> Result<Self, Box<dyn LangError>> where Self: Sized {
        let token: Token = match parser.tokens.pop_front() {
            None => return Err(Box::new(NoTokensOnParseErr)),
            Some(t) => t,
        };

        let ty = match token.token_type() {
            TokenType::Bang => LangType::Boolean,
            TokenType::Minus => LangType::Number,
            _ => return Err(Box::new(InvalidUnaryOperatorErr::new(token)))
        };

        let res = UnaryOperator::new(token, ty);
        res.type_check()?;
        Ok(res)
    }
}

impl Parsable for BinaryOperator {
    fn parse(parser: &mut Parser) -> Result<Self, Box<dyn LangError>> where Self: Sized {
        let token: Token = match parser.tokens.pop_front() {
            None => return Err(Box::new(NoTokensOnParseErr)),
            Some(t) => t,
        };

        let ty = match token.token_type() {
            TokenType::EqEq | TokenType::BangEq => LangType::Boolean,

            TokenType::LessThan | TokenType::LessThanEq | TokenType::GreaterThan
            | TokenType::GreaterThanEq | TokenType::Plus | TokenType::Minus
            | TokenType::Star | TokenType::Slash => LangType::Number,

            _ => todo!("binary operator error")
        };

        let res = BinaryOperator::new(token, ty);
        res.type_check()?;
        Ok(res)
    }
}






mod tests {
    use super::*;
    
    #[test]
    fn parse_literal_true() {
        let tokens = vec![Token::new(TokenType::True, 0, 0, 4)];
        let mut parser: Parser = Parser::new(&tokens);
        let result = Literal::parse(&mut parser).unwrap();
        let expected: Literal = Literal::new(tokens.first().unwrap().clone(), LangType::Boolean);
        assert_eq!(expected, result);
    }

    #[test]
    fn parse_literal_false() {
        let tokens = vec![Token::new(TokenType::False, 0, 0, 5)];
        let mut parser: Parser = Parser::new(&tokens);
        let result = Literal::parse(&mut parser).unwrap();
        let expected: Literal = Literal::new(tokens.first().unwrap().clone(), LangType::Boolean);
        assert_eq!(expected, result);
    }

    #[test]
    fn parse_literal_number() {
        let tokens = vec![Token::new(TokenType::Number(34, "34".to_owned()), 0, 0, 2)];
        let mut parser: Parser = Parser::new(&tokens);
        let result = Literal::parse(&mut parser).unwrap();
        let expected: Literal = Literal::new(tokens.first().unwrap().clone(), LangType::Number);
        assert_eq!(expected, result);
    }

    #[test]
    fn parse_literal_char() {
        let tokens = vec![Token::new(TokenType::Char('h'), 0, 0, 3)];
        let mut parser: Parser = Parser::new(&tokens);
        let result = Literal::parse(&mut parser).unwrap();
        let expected: Literal = Literal::new(tokens.first().unwrap().clone(), LangType::Char);
        assert_eq!(expected, result);
    }

    #[test]
    fn parse_unary_op_bang() {
        let tokens = vec![Token::new(TokenType::Bang, 0, 0, 1)];
        let mut parser: Parser = Parser::new(&tokens);
        let result = UnaryOperator::parse(&mut parser).unwrap();
        let expected = UnaryOperator::new(tokens.first().unwrap().clone(), LangType::Boolean);
        assert_eq!(expected, result);
    }

    #[test]
    fn parse_unary_op_minus() {
        let tokens = vec![Token::new(TokenType::Minus, 0, 0, 1)];
        let mut parser: Parser = Parser::new(&tokens);
        let result = UnaryOperator::parse(&mut parser).unwrap();
        let expected = UnaryOperator::new(tokens.first().unwrap().clone(), LangType::Number);
        assert_eq!(expected, result);
    }

    #[test]
    fn parse_binary_op_eqeq() {
        let tokens = vec![Token::new(TokenType::EqEq, 0, 0, 2)];
        let mut parser: Parser = Parser::new(&tokens);
        let result = BinaryOperator::parse(&mut parser).unwrap();
        let expected = BinaryOperator::new(tokens.first().unwrap().clone(), LangType::Boolean);
        assert_eq!(expected, result);
    }





}