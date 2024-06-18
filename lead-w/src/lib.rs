use lead_vm::air::Instruction;
use leadc::pipeline::Pipeline;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn build_from_string(string: String) -> String {
    let instructions: Vec<Instruction> = Pipeline::Text(string, None)
        .lex()
        .unwrap()
        .parse()
        .unwrap()
        .build()
        .unwrap()
        .into();

    let mut buf: String = String::new();

    for instruction in instructions {
        buf.push_str(&format!("{instruction}"));
    }
    buf
}
