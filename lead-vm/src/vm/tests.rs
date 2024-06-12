use std::sync::mpsc::channel;

use super::*;
use crate::air::Instruction;

#[test]
fn r#yield() {
    let instructions = vec![Instruction::CON(Reg(0), 5), Instruction::YLD(Reg(0))];
    let (sndr, recvr) = channel();
    let mut vm = Machine::new(instructions, sndr);
    vm.run();
    assert_eq!(Ok(Message::Yield(5)), recvr.recv())
}

#[test]
fn branch_simple() {
    let instructions = vec![
        Instruction::BRA("label".to_owned()),
        Instruction::CON(Reg(0), 5),
        Instruction::YLD(Reg(0)),
        Instruction::LBL("label".to_owned()),
        Instruction::CON(Reg(0), 17),
        Instruction::YLD(Reg(0)),
    ];
    let (sndr, recvr) = channel();
    let mut vm = Machine::new(instructions, sndr);
    vm.run();
    assert_eq!(Ok(Message::Yield(17)), recvr.recv())
}
