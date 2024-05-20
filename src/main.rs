use lex::Lexer;
// use miette::NamedSource;
// use miette::{Diagnostic, Result, SourceSpan};

mod error;
mod lang;
mod lex;
mod parse;
mod repl;

fn main() {
    let input = "let new_var := 5 + 17 - 0xf";

    let mut lexer: Lexer = Lexer::new(input);
    let tokens = match lexer.run() {
        Ok(tokens) => tokens,
        Err(e) => {
            eprintln!("{e:?}");
            return;
        }
    };

    dbg!(tokens);
}
