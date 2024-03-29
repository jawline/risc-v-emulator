use crate::cpu::instruction_sets::rv32i::{CpuState, InstructionSet, OpArgs};
use crate::instruction::encoder::{self, Instruction};
use crate::memory::Memory;

struct TestEnvironment {
    state: CpuState,
    memory: Memory,
    tbl: InstructionSet,
}

impl TestEnvironment {
    fn new() -> Self {
        TestEnvironment {
            state: CpuState::new(),
            memory: Memory::new(4096),
            tbl: InstructionSet::new(),
        }
    }

    fn step_with_ecall<F: FnOnce(&mut OpArgs) -> ()>(
        &mut self,
        instruction: &Instruction,
        ecall: F,
    ) {
        self.tbl.step(
            &mut self.state,
            &mut self.memory,
            instruction.encode(),
            ecall,
        );
    }

    fn step(&mut self, instruction: &Instruction) {
        self.step_with_ecall(instruction, |_op| {});
    }

    /// Step function that asserts an expected new PC
    fn dbg_step_jmp(&mut self, instruction: &Instruction, expected_new_pc: u32) {
        self.step(instruction);
        assert_eq!(expected_new_pc, self.state.registers.pc);
    }

    /// Step function that checks the PC is incremented by 4
    fn dbg_step(&mut self, instruction: &Instruction) {
        let pc = self.state.registers.pc;
        self.dbg_step_jmp(instruction, pc + 4);
    }

    fn get_register(&mut self, index: usize) -> i32 {
        self.state.registers.geti(index)
    }

    fn set_register(&mut self, index: usize, value: i32) {
        self.state.registers.seti(index, value);
    }

    fn set_pc(&mut self, pc: u32) {
        self.state.registers.pc = pc;
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
    test.dbg_step(&encoder::addi(2, 1, -1));
    test.expect_register(1, 3);
    test.expect_register(2, 2);

    // Test r3 = r2 + 4
    test.dbg_step(&encoder::addi(3, 2, 4));
    test.expect_register(1, 3);
    test.expect_register(2, 2);
    test.expect_register(3, 6);
}

#[test]
fn execute_addi_overflow() {
    let mut test = init();

    test.set_register(1, i32::MAX);
    test.dbg_step(&encoder::addi(1, 1, 1));
    test.expect_register(1, i32::MIN);

    test.set_register(1, i32::MIN);
    test.dbg_step(&encoder::addi(1, 1, -1));
    test.expect_register(1, i32::MAX);
}

#[test]
fn execute_slti() {
    let mut test = init();

    // Test positive
    test.set_register(1, 5);
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
    test.dbg_step(&encoder::slti(2, 1, -2048));
    test.expect_register(2, 1);

    test.set_register(1, -2000);
    test.dbg_step(&encoder::slti(2, 1, -2048));
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
    test.dbg_step(&encoder::slli(2, 1, 31));
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
    let mut test = init();

    test.set_register(1, i32::MIN);
    test.set_register(2, -1);
    test.dbg_step(&encoder::add(1, 1, 2));
    test.expect_register(1, i32::MAX);

    test.set_register(1, i32::MAX);
    test.set_register(2, 1);
    test.dbg_step(&encoder::add(1, 1, 2));
    test.expect_register(1, i32::MIN);
}

#[test]
fn execute_sub() {
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
fn execute_sub_overflow() {
    let mut test = init();

    test.set_register(1, i32::MAX);
    test.set_register(2, -1);
    test.dbg_step(&encoder::sub(1, 1, 2));
    test.expect_register(1, i32::MIN);

    test.set_register(1, i32::MIN);
    test.set_register(2, 1);
    test.dbg_step(&encoder::sub(1, 1, 2));
    test.expect_register(1, i32::MAX);
}

#[test]
fn execute_slt() {
    let mut test = init();

    // Test positive
    test.set_register(1, 4);
    test.set_register(2, 4);
    test.dbg_step(&encoder::slt(3, 1, 2));
    test.expect_register(3, 0);

    test.set_register(1, 3);
    test.dbg_step(&encoder::slt(3, 1, 2));
    test.expect_register(3, 1);

    // Test max int
    test.set_register(1, 2147483647);
    test.set_register(2, 2147483647);
    test.dbg_step(&encoder::slt(3, 1, 2));
    test.expect_register(3, 0);

    test.set_register(1, 2147483646);
    test.dbg_step(&encoder::slt(3, 1, 2));
    test.expect_register(3, 1);

    // Test negative
    test.set_register(1, -2147483648);
    test.set_register(2, -2147483648);
    test.dbg_step(&encoder::slt(3, 1, 2));
    test.expect_register(3, 0);

    test.set_register(2, -2147483647);
    test.dbg_step(&encoder::slt(3, 1, 2));
    test.expect_register(3, 1);
}

#[test]
fn execute_sltu() {
    let mut test = init();

    // Test positive
    test.set_register(1, 0);
    test.set_register(2, 0);
    test.dbg_step(&encoder::sltu(3, 1, 2));
    test.expect_register(3, 0);

    test.set_register(1, 1);
    test.set_register(2, 0);
    test.dbg_step(&encoder::sltu(3, 1, 2));
    test.expect_register(3, 0);
    test.dbg_step(&encoder::sltu(3, 2, 1));
    test.expect_register(3, 1);

    // Test max int
    test.set_register(1, 4294967295u32 as i32);
    test.set_register(2, 4294967295u32 as i32);
    test.dbg_step(&encoder::sltu(3, 1, 2));
    test.expect_register(3, 0);

    test.set_register(1, 4294967293u32 as i32);
    test.dbg_step(&encoder::sltu(3, 1, 2));
    test.expect_register(3, 1);
}

#[test]
fn execute_and() {
    let mut test = init();
    test.set_register(1, 0b1110010101);
    test.set_register(2, 0b0010101101);
    test.dbg_step(&encoder::and(3, 1, 2));
    test.expect_register(3, 0b0010000101);
}

#[test]
fn execute_or() {
    let mut test = init();
    test.set_register(1, 0b1110010101);
    test.set_register(2, 0b0010101101);
    test.dbg_step(&encoder::or(3, 1, 2));
    test.expect_register(3, 0b1110111101);
}

#[test]
fn execute_xor() {
    let mut test = init();
    test.set_register(1, 0b1110010101);
    test.set_register(2, 0b0010101101);
    test.dbg_step(&encoder::xor(3, 1, 2));
    test.expect_register(3, 0b1100111000);
}

#[test]
fn execute_sll() {
    let mut test = init();
    test.set_register(1, 0b11100000_00000000_00000011_10010101u32 as i32);
    test.set_register(2, 2);
    test.dbg_step(&encoder::sll(3, 1, 2));
    test.expect_register(3, 0b100000_00000000_00000011_1001010100u32 as i32);
}

#[test]
fn execute_srl() {
    let mut test = init();
    test.set_register(1, -1);
    test.set_register(2, 16);
    test.dbg_step(&encoder::srl(3, 1, 2));
    println!("{:032b}", test.get_register(1));
    println!("{:032b}", test.get_register(2));
    println!("{:032b}", test.get_register(3));
    test.expect_register(3, 0b00000000_00000000_11111111_11111111u32 as i32);
}

#[test]
fn execute_sra() {
    let mut test = init();
    test.set_register(1, -1);
    test.set_register(2, 16);
    test.dbg_step(&encoder::sra(3, 1, 2));
    test.expect_register(3, 0b11111111_11111111_11111111_11111111u32 as i32);
}

#[test]
fn execute_lui() {
    let value = 0b1101_1111_0101_1010_0101_0000_0000_0000u32;
    let mut test = init();
    test.set_register(1, -1);
    test.dbg_step(&encoder::lui(1, value));
    test.expect_register(1, value as i32);
}

#[test]
fn execute_auipc() {
    let mut test = init();
    let value = 0b1101_1111_0101_1010_0101_0000_0000_0000u32;
    test.set_pc(0b1010_1010_1010);
    test.set_register(1, -1);
    test.dbg_step(&encoder::auipc(1, value));
    test.expect_register(1, 0b1101_1111_0101_1010_0101_1010_1010_1010u32 as i32);
}

#[test]
fn execute_jal() {
    let mut test = init();
    test.set_pc(5000);
    test.set_register(1, -1);
    test.dbg_step_jmp(&encoder::jal(1, 500), 5500);
    test.expect_register(1, 5004);

    test.set_pc(5000);
    test.set_register(1, -1);
    test.dbg_step_jmp(&encoder::jal(2, -500), 4500);
    test.expect_register(2, 5004);
}

#[test]
fn execute_jalr() {
    let mut test = init();

    test.set_pc(5000);
    test.set_register(1, 9000);
    test.dbg_step_jmp(&encoder::jalr(1, 1, 500), 9500);
    test.expect_register(1, 5004);

    test.set_pc(5000);
    test.set_register(1, 9000);
    test.dbg_step_jmp(&encoder::jalr(2, 1, -500), 8500);
    test.expect_register(2, 5004);
}

#[test]
#[should_panic]
fn execute_jalr_result_is_misaligned_1_bit() {
    let mut test = init();

    test.set_pc(5000);
    test.set_register(1, 9001);
    // This should trap (represented as a panic) on an unaligned instruction address
    test.dbg_step(&encoder::jalr(1, 1, 500));
}

#[test]
#[should_panic]
fn execute_jalr_result_is_misaligned_2_bit() {
    let mut test = init();

    test.set_pc(5000);
    test.set_register(1, 9002);
    // This should trap (represented as a panic) on an unaligned instruction address
    test.dbg_step(&encoder::jalr(1, 1, 500));
}

#[test]
fn execute_beq() {
    let mut test = init();

    test.set_pc(5000);
    test.set_register(1, 0);
    test.dbg_step_jmp(&encoder::beq(1, 0, 500), 5500);

    test.set_pc(5000);
    test.set_register(1, 0);
    test.dbg_step_jmp(&encoder::beq(1, 0, -500), 4500);

    test.set_pc(5000);
    test.set_register(1, 500);
    test.dbg_step(&encoder::beq(1, 0, 500));
}

#[test]
fn execute_bne() {
    let mut test = init();

    test.set_pc(5000);
    test.set_register(1, 0);
    test.dbg_step(&encoder::bne(1, 0, 500));

    test.set_pc(5000);
    test.set_register(1, 500);
    test.dbg_step_jmp(&encoder::bne(1, 0, 500), 5500);

    test.set_pc(5000);
    test.set_register(1, 500);
    test.dbg_step_jmp(&encoder::bne(1, 0, -500), 4500);
}

#[test]
fn execute_blt() {
    let mut test = init();

    test.set_pc(5000);
    test.set_register(1, 0);
    test.dbg_step(&encoder::blt(1, 0, 500));

    test.set_pc(5000);
    test.set_register(1, 1);
    test.dbg_step(&encoder::blt(1, 0, 500));

    test.set_pc(5000);
    test.set_register(1, -1);
    test.dbg_step_jmp(&encoder::blt(1, 0, 500), 5500);

    test.set_pc(5000);
    test.set_register(1, -1);
    test.dbg_step_jmp(&encoder::blt(1, 0, -500), 4500);
}

#[test]
fn execute_bge() {
    let mut test = init();

    test.set_pc(5000);
    test.set_register(1, -1);
    test.dbg_step(&encoder::bge(1, 0, 500));

    test.set_pc(5000);
    test.set_register(1, 0);
    test.dbg_step_jmp(&encoder::bge(1, 0, 500), 5500);

    test.set_pc(5000);
    test.set_register(1, 0);
    test.dbg_step_jmp(&encoder::bge(1, 0, -500), 4500);

    test.set_pc(5000);
    test.set_register(1, 1);
    test.dbg_step_jmp(&encoder::bge(1, 0, 500), 5500);

    test.set_pc(5000);
    test.set_register(1, 1);
    test.dbg_step_jmp(&encoder::bge(1, 0, -500), 4500);
}

#[test]
fn execute_bltu() {
    let mut test = init();

    test.set_pc(5000);
    test.set_register(1, 0);
    test.dbg_step(&encoder::bltu(1, 0, 500));

    test.set_pc(5000);
    test.set_register(1, 1);
    test.dbg_step(&encoder::bltu(1, 0, 500));

    test.set_pc(5000);
    test.set_register(1, -1);
    test.dbg_step(&encoder::bltu(1, 0, 500));

    test.set_pc(5000);
    test.set_register(1, 0);
    test.set_register(2, 50);
    test.dbg_step_jmp(&encoder::bltu(1, 2, 500), 5500);

    test.set_pc(5000);
    test.set_register(1, 0);
    test.set_register(2, 50);
    test.dbg_step_jmp(&encoder::bltu(1, 2, -500), 4500);
}

#[test]
fn execute_bgeu() {
    let mut test = init();

    test.set_pc(5000);
    test.set_register(1, 0);
    test.dbg_step_jmp(&encoder::bgeu(1, 0, 500), 5500);

    test.set_pc(5000);
    test.set_register(1, 1);
    test.dbg_step_jmp(&encoder::bgeu(1, 0, 500), 5500);

    test.set_pc(5000);
    test.set_register(1, -1);
    test.dbg_step_jmp(&encoder::bgeu(1, 0, 500), 5500);

    test.set_pc(5000);
    test.set_register(1, 0);
    test.set_register(2, 50);
    test.dbg_step(&encoder::bgeu(1, 2, 500));

    test.set_pc(5000);
    test.set_register(1, 0);
    test.set_register(2, 50);
    test.dbg_step(&encoder::bgeu(1, 2, -500));
}

#[test]
fn execute_lb() {
    let mut test = init();
    test.memory.set8(499, 10).unwrap();
    test.memory.set8(500, 50).unwrap();
    test.memory.set8(501, 25).unwrap();
    test.memory.set8(502, 0xFF).unwrap();
    test.set_register(1, 500);
    test.set_register(2, 0);
    test.dbg_step(&encoder::lb(1, 2, 0));
    assert_eq!(test.get_register(1), 500);
    assert_eq!(test.get_register(2), 50);

    test.dbg_step(&encoder::lb(1, 2, 1));
    assert_eq!(test.get_register(1), 500);
    assert_eq!(test.get_register(2), 25);

    test.dbg_step(&encoder::lb(1, 2, -1));
    assert_eq!(test.get_register(1), 500);
    assert_eq!(test.get_register(2), 10);

    // Test sign extension
    test.dbg_step(&encoder::lb(1, 2, 2));
    assert_eq!(test.get_register(1), 500);
    assert_eq!(test.get_register(2), -1);
}

#[test]
fn execute_lh() {
    let mut test = init();

    test.memory.set8(501, 0b0101_1100).unwrap();
    test.memory.set8(500, 0b1010_1010).unwrap();
    test.set_register(1, 500);
    test.set_register(2, 0);
    test.dbg_step(&encoder::lh(1, 2, 0));
    assert_eq!(test.get_register(1), 500);
    assert_eq!(test.get_register(2), 0b0101_1100_1010_1010);

    // Test immediate offset
    test.memory.set8(503, 0b0111_1100).unwrap();
    test.memory.set8(502, 0b1010_1110).unwrap();
    test.set_register(1, 500);
    test.set_register(2, 0);
    test.dbg_step(&encoder::lh(1, 2, 2));
    assert_eq!(test.get_register(1), 500);
    assert_eq!(test.get_register(2), 0b0111_1100_1010_1110);

    // Test that the LH is sign extended by default
    test.memory.set8(501, 0b1101_1100).unwrap();
    test.memory.set8(500, 0b1010_1010).unwrap();
    test.set_register(1, 500);
    test.set_register(2, 0);
    test.dbg_step(&encoder::lh(1, 2, 0));
    assert_eq!(test.get_register(1), 500);
    assert_eq!(
        test.get_register(2),
        0b1111_1111_1111_1111_1101_1100_1010_1010u32 as i32
    );
}

#[test]
fn execute_lw() {
    let mut test = init();

    test.memory.set8(503, 0b1111_1111).unwrap();
    test.memory.set8(502, 0b0000_0000).unwrap();
    test.memory.set8(501, 0b0101_1100).unwrap();
    test.memory.set8(500, 0b1010_1010).unwrap();

    test.set_register(1, 500);
    test.set_register(2, 0);
    test.dbg_step(&encoder::lw(1, 2, 0));
    assert_eq!(test.get_register(1), 500);
    assert_eq!(
        test.get_register(2),
        0b1111_1111_0000_0000_0101_1100_1010_1010u32 as i32
    );
}

#[test]
fn execute_lbu() {
    let mut test = init();
    test.memory.set8(499, 10).unwrap();
    test.memory.set8(500, 50).unwrap();
    test.memory.set8(501, 25).unwrap();
    test.memory.set8(502, 0xFF).unwrap();
    test.set_register(1, 500);
    test.set_register(2, 0);
    test.dbg_step(&encoder::lbu(1, 2, 0));
    assert_eq!(test.get_register(1), 500);
    assert_eq!(test.get_register(2), 50);

    test.dbg_step(&encoder::lbu(1, 2, 1));
    assert_eq!(test.get_register(1), 500);
    assert_eq!(test.get_register(2), 25);

    test.dbg_step(&encoder::lbu(1, 2, -1));
    assert_eq!(test.get_register(1), 500);
    assert_eq!(test.get_register(2), 10);

    // Test sign extension
    test.dbg_step(&encoder::lbu(1, 2, 2));
    assert_eq!(test.get_register(1), 500);
    assert_eq!(test.get_register(2), 0xFF);
}

#[test]
fn execute_lhu() {
    let mut test = init();

    test.memory.set8(501, 0b0101_1100).unwrap();
    test.memory.set8(500, 0b1010_1010).unwrap();
    test.set_register(1, 500);
    test.set_register(2, 0);
    test.dbg_step(&encoder::lhu(1, 2, 0));
    assert_eq!(test.get_register(1), 500);
    assert_eq!(test.get_register(2), 0b0101_1100_1010_1010);

    // Test immediate offset
    test.memory.set8(503, 0b0111_1100).unwrap();
    test.memory.set8(502, 0b1010_1110).unwrap();
    test.set_register(1, 500);
    test.set_register(2, 0);
    test.dbg_step(&encoder::lhu(1, 2, 2));
    assert_eq!(test.get_register(1), 500);
    assert_eq!(test.get_register(2), 0b0111_1100_1010_1110);

    // Test that the LH is sign extended by default
    test.memory.set8(501, 0b1101_1100).unwrap();
    test.memory.set8(500, 0b1010_1010).unwrap();
    test.set_register(1, 500);
    test.set_register(2, 0);
    test.dbg_step(&encoder::lhu(1, 2, 0));
    assert_eq!(test.get_register(1), 500);
    assert_eq!(test.get_register(2), 0b1101_1100_1010_1010u32 as i32);
}

#[test]
fn execute_sb() {
    let mut test = init();
    test.memory.set8(500, 50).unwrap();
    test.set_register(1, 500);
    test.set_register(2, 0xDEADBEFFu32 as i32);
    test.dbg_step(&encoder::sb(1, 2, 0));
    assert_eq!(test.get_register(1), 500);
    assert_eq!(test.memory.get8(500), Ok(0xFF));
    assert_eq!(test.memory.get8(501), Ok(0x0));
    test.dbg_step(&encoder::sb(1, 2, 1));
    assert_eq!(test.memory.get8(501), Ok(0xFF));
}

#[test]
fn execute_sh() {
    let mut test = init();
    test.memory.set8(500, 50).unwrap();
    test.set_register(1, 500);
    test.set_register(2, 0xDEADBEFFu32 as i32);
    test.dbg_step(&encoder::sh(1, 2, 0));
    assert_eq!(test.get_register(1), 500);
    assert_eq!(test.memory.get16(500), Ok(0xBEFF));
    assert_eq!(test.memory.get16(502), Ok(0x0));
    test.dbg_step(&encoder::sh(1, 2, 2));
    assert_eq!(test.memory.get16(502), Ok(0xBEFF));
}

#[test]
fn execute_sw() {
    let mut test = init();
    test.memory.set8(500, 50).unwrap();
    test.set_register(1, 500);
    test.set_register(2, 0xDEADBEFFu32 as i32);
    test.dbg_step(&encoder::sw(1, 2, 0));
    assert_eq!(test.get_register(1), 500);
    assert_eq!(test.memory.get32(500), Ok(0xDEADBEFF));
    assert_eq!(test.memory.get32(504), Ok(0x0));
    test.dbg_step(&encoder::sw(1, 2, 4));
    assert_eq!(test.memory.get32(504), Ok(0xDEADBEFF));
}

#[test]
fn execute_fence() {
    let mut test = init();
    test.dbg_step(&encoder::fence());
}

#[test]
fn execute_fence_i() {
    let mut test = init();
    test.dbg_step(&encoder::fence_i());
}

#[test]
fn ecall() {
    let mut test = init();
    let mut executed = false;
    test.step_with_ecall(&encoder::ecall(), |_| {
        executed = true;
    });
    assert_eq!(executed, true);
    assert_eq!(4, test.state.registers.pc);
}

#[test]
fn ebreak() {
    // TODO: I tested this and observed a breakpoint trap. I'm not sure how to test it
    // systematically. Maybe we could set a flag debug_breakpoint to true rather than actually
    // breaking in the core.
}

#[test]
fn csrrw() {
    let mut test = init();
    test.state.registers.csrs.test = 400;
    test.set_register(2, 500);
    test.dbg_step(&encoder::csrrw(2, 2, 0x1));
    assert_eq!(test.get_register(2), 400);
    assert_eq!(test.state.registers.csrs.test, 500);
    test.dbg_step(&encoder::csrrw(2, 2, 0x1));
    assert_eq!(test.get_register(2), 500);
    assert_eq!(test.state.registers.csrs.test, 400);
}

#[test]
#[should_panic]
fn csrrw_illegal() {
    let mut test = init();
    test.dbg_step(&encoder::csrrw(2, 2, 0x2));
}

#[test]
fn csrrs() {
    let mut test = init();
    test.state.registers.csrs.test = 400;
    test.dbg_step(&encoder::csrrs(0, 1, 0x1));
    assert_eq!(test.get_register(1), 400);

    test.set_register(1, 0);
    test.set_register(2, 0b101);
    test.dbg_step(&encoder::csrrs(2, 1, 0x1));
    assert_eq!(test.get_register(1), 400);
    assert_eq!(test.state.registers.csrs.test, 405);

    // Test that read with source of zero from instret succeeds
    test.state.registers.csrs.instret = 0;
    test.dbg_step(&encoder::csrrs(0, 1, 0xC02));
    assert_eq!(test.get_register(1), 0);
}

#[test]
#[should_panic]
fn csrrs_illegal_write() {
    let mut test = init();
    test.dbg_step(&encoder::csrrs(1, 2, 0xC01));
}

#[test]
fn csrrc() {
    let mut test = init();
    test.state.registers.csrs.test = 405;
    test.dbg_step(&encoder::csrrc(0, 1, 0x1));
    assert_eq!(test.get_register(1), 405);

    test.set_register(1, 0);
    test.set_register(2, 0b101);
    test.dbg_step(&encoder::csrrc(2, 1, 0x1));
    assert_eq!(test.get_register(1), 405);
    assert_eq!(test.state.registers.csrs.test, 400);

    // Test that read with source of zero from instret succeeds
    test.state.registers.csrs.instret = 0;
    test.dbg_step(&encoder::csrrc(0, 1, 0xC02));
    assert_eq!(test.get_register(1), 0);
}

#[test]
#[should_panic]
fn csrrc_illegal_write() {
    let mut test = init();
    test.dbg_step(&encoder::csrrc(1, 2, 0xC01));
}

#[test]
fn csrrwi() {
    let mut test = init();
    test.state.registers.csrs.test = 400;
    test.set_register(2, 500);
    test.dbg_step(&encoder::csrrwi(2, 2, 0x1));
    assert_eq!(test.get_register(2), 400);
    assert_eq!(test.state.registers.csrs.test, 2);
}

#[test]
#[should_panic]
fn csrrwi_illegal() {
    let mut test = init();
    test.dbg_step(&encoder::csrrwi(2, 2, 0x2));
}

#[test]
fn csrrsi() {
    let mut test = init();
    test.state.registers.csrs.test = 400;
    test.dbg_step(&encoder::csrrsi(0, 1, 0x1));
    assert_eq!(test.get_register(1), 400);

    test.dbg_step(&encoder::csrrsi(1, 1, 0x1));
    assert_eq!(test.get_register(1), 400);
    assert_eq!(test.state.registers.csrs.test, 401);

    // Test that read with source of zero from instret succeeds
    test.state.registers.csrs.instret = 0;
    test.dbg_step(&encoder::csrrsi(0, 1, 0xC02));
    assert_eq!(test.get_register(1), 0);
}

#[test]
#[should_panic]
fn csrrsi_illegal_write() {
    let mut test = init();
    test.dbg_step(&encoder::csrrsi(1, 2, 0xC01));
}

#[test]
fn csrrci() {
    let mut test = init();
    test.state.registers.csrs.test = 405;
    test.dbg_step(&encoder::csrrci(0, 1, 0x1));
    assert_eq!(test.get_register(1), 405);

    test.set_register(1, 0);
    test.dbg_step(&encoder::csrrci(5, 1, 0x1));
    assert_eq!(test.get_register(1), 405);
    assert_eq!(test.state.registers.csrs.test, 400);

    // Test that read with source of zero from instret succeeds
    test.state.registers.csrs.instret = 0;
    test.dbg_step(&encoder::csrrci(0, 1, 0xC02));
    assert_eq!(test.get_register(1), 0);
}

#[test]
#[should_panic]
fn csrrci_illegal_write() {
    let mut test = init();
    test.dbg_step(&encoder::csrrci(1, 2, 0xC01));
}
