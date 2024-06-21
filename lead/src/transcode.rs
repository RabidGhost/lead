use crate::air::air::Inst;
use crate::lex::span::Span;

pub struct Line {
    text: String,
    span: Span,
}

pub trait Transcoder {
    fn transcode(instructions: Vec<Inst>) -> Vec<Line>;
}
