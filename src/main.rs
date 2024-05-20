use std::io::{self, Stdin, Stdout};

//use repl::Repl;

mod error;
mod lang;
mod lex;
mod parse;
mod repl;

fn main() {
    // let mut repl: repl::Repl<Stdin, Stdout> = Repl::new(io::stdin(), io::stdout());
    // repl.go()

    // let src: &str = r#"let xyz := 56 ;"#;
    // let lexer = lex::Lexer::new(src);
    // let tokens = lexer.run().unwrap();
    // println!("{src}");
    // print!("[");
    // for tok in tokens {
    //     print!("{tok}, ")
    // }
    // println!("]");
}
