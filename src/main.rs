mod error;
mod lang;
mod lex;
mod parse;

fn main() {
    let src: &str = r#"let xyz := 56 ;"#;
    let lexer = lex::Lexer::new(src);
    let tokens = lexer.run().unwrap();
    println!("{src}");
    print!("[");
    for tok in tokens {
        print!("{tok}, ")
    }
    println!("]");
}
