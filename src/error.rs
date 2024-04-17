use std::io::Write;

pub trait LangError {
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
