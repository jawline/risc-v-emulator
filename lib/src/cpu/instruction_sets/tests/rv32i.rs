use super::*;
use crate::cpu::instruction_sets::rv32i::{CpuState, InstructionTable};
use crate::instruction::encoder::{self, Instruction};
use crate::memory::Memory;

const fn pack_negative_into_12b(val: i16) -> u16 {
    if val < -2048 || val > 2047 {
        panic!("12b signed value is out of range");
    }

    (val as u16) & 0b0000_1111_1111_1111
}

#[test]
fn create_table() {
    let _table = InstructionTable::new();
}

struct TestEnvironment {
    state: CpuState,
    memory: Memory,
    tbl: InstructionTable,
    last_pc: u32,
}

impl TestEnvironment {
    fn new() -> Self {
        TestEnvironment {
            state: CpuState::new(),
            memory: Memory::new(4096),
            tbl: InstructionTable::new(),
            last_pc: 0,
        }
    }

    fn step(&mut self, instruction: &Instruction) {
        self.last_pc = self.state.registers.pc;
        self.tbl
            .step(&mut self.state, &mut self.memory, instruction.encode());
    }

    /// Step function that applys some default checks
    fn dbg_step(&mut self, instruction: &Instruction) {
        self.step(instruction);
        assert_eq!(self.last_pc + 4, self.state.registers.pc);
    }

    fn set_register(&mut self, index: usize, value: i32) {
        self.state.registers.seti(index, value);
    }

    fn expect_register(&self, index: usize, value: i32) {
        assert_eq!(self.state.registers.geti(index), value)
    }

    fn expect_all_register(&self, value: i32) {
        for i in 0..32 {
            self.expect_register(i, value);
        }
    }
}

fn init() -> TestEnvironment {
    TestEnvironment::new()
}

#[test]
fn execute_no_op() {
    let mut test = init();
    test.dbg_step(&encoder::no_op());
    test.expect_all_register(0);
}

#[test]
fn execute_addi() {
    let mut test = init();

    // Test INCR r1
    test.dbg_step(&encoder::addi(1, 1, 1));
    test.expect_register(1, 1);
    test.dbg_step(&encoder::addi(1, 1, 1));
    test.expect_register(1, 2);
    test.dbg_step(&encoder::addi(1, 1, 1));
    test.expect_register(1, 3);

    // Test r2 = r1 + 1
    test.dbg_step(&encoder::addi(2, 1, 1));
    test.expect_register(2, 4);
    test.expect_register(1, 3);

    // Test r2 = r1 - 1
    test.dbg_step(&encoder::addi(2, 1, pack_negative_into_12b(-1)));
    test.expect_register(1, 3);
    test.expect_register(2, 2);

    // Test r3 = r2 + 4
    test.dbg_step(&encoder::addi(3, 2, pack_negative_into_12b(4)));
    test.expect_register(1, 3);
    test.expect_register(2, 2);
    test.expect_register(3, 6);
}

#[test]
fn execute_addi_overflow() {
    unimplemented!();
}

#[test]
fn execute_slti() {
    let mut test = init();

    // Test positive
    test.set_register(1, 4);
    test.dbg_step(&encoder::slti(2, 1, 4));
    test.expect_register(1, 5);
    test.expect_register(2, 0);

    test.dbg_step(&encoder::slti(2, 1, 5));
    test.expect_register(1, 5);
    test.expect_register(2, 0);

    test.dbg_step(&encoder::slti(2, 1, 6));
    test.expect_register(1, 5);
    test.expect_register(2, 1);

    // Test max int
    test.set_register(1, 2047);
    test.dbg_step(&encoder::slti(2, 1, 2047));
    test.expect_register(2, 0);

    test.set_register(1, 2046);
    test.dbg_step(&encoder::slti(2, 1, 2047));
    test.expect_register(2, 1);

    // Test negative
    test.set_register(1, -5000);
    test.dbg_step(&encoder::slti(2, 1, pack_negative_into_12b(-2048)));
    test.expect_register(2, 1);

    test.set_register(1, -2000);
    test.dbg_step(&encoder::slti(2, 1, pack_negative_into_12b(-2048)));
    test.expect_register(2, 0);
}

#[test]
fn execute_sltiu() {
    let mut test = init();

    test.set_register(1, 0);
    test.dbg_step(&encoder::sltiu(2, 1, 0));
    test.expect_register(2, 0);

    test.dbg_step(&encoder::sltiu(2, 1, 1));
    test.expect_register(2, 1);

    test.set_register(1, 2);
    test.dbg_step(&encoder::sltiu(2, 1, 1));
    test.expect_register(2, 0);

    test.set_register(1, 2048);
    test.dbg_step(&encoder::sltiu(2, 1, 2047));
    test.expect_register(2, 0);

    test.set_register(1, 2046);
    test.dbg_step(&encoder::sltiu(2, 1, 2047));
    test.expect_register(2, 1);
}

#[test]
fn execute_andi() {
    let mut test = init();

    test.set_register(1, 0);
    test.dbg_step(&encoder::andi(2, 1, 1));
    test.expect_register(2, 0);

    test.set_register(1, 0b0110);
    test.dbg_step(&encoder::andi(2, 1, 0b0101));
    test.expect_register(2, 0b0100);
}

#[test]
fn execute_ori() {
    let mut test = init();

    test.set_register(1, 0);
    test.dbg_step(&encoder::ori(2, 1, 1));
    test.expect_register(2, 1);

    test.set_register(1, 0b0110);
    test.dbg_step(&encoder::ori(2, 1, 0b0101));
    test.expect_register(2, 0b0111);
}

#[test]
fn execute_xori() {
    let mut test = init();

    test.set_register(1, 0);
    test.dbg_step(&encoder::xori(2, 1, 1));
    test.expect_register(2, 1);

    test.set_register(1, 0b0110);
    test.dbg_step(&encoder::xori(2, 1, 0b0101));
    test.expect_register(2, 0b0011);
}

#[test]
fn execute_slli() {
    let mut test = init();
    test.set_register(1, 0);
    test.dbg_step(&encoder::slli(2, 1, 1));
    test.expect_register(2, 0);
    test.set_register(1, 0b0110);
    test.dbg_step(&encoder::slli(2, 1, 1));
    test.expect_register(2, 0b1100);
    test.dbg_step(&encoder::slli(2, 1, 2));
    test.expect_register(2, 0b11000);
    test.dbg_step(&encoder::slli(2, 1, 3));
    test.expect_register(2, 0b110000);
    test.dbg_step(&encoder::slli(2, 1, 32));
    test.expect_register(2, 0b0);
    test.set_register(1, -1);
    test.dbg_step(&encoder::slli(2, 1, 16));
    test.expect_register(2, 0b1111_1111_1111_1111_0000_0000_0000_0000u32 as i32);
}

#[test]
fn execute_srli() {
    let mut test = init();
    test.set_register(1, 0);
    test.dbg_step(&encoder::srli(2, 1, 1));
    test.expect_register(2, 0);
    test.set_register(1, 0b0110);
    test.dbg_step(&encoder::srli(2, 1, 1));
    test.expect_register(2, 0b0011);
    test.dbg_step(&encoder::srli(2, 1, 2));
    test.expect_register(2, 0b0001);
    test.dbg_step(&encoder::srli(2, 1, 3));
    test.expect_register(2, 0b0000);
    test.set_register(1, -1);
    test.dbg_step(&encoder::srli(2, 1, 16));
    test.expect_register(2, 0b0000_0000_0000_0000_1111_1111_1111_1111u32 as i32);
}

#[test]
fn execute_srai() {
    let mut test = init();
    test.set_register(1, 0);
    test.dbg_step(&encoder::srai(2, 1, 1));
    test.expect_register(2, 0);
    test.set_register(1, 0b0110);
    test.dbg_step(&encoder::srai(2, 1, 1));
    test.expect_register(2, 0b0011);
    test.dbg_step(&encoder::srai(2, 1, 2));
    test.expect_register(2, 0b0001);
    test.dbg_step(&encoder::srai(2, 1, 3));
    test.expect_register(2, 0b0000);
    test.set_register(1, -1);
    test.dbg_step(&encoder::srai(2, 1, 16));
    test.expect_register(2, 0b1111_1111_1111_1111_1111_1111_1111_1111u32 as i32);
}

#[test]
fn execute_add() {
    let mut test = init();

    test.set_register(1, 5);
    test.set_register(2, 10);
    test.dbg_step(&encoder::add(1, 1, 2));
    test.expect_register(1, 15);
    test.expect_register(2, 10);

    test.set_register(2, -5);
    test.dbg_step(&encoder::add(2, 1, 2));
    test.expect_register(1, 15);
    test.expect_register(2, 10);
}

#[test]
fn execute_add_overflow() {
    unimplemented!();
}

#[test]
fn execute_sub_overflow() {
    let mut test = init();

    test.set_register(1, 5);
    test.set_register(2, 10);
    test.dbg_step(&encoder::sub(1, 1, 2));
    test.expect_register(1, -5);
    test.expect_register(2, 10);

    test.set_register(2, -5);
    test.dbg_step(&encoder::sub(2, 1, 2));
    test.expect_register(1, -5);
    test.expect_register(2, -0);
}

#[test]
fn execute_sub_underflow() {
    unimplemented!();
}

#[test]
fn execute_slt() {
    unimplemented!();
}

#[test]
fn execute_sltu() {
    unimplemented!();
}

#[test]
fn execute_and() {
    unimplemented!();
}

#[test]
fn execute_or() {
    unimplemented!();
}

#[test]
fn execute_xor() {
    unimplemented!();
}

#[test]
fn execute_sll() {
    unimplemented!();
}

#[test]
fn execute_srl() {
    unimplemented!();
}

#[test]
fn execute_sra() {
    unimplemented!();
}

#[test]
fn execute_lui() {
    unimplemented!();
}

#[test]
fn execute_auipc() {
    unimplemented!();
}

#[test]
fn execute_jal() {
    unimplemented!();
}

#[test]
fn execute_jalr() {
    unimplemented!();
}

#[test]
fn execute_jalr_result_is_misaligned() {
    unimplemented!();
}

#[test]
fn execute_beq() {
    unimplemented!();
}

#[test]
fn execute_bne() {
    unimplemented!();
}

#[test]
fn execute_blt() {
    unimplemented!();
}

#[test]
fn execute_bge() {
    unimplemented!();
}

#[test]
fn execute_bltu() {
    unimplemented!();
}

#[test]
fn execute_bgeu() {
    unimplemented!();
}

#[test]
fn execute_lb() {
    unimplemented!();
}

#[test]
fn execute_lh() {
    unimplemented!();
}

#[test]
fn execute_lw() {
    unimplemented!();
}

#[test]
fn execute_lbu() {
    unimplemented!();
}

#[test]
fn execute_lhu() {
    unimplemented!();
}

#[test]
fn execute_sb() {
    unimplemented!();
}

#[test]
fn execute_sh() {
    unimplemented!();
}

#[test]
fn execute_sw() {
    unimplemented!();
}

#[test]
fn execute_fence() {
    unimplemented!();
}

#[test]
fn execute_fence_i() {
    unimplemented!();
}
