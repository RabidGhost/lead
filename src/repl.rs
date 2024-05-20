// use std::{error::Error, io::{BufReader, BufRead, Read, Write}};
// use crate::{lex::Lexer, parse::{ast::Expression, Parser}};

// pub struct Repl<R: Read,W: Write> {
//     reader: BufReader<R>,
//     writer: W,
//     buf: Vec<u8>,
//     lexer: Option<Lexer>,
//     parser: Option<Parser>,
// }

// impl<R: Read, W: Write> Repl<R, W> {
//     pub fn new(reader: R, writer: W) -> Self {
//         Self {
//             reader: BufReader::new(reader),
//             writer,
//             buf: Vec::new(),
//             lexer: Some(Lexer::empty()),
//             parser: Some(Parser::empty()),
//         }
//     }

//     fn read_line(&mut self) -> Result<(), Box<dyn Error>> {
//         // reads a line from the reader and gives it to the lexer
//         self.reader.read_until(b'\n', &mut self.buf);
//         self.lexer.as_mut().unwrap().with_src(&self.buf);
//         Ok(())
//     }

//     fn lex(&mut self) -> Result<(), Box<dyn Error>> {
//         let tokens = self.lexer.take().unwrap().run()?;
//         self.parser.as_mut().unwrap().add_tokens(&tokens);
//         Ok(())
//     }

//     fn parse(&mut self) -> Result<Expression, Box<dyn Error>> {
//         self.parser.as_mut().unwrap().parse()
//     }

//     pub fn go(&mut self) {
//         match self.read_line() {
//             Err(e) => self.writer.write_fmt(format_args!("{:?}", e)).unwrap(),
//             _ => (),
//         };
//         match self.lex() {
//             Err(e) => self.writer.write_fmt(format_args!("{:?}", e)).unwrap(),
//             _ => (),
//         };
//         match self.parse() {
//             Err(e) => self.writer.write_fmt(format_args!("{:?}", e)).unwrap(),
//             Ok(expr) => self.writer.write_fmt(format_args!("{:?}", expr)).unwrap(),
//         };
//     }

//     pub fn go_lines(&mut self) {
//         self.r
//         todo!()
//     }
// }
