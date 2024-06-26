use super::*;
use lead::air::air::{
    Flag,
    Instruction::{self, *},
};
use log::{info, warn};
use ntest::timeout;
use std::sync::mpsc::channel;

use test_log::test;

const NO_FLAGS: VMFlags = VMFlags::none();
const VERY_VERBOSE: VMFlags = VMFlags::new(DEFAULT_MEMORY_SIZE, Verbosity::VeryVerbose as u8);

const R0: Reg = Reg(0);
const R1: Reg = Reg(1);
const R2: Reg = Reg(2);
const R3: Reg = Reg(3);
// const R4: Reg = Reg(4);
// const R5: Reg = Reg(5);

// fn init() {
//     let _ = env_logger::builder().is_test(true).try_init();
// }
//
#[test]
fn it_works() {
    info!("Checking whether it still works...");
    assert_eq!(2 + 2, 4);
    info!("Looks good!");
}

#[test]
fn r#yield() {
    warn!("checking!");
    let instructions = vec![CON(R0, 5), YLD(R0)];
    let (sndr, recvr) = channel();
    let mut vm = Machine::new(instructions, sndr, VERY_VERBOSE);
    vm.run();
    assert_eq!(Ok(Message::Yield(5)), recvr.recv())
}

#[test]
fn branch_simple() {
    let instructions = vec![
        BRA("label".to_owned()),
        CON(R0, 5),
        YLD(R0),
        LBL("label".to_owned()),
        CON(R0, 17),
        YLD(R0),
    ];
    let (sndr, recvr) = channel();
    let mut vm = Machine::new(instructions, sndr, NO_FLAGS);
    vm.run();
    assert_eq!(Ok(Message::Yield(17)), recvr.recv())
}

#[test]
fn store_and_load() {
    let instructions = vec![
        CON(R0, 0xdeadbeef),     // mov r0, #0xdeadbeef
        CON(R1, 0),              // mov r1, #0
        STR(R0, R1, Mode::None), // str r0, [r1]
        ADD(R1, R1, R1),         // nop
        LDR(R2, R1, Mode::None), // ldr r2, [r1]
        YLD(R2),                 // yield r2
    ];

    let (sndr, recvr) = channel();
    let mut vm = Machine::new(instructions, sndr, NO_FLAGS);
    vm.run();

    assert_eq!(vm.memory[0..4], vec![0xde, 0xad, 0xbe, 0xef]);
    assert_eq!(Ok(Message::Yield(0xdeadbeef)), recvr.recv())
}

#[timeout(1000)]
#[test]
fn loop_not_taken() {
    let instructions = vec![
        CON(R0, 37),                   // mov r0, #37
        LBL("check-condition".into()), // check-condition:
        CON(R1, 12),                   //     mov r1, 12
        CMP(R0, R1, None),             //     cmp r0, r1
        CHK(Flag::Gt),                 //
        BRA("break".into()),           //     bgt break
        YLD(R1),                       //     yld r1
        BRA("check-condition".into()), //     b check-condition
        LBL("break".into()),           // break:
        CON(R2, 0xbeef),               //     mov r2, #0xbeef
        YLD(R2),                       //     yld r2
    ];

    let (sndr, recvr) = channel();
    let mut vm = Machine::new(instructions, sndr, VERY_VERBOSE);
    vm.run();

    assert_eq!(Ok(Message::Yield(0xbeef)), recvr.recv())
}

#[test]
#[timeout(1000)]
fn loop_and_break() {
    let instructions = vec![
        CON(R0, 1),                    // mov r0, #1
        LBL("check-condition".into()), // check-condition:
        CON(R1, 5),                    //      mov r1, #5
        CMP(R0, R1, None),             //      cmp r0, r1
        CHK(Flag::Ge),                 //
        BRA("break".into()),           //      bge break
        YLD(R0),                       //      yld r0
        CON(R2, 1),                    //      mov r2, #1
        ADD(R0, R0, R2),               //      add r0, r2
        BRA("check-condition".into()), //      b check-condition
        LBL("break".into()),           // break:
        CON(R3, 64),                   // mov r3, #64
        YLD(R3),                       // yld r3
    ];

    let (sndr, recvr) = channel();
    let mut vm = Machine::new(instructions, sndr, NO_FLAGS);
    vm.run();

    assert_eq!(Ok(Message::Yield(1)), recvr.recv());
    assert_eq!(Ok(Message::Yield(2)), recvr.recv());
    assert_eq!(Ok(Message::Yield(3)), recvr.recv());
    assert_eq!(Ok(Message::Yield(4)), recvr.recv());
    assert_eq!(Ok(Message::Yield(64)), recvr.recv())
}
