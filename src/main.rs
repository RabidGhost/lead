use lex::Lexer;
use parse::LangParser;
// use miette::NamedSource;
// use miette::{Diagnostic, Result, SourceSpan};

mod error;
mod lang;
mod lex;
mod parse;
mod repl;

fn main() {
    let input = "(16 * (17 / 0xf)) + 17 + 12 + 4";

    let mut lexer: Lexer = Lexer::new(input);

    let tokens = match lexer.run() {
        Ok(tokens) => tokens,
        Err(e) => {
            eprintln!("{e:?}");
            return;
        }
    };

    let mut parser: LangParser = LangParser::new(&tokens);
    let ast = match parser.parse() {
        Ok(expr) => expr,
        Err(e) => {
            eprintln!("{e:?}");
            return;
        }
    };

    dbg!(ast);
}
