

pub trait LangError: std::fmt::Debug {
    fn report(&self);
}

pub mod lex_error {
    use super::*;

    // pub struct BufFullOnPushErr {
    //     string: String,
    //     line: usize,
    //     col: usize,
    //     length: usize,
    // }

    // impl BufFullOnPushErr {
    //     pub fn new(string: String, line: usize, col: usize, length: usize) -> Self {
    //         Self { string, line, col, length }
    //     }
    // }
    
    #[derive(Debug)]
    pub struct InvalidLexemeErr {
        string: String,
        line: usize,
        col: usize,
        length: usize,
    }

    impl InvalidLexemeErr {
        pub fn new(string: String, line: usize, col: usize, length: usize) -> Self {
            Self {
                string,
                line,
                col,
                length,
            }
        }
    }
}

pub mod parse_error {
    use crate::parse::ast::{LangExpression, LangType};
    use crate::lex::token::Token;

    use super::LangError;

    #[derive(Debug)]
    pub struct MixedTypeExpressionError {
        left: LangType,
        right: LangType,
    }
    // will need more detail in future

    impl MixedTypeExpressionError {
        pub fn new(left: LangType, right: LangType) -> Self {
            Self { left, right }
        }
    }

    impl LangError for MixedTypeExpressionError {
        fn report(&self) {
            todo!()
        }
    }

    #[derive(Debug)]
    pub struct NoTokensOnParseErr;
    #[derive(Debug)]
    pub struct InvalidLiteral {
        literal: Token,
    }

    impl InvalidLiteral {
        pub fn new(literal: Token) -> Self {
            InvalidLiteral { literal}
        }
    }

    impl LangError for InvalidLiteral {
        fn report(&self) {
            todo!()
        }
    }

    impl LangError for NoTokensOnParseErr {
        fn report(&self) {
            todo!()
        }
    }

    #[derive(Debug)]
    pub struct InvalidUnaryOperatorErr {
        op: Token
    }

    impl InvalidUnaryOperatorErr {
        pub fn new(op: Token) -> Self {
            Self { op }
        }
    }

    impl LangError for InvalidUnaryOperatorErr {
        fn report(&self) {
            todo!()
        }
    }

    // use std::marker::PhantomData;

    // use crate::lex::token::Token;
    // use crate::parse::ast::Boolean;


    // struct BooleanExpressionParseErr<T: Boolean> {
    //     tokens: Vec<Token>,
    //     line: usize,
    //     col: usize,
    //     length: usize,
    //     _ast_type: PhantomData<T>,
    // }

}

// impl LangError for lex_error::BufFullOnPushErr {
//     fn report(&self) {
//         todo!()
//     }
// }

impl LangError for lex_error::InvalidLexemeErr {
    fn report(&self) {
        todo!()
    }
}
