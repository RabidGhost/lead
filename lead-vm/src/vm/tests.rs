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

#[test]
fn store_and_load() {
    let instructions = vec![
        Instruction::CON(Reg(0), 0xdeadbeef),         // mov r0, #0xdead
        Instruction::CON(Reg(1), 0),                  // mov r1, #0
        Instruction::STR(Reg(0), Reg(1), Mode::None), // str r0, [r1]
        Instruction::ADD(Reg(1), Reg(1), Reg(1)),     // nop
        Instruction::LDR(Reg(2), Reg(1), Mode::None), // ldr r2, [r1]
        Instruction::YLD(Reg(2)),                     // yield r2
    ];

    let memory: Vec<u8> = vec![0, 0, 0, 0];

    let (sndr, recvr) = channel();
    let mut vm = Machine::with_memory(instructions, sndr, memory);
    vm.run();

    assert_eq!(vm.memory, vec![0xde, 0xad, 0xbe, 0xef]);
    assert_eq!(Ok(Message::Yield(0xdeadbeef)), recvr.recv())
}
