use lead::{air::air::Inst, lex::span::*};
use primes::factors_uniq;
use serde::Serialize;

#[derive(Serialize)]
pub struct SourceSpan {
    start: usize,
    end: usize,
    ids: Vec<u64>,
}

impl Into<SourceSpan> for Span {
    fn into(self) -> SourceSpan {
        SourceSpan {
            start: self.span().0,
            end: self.span().1,
            ids: factors_uniq(self.id()),
        }
    }
}

pub fn render_and_compute_spans(instructions: Vec<Inst>) -> (String, Vec<SourceSpan>) {
    let mut rendered_instructions: String = String::new();
    let mut spans: Vec<SourceSpan> = Vec::new();

    for inst in instructions {
        let Inst { instruction, span } = inst;
        let start = rendered_instructions.len();
        rendered_instructions.push_str(&format!("{instruction}\n"));
        let end = rendered_instructions.len();
        // this might need to be len - 1
        //

        spans.push(SourceSpan {
            start,
            end,
            ids: factors_uniq(span.id()),
        });
    }
    (rendered_instructions, spans)
}
