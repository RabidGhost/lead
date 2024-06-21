use lead::air::air::{Inst, Instruction};
use lead::lex::token::Token;
use leadc::pipeline::Pipeline;
use serde::Serialize;
use sourcespan::SourceSpan;
use wasm_bindgen::prelude::*;

mod sourcespan;

#[derive(Serialize)]
pub struct Message {
    ed_lang_spans: Vec<SourceSpan>,
    air_text: String,
    air_spans: Vec<SourceSpan>,
}

#[wasm_bindgen]
pub fn build_from_string(string: String) -> String {
    let instructions: Vec<Instruction> = match Pipeline::Text(string, None).lex() {
        Err(err) => return err.to_string(),
        Ok(pipeline) => match pipeline.parse() {
            Err(err) => return err.to_string(),
            Ok(pipeline) => match pipeline.build() {
                Err(err) => return err.to_string(),
                Ok(pipeline) => pipeline.try_into().unwrap(),
            },
        },
    };

    let mut buf: String = String::new();

    for instruction in instructions {
        buf.push_str(&format!("{instruction}"));
    }
    buf
}

#[wasm_bindgen]
pub fn build_and_generate_spans(string: String) -> String {
    let lexed_pipeline = match Pipeline::Text(string, None).lex() {
        Err(err) => return err.to_string(),
        Ok(pipeline) => pipeline,
    };

    let tokens: Vec<Token> = lexed_pipeline.clone().into();

    let ed_lang_spans: Vec<SourceSpan> = tokens.iter().map(|tok| tok.span().into()).collect();

    let instructions: Vec<Inst> = match lexed_pipeline.parse() {
        Err(err) => return err.to_string(),
        Ok(pipeline) => match pipeline.build() {
            Err(err) => return err.to_string(),
            Ok(pipeline) => pipeline.try_into().unwrap(),
        },
    };

    let (air_text, air_spans) = sourcespan::render_and_compute_spans(instructions);

    let message = Message {
        ed_lang_spans,
        air_text,
        air_spans,
    };

    let serialized = serde_json::to_string(&message).unwrap();
    serialized
    // let mut buf: String = String::new();

    // for instruction in instructions {
    //     buf.push_str(&format!("{instruction}"));
    // }
    // buf
}
