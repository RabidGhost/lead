use lead_vm::air::Instruction;
use leadc::pipeline::Pipeline;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn build_from_string(string: String) -> String {
    let instructions: Vec<Instruction> = match Pipeline::Text(string, None).lex() {
        Err(err) => return err.to_string(),
        Ok(pipeline) => match pipeline.parse() {
            Err(err) => return err.to_string(),
            Ok(pipeline) => match pipeline.build() {
                Err(err) => return err.to_string(),
                Ok(pipeline) => pipeline.into(),
            },
        },
    };

    let mut buf: String = String::new();

    for instruction in instructions {
        buf.push_str(&format!("{instruction}"));
    }
    buf
}
