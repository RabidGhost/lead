pub mod ast;

use std::collections::VecDeque;

use ast::*;
use crate::{error::{parse_error::{InvalidLiteral, InvalidUnaryOperatorErr, NoTokensOnParseErr}, LangError}, lex::token::{Token, TokenType}};

#[derive(Debug)]
pub struct Parser {
    tokens: VecDeque<Token>
}

impl Parser {
    pub fn new(tokens: &[Token]) -> Self {
        let mut vec_deque = VecDeque::with_capacity(tokens.len());
        vec_deque.extend(tokens.iter().cloned());
        Parser { tokens: vec_deque}
    }

    pub fn sub_parser<R: std::slice::SliceIndex<[Token], Output = [Token]>>(&mut self, range: R) -> Self {
        Self::new(&(self.tokens.make_contiguous()[range]))
    }

    pub fn find_matching_parenthesis(&self) -> Option<usize> {
        if *self.tokens.front()?.token_type() != TokenType::LeftParen {return None};

        let mut nest_count: usize = 0;
        let mut pos = 0;
        for token in self.tokens.iter() {
            if *token.token_type() == TokenType::LeftParen {nest_count += 1}
            if *token.token_type() == TokenType::RightParen {nest_count -= 1}
            if nest_count == 0 {
                return Some(pos)
            }
            pos += 1;
        }
        todo!("unbalanced parens. need to make error for this. or return none")
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

impl Parsable for UnaryApplication {
    fn parse(parser: &mut Parser) -> Result<Self, Box<dyn LangError>> where Self: Sized {
        let op = UnaryOperator::parse(parser)?;
        let expr = Expression::parse(parser)?;
        let res = UnaryApplication::new(op, expr);
        res.type_check()?;
        Ok(res)
    }
}

impl Parsable for BinaryApplication {
    fn parse(parser: &mut Parser) -> Result<Self, Box<dyn LangError>> where Self: Sized {
        let left = Expression::parse(parser)?;
        let op = BinaryOperator::parse(parser)?;
        let right = Expression::parse(parser)?;
        let res = BinaryApplication::new(op, left, right);
        res.type_check()?;
        Ok(res)
    }
}

impl Parsable for Grouping {
    fn parse(parser: &mut Parser) -> Result<Self, Box<dyn LangError>> where Self: Sized {
        let matching_paren = match parser.find_matching_parenthesis() {
            Some(pos) => pos,
            None => todo!("the method should probably just return a result")
        };
        let expr = Expression::parse(&mut parser.sub_parser(1..matching_paren))?;
        // might need type check line
        Ok(Grouping::new(expr))

        // let left_pr = match parser.tokens.pop_front() {
        //     None => return Err(Box::new(NoTokensOnParseErr)),
        //     Some(t) => if *t.token_type() != TokenType::LeftParen {todo!() /* error here for no paren */} ,
        // };
        // let expr = Expression::parse(parser)?;
        // let right_pr = match parser.tokens.pop_front() {
        //     None => return Err(Box::new(NoTokensOnParseErr)),
        //     Some(t) => if *t.token_type() != TokenType::RightParen {todo!() /* error here for no paren */},
        // };
        
        // let res = Grouping::new(expr);
        // res.type_check()?;
        // Ok(res)
    }
}

impl Parsable for Expression {
    fn parse(parser: &mut Parser) -> Result<Self, Box<dyn LangError>> where Self: Sized {
        match parser.tokens.front() {
            None => return Err(Box::new(NoTokensOnParseErr)),
            Some(t) => match *t.token_type() {
                TokenType::Bang | TokenType::Minus => return Ok(Self::ExprUnary(UnaryApplication::parse(parser)?)),
                TokenType::True | TokenType::False | TokenType::Number(_,_) => {
                    match parser.tokens.get(1) {
                        None => Ok(Expression::ExprLiteral(Literal::parse(parser)?)),
                        _ => todo!(),
                    }
                }
                TokenType::LeftParen => {
                    let mut token_iter = parser.tokens.iter();
                    token_iter.next();
                    let pos_matching_paren = match token_iter.position(|t| *t.token_type() == TokenType::RightParen) {
                        Some(pos) => pos,
                        None => todo!("unmatched parenthesise error")
                    };

                    println!("{pos_matching_paren}");

                    let mut subparser: Parser = parser.sub_parser(0..=(pos_matching_paren + 1));
                    dbg!(&subparser);

                    let grouping = Self::ExprGrouping(Grouping::parse(&mut subparser)?);
                    //grouping.type_check()?; //unimplemented for now so dissabled
                    Ok(grouping)
                },
                _ => todo!()
            } ,
        }
    }
}



mod tests {
    use std::vec;

    use crate::error::parse_error;

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

    fn parse_binary_op_helper(tokens: Vec<Token>, expecteds: Vec<BinaryOperator>) {
        let mut parser: Parser = Parser::new(&tokens);
        for i in 0..tokens.len() {
            let result = BinaryOperator::parse(&mut parser).unwrap();
            let expected = &expecteds[i];
            assert_eq!(expected, &result)
        }
    }

    #[test]
    fn parse_binary_op() {
        let to_test = vec![
            Token::new(TokenType::EqEq, 0, 0, 2),
            Token::new(TokenType::BangEq, 0, 0, 2),
            Token::new(TokenType::LessThan, 0, 0, 1),
            Token::new(TokenType::LessThanEq, 0, 0, 2),
            Token::new(TokenType::GreaterThan, 0, 0, 1),
            Token::new(TokenType::GreaterThanEq, 0, 0, 2),
            Token::new(TokenType::Plus, 0, 0, 1),
            Token::new(TokenType::Minus, 0, 0, 1),
            Token::new(TokenType::Star, 0, 0, 1),
            Token::new(TokenType::Slash, 0, 0, 1),
        ];

        let results = vec![
            BinaryOperator::new(Token::new(TokenType::EqEq, 0, 0, 2), LangType::Boolean),
            BinaryOperator::new(Token::new(TokenType::BangEq, 0, 0, 2), LangType::Boolean),
            BinaryOperator::new(Token::new(TokenType::LessThan, 0, 0, 1), LangType::Number),
            BinaryOperator::new(Token::new(TokenType::LessThanEq, 0, 0, 2), LangType::Number),
            BinaryOperator::new(Token::new(TokenType::GreaterThan, 0, 0, 1), LangType::Number),
            BinaryOperator::new(Token::new(TokenType::GreaterThanEq, 0, 0, 2), LangType::Number),
            BinaryOperator::new(Token::new(TokenType::Plus, 0, 0, 1), LangType::Number),
            BinaryOperator::new(Token::new(TokenType::Minus, 0, 0, 1), LangType::Number),
            BinaryOperator::new(Token::new(TokenType::Star, 0, 0, 1), LangType::Number),
            BinaryOperator::new(Token::new(TokenType::Slash, 0, 0, 1), LangType::Number),
            BinaryOperator::new(Token::new(TokenType::EqEq, 0, 0, 2), LangType::Number),
        ];

        parse_binary_op_helper(to_test, results)
    }

    #[test]
    fn parse_grouping_literal() {
        let tokens = vec![Token::new(TokenType::LeftParen, 0, 0, 1), Token::new(TokenType::Number(56, "56".to_owned()), 0, 1, 2), Token::new(TokenType::RightParen, 0, 3, 1)];
        let mut parser: Parser = Parser::new(&tokens);
        let result = Expression::parse(&mut parser).unwrap();
        let expected = Expression::ExprGrouping(
            Grouping::new(
                Expression::ExprLiteral(
                    Literal::new(tokens.iter().nth(1).unwrap().clone(), LangType::Number)
                )));
        assert_eq!(expected, result);
    }





}