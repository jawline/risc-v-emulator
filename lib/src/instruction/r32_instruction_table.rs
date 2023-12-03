use super::{
    decoder,
    funct3::{branch, op, op_imm},
    util::C_5_BITS,
};
use crate::instruction::{op_args::OpArgs, opcodes};
use crate::memory::Memory;

const INSTRUCTION_SIZE: u32 = 4;
const FUNCT7_SWITCH: u8 = 0b0100000;

type CpuState = crate::cpu_r32i::CpuState<u32, 32>;

fn panic_dump_state(reason: &str, instruction: u32, c: &mut CpuState) {
    panic!("{reason} {instruction:032b} {c:?}")
}

fn trap_opcode(op: &mut OpArgs) {
    panic_dump_state(
        "Trap handler called. The emulated CPU encountered an illegal opcode",
        op.instruction,
        op.state,
    );
}

fn apply_op<F: Fn(i32, i32) -> i32>(op: &mut OpArgs, f: F) {
    let source_register1 = op.rs1();
    let source_register2 = op.rs2();
    let destination_register = op.rd();
    let source_value1 = op.state.registers.geti(source_register1);
    let source_value2 = op.state.registers.geti(source_register2);
    let new_value = f(source_value1, source_value2);
    op.state
        .registers
        .set(destination_register, new_value as u32);
}

fn apply_op_unsigned<F: Fn(u32, u32) -> u32>(op: &mut OpArgs, f: F) {
    apply_op(op, |r1, r2| f(r1 as u32, r2 as u32) as i32)
}

fn apply_op_with_funct7_switch<F1: Fn(i32, i32) -> i32, F2: Fn(i32, i32) -> i32>(
    op: &mut OpArgs,
    f_switch: F1,
    f_no_switch: F2,
) {
    let funct7 = op.funct7();
    apply_op(op, |r1, r2| {
        if funct7 == FUNCT7_SWITCH {
            f_switch(r1, r2)
        } else if funct7 == 0 {
            f_no_switch(r1, r2)
        } else {
            panic!("funct7 must be zero or FUNCT7_SWITCH")
        }
    })
}

fn apply_op_imm<F: Fn(i32, i32) -> i32>(op: &mut OpArgs, f: F) {
    let source_register = op.rs1();
    let destination_register = op.rd();
    let immediate = op.i_imm();
    let source_value = op.state.registers.get(source_register) as i32;
    let new_value = f(source_value, immediate);
    op.state
        .registers
        .set(destination_register, new_value as u32);
}

fn apply_op_imm_unsigned<F: Fn(u32, u32) -> u32>(op: &mut OpArgs, f: F) {
    apply_op_imm(op, |r, i| f(r as u32, i as u32) as i32)
}

fn apply_op_imm_unsigned_truncated_to_5_bits<F: Fn(u32, u32) -> u32>(op: &mut OpArgs, f: F) {
    apply_op_imm_unsigned(op, |r, i| f(r, i & C_5_BITS))
}

fn apply_op_imm_funct7<F1: Fn(u32, u32) -> u32, F2: Fn(u32, u32) -> u32>(
    op: &mut OpArgs,
    f_switch: F1,
    f_no_switch: F2,
) {
    let funct7 = op.funct7();
    apply_op_imm_unsigned_truncated_to_5_bits(op, |r, i| {
        if funct7 == FUNCT7_SWITCH {
            f_switch(r, i)
        } else if funct7 == 0 {
            f_no_switch(r, i)
        } else {
            panic!("funct7 must be zero or FUNCT7_SWITCH")
        }
    })
}

/// Apply the branch instruction. All branch instructions take 2 registers and either advance
/// the PC normally or jump to PC + a b-type coded immediate depending on the result.
fn apply_branch<F: Fn(i32, i32) -> bool>(op: &mut OpArgs, f: F) {
    let source_one = op.rs1();
    let source_two = op.rs2();
    let offset = op.b_imm();

    if f(
        op.state.registers.geti(source_one),
        op.state.registers.geti(source_two),
    ) {
        op.state.registers.pc = ((op.state.registers.pc as i32) + offset) as u32;
    } else {
        op.state.registers.pc += INSTRUCTION_SIZE;
    }
}

/// Identical to apply_branch but comparisons are done on the unsigned interpretation of the
/// registers
fn apply_branch_unsigned<F: Fn(u32, u32) -> bool>(op: &mut OpArgs, f: F) {
    apply_branch(op, |r1, r2| f(r1 as u32, r2 as u32))
}

/// A series of instructions that operate on a source register and an I-type (12-bit) immediate,
/// placing the result in rd.
fn op_imm(op: &mut OpArgs) {
    match op.funct3() {
        op_imm::ADDI => apply_op_imm(op, |r, i| r + i),
        op_imm::SLTI => apply_op_imm(op, |r, i| i32::from(r < i)),
        op_imm::SLTIU =>
        // This is the same as SLTI but the immediate is sign extended and then treated as an
        // unsigned and the comparison is done as an unsigned.
        {
            apply_op_imm_unsigned(op, |r, i| u32::from(r < i))
        }
        op_imm::ANDI => apply_op_imm(op, |r, i| r & i),
        op_imm::ORI => apply_op_imm(op, |r, i| r | i),
        op_imm::XORI => apply_op_imm(op, |r, i| r ^ i),
        op_imm::SLLI => {
            apply_op_imm_funct7(op, |_r, _i| panic!("SLL mode not zero"), |r, i| r << i)
        }
        op_imm::SRLI_OR_SRAI => {
            apply_op_imm_funct7(op, |r, i| ((r as i32) >> i) as u32, |r, i| r >> i)
        }
        8..=u8::MAX => panic!("funct3 parameter should not be > 0b111. This is an emulation bug."),
    };

    op.state.registers.pc += INSTRUCTION_SIZE;
}

/// A series of instructions that operate on two source registers, placing the result in rd.
fn op(op: &mut OpArgs) {
    match op.funct3() {
        op::ADD_OR_SUB => apply_op_with_funct7_switch(op, |r1, r2| r1 - r2, |r1, r2| r1 + r2),
        op::SLT => apply_op(op, |r1, r2| i32::from(r1 < r2)),
        op::SLTU =>
        // This is the same as SLTI but the immediate is sign extended and then treated as an
        // unsigned and the comparison is done as an unsigned.
        {
            apply_op_unsigned(op, |r1, r2| u32::from(r1 < r2))
        }
        op::AND => apply_op(op, |r1, r2| r1 & r2),
        op::OR => apply_op(op, |r, i| r | i),
        op::XOR => apply_op(op, |r, i| r ^ i),
        op::SLL => apply_op(op, |r1, r2| ((r1 as u32) << (r2 as u32)) as i32),
        op::SRL_OR_SRA => apply_op_with_funct7_switch(
            op,
            |r1, r2| ((r1 as u32) >> (r2 as u32)) as i32,
            |r1, r2| r1 >> (r2 as u32),
        ),
        8..=u8::MAX => panic!("funct3 parameter should not be > 0b111. This is an emulation bug."),
    };

    op.state.registers.pc += INSTRUCTION_SIZE;
}

/// A series of instructions that operate on two source registers, jumping to PC + a B-type immediate
/// offset if a condition is met, otherwise advancing the program counter normally.
fn branch(op: &mut OpArgs) {
    match op.funct3() {
        branch::BEQ => apply_branch(op, |r1, r2| r1 == r2),
        branch::BNE => apply_branch(op, |r1, r2| r1 != r2),
        branch::BLT => apply_branch(op, |r1, r2| r1 < r2),
        branch::BGE => apply_branch(op, |r1, r2| r1 >= r2),
        branch::BLTU => apply_branch_unsigned(op, |r1, r2| r1 < r2),
        branch::BGEU => apply_branch_unsigned(op, |r1, r2| r1 >= r2),
        2 | 3 | 8..=u8::MAX => {
            panic!("funct3 parameter should not be > 0b111. This is an emulation bug.")
        }
    };

    op.state.registers.pc += INSTRUCTION_SIZE;
}

/// Load upper immediate (Places a u-type immediate containing the upper 20 bits of a 32-bit value
/// into rd. All other bits are set to zero)
fn lui(op: &mut OpArgs) {
    let destination_register = op.rd();
    let immediate = op.u_imm();
    op.state
        .registers
        .set(destination_register, immediate as u32);
    op.state.registers.pc += INSTRUCTION_SIZE;
}

/// Add upper immediate to PC. Similar to LUI but adds the loaded immediate to current the program counter
/// and places it in RD. This can be used to compute addresses for JALR instructions.
fn auipc(op: &mut OpArgs) {
    let destination_register = op.rd();
    let immediate = op.u_imm();
    op.state.registers.set(
        destination_register,
        op.state.registers.pc + (immediate as u32),
    );
    op.state.registers.pc += INSTRUCTION_SIZE;
}

/// JAL (jump and link) adds the signed J-immediate value to the current PC after storing the
/// current PC + 4 in the destination register.
fn jal(op: &mut OpArgs) {
    let destination_register = op.rd();
    let imm_value = op.j_imm();
    let new_pc = ((op.state.registers.pc as i32) + imm_value) as u32;

    op.state
        .registers
        .set(destination_register, op.state.registers.pc + 4);
    op.state.registers.pc = new_pc;
}

/// JALR (Indirect jump) adds a 12-bit signed immediate to whatever is at rs1, sets the LSB of that
/// result to zero (e.g, result = result & (!1)), and finally sets the PC to this new result.
/// rd is set to the original PC + 4 (the start of the next instruction). Regiser 0 can be used to
/// discard the result.
fn jalr(op: &mut OpArgs) {
    let source_register = op.rs1();
    let destination_register = op.rd();
    let source_value = op.state.registers.geti(source_register);
    let imm_value = op.i_imm();
    let new_pc = (source_value + imm_value) as u32;
    let new_pc = new_pc & !1;
    op.state
        .registers
        .set(destination_register, op.state.registers.pc + 4);
    op.state.registers.pc = new_pc;
}

pub struct InstructionTable {}

impl InstructionTable {
    pub const fn new() -> Self {
        Self {}
    }

    pub fn step(&self, cpu_state: &mut CpuState, memory: &mut Memory, instruction: u32) {

        let op_arg = &mut OpArgs {
            state: cpu_state,
            memory: memory,
            instruction: instruction,
        };

        match decoder::opcode(instruction) {
            opcodes::OP => op(op_arg),
            opcodes::OP_IMM => op_imm(op_arg),
            opcodes::JAL => jal(op_arg),
            opcodes::JALR => jalr(op_arg),
            opcodes::LUI => lui(op_arg),
            opcodes::AUIPC => auipc(op_arg),
            opcodes::BRANCH => branch(op_arg),
            _ => trap_opcode(op_arg),
        }
    }
}

#[cfg(test)]
mod test {
    use super::super::encoder;
    use super::*;

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

    fn test_args() -> (CpuState, Memory, InstructionTable) {
        (CpuState::new(), Memory::new(4096), InstructionTable::new())
    }

    #[test]
    fn execute_no_op() {
        let (mut cpu, mut memory, table) = test_args();
        table.step(&mut cpu, &mut memory, encoder::no_op());
        assert_eq!(cpu.registers.pc, 4);
        assert_eq!(cpu.registers.get(0), 0);
    }

    #[test]
    fn execute_addi() {
        let (mut cpu, mut memory, table) = test_args();

        // Test INC 1
        table.step(&mut cpu, &mut memory, encoder::addi(1, 1, 1));
        assert_eq!(cpu.registers.pc, 4);
        assert_eq!(cpu.registers.get(1), 1);

        table.step(&mut cpu, &mut memory, encoder::addi(1, 1, 1));
        assert_eq!(cpu.registers.pc, 8);
        assert_eq!(cpu.registers.get(1), 2);

        table.step(&mut cpu, &mut memory, encoder::addi(1, 1, 1));
        assert_eq!(cpu.registers.pc, 12);
        assert_eq!(cpu.registers.get(1), 3);

        // Test r2 = r1 + 1
        table.step(&mut cpu, &mut memory, encoder::addi(2, 1, 1));
        assert_eq!(cpu.registers.pc, 16);
        assert_eq!(cpu.registers.get(1), 3);
        assert_eq!(cpu.registers.get(2), 4);

        // Test r2 = r1 - 1
        table.step(
            &mut cpu,
            &mut memory,
            encoder::addi(2, 1, pack_negative_into_12b(-1)),
        );
        assert_eq!(cpu.registers.pc, 20);
        assert_eq!(cpu.registers.get(1), 3);
        assert_eq!(cpu.registers.get(2), 2);

        // Test r3 = r2 + 4
        table.step(
            &mut cpu,
            &mut memory,
            encoder::addi(3, 2, pack_negative_into_12b(4)),
        );
        assert_eq!(cpu.registers.pc, 24);
        assert_eq!(cpu.registers.get(1), 3);
        assert_eq!(cpu.registers.get(2), 2);
        assert_eq!(cpu.registers.get(3), 6);
    }

    #[test]
    fn execute_addi_overflow() {
        unimplemented!();
    }

    #[test]
    fn execute_slti() {
        let (mut cpu, mut memory, table) = test_args();

        // Test positive
        cpu.registers.set(1, 5);
        table.step(&mut cpu, &mut memory, encoder::slti(2, 1, 4));
        assert_eq!(cpu.registers.get(1), 5);
        assert_eq!(cpu.registers.get(2), 0);
        assert_eq!(cpu.registers.pc, 4);

        table.step(&mut cpu, &mut memory, encoder::slti(2, 1, 5));
        assert_eq!(cpu.registers.get(1), 5);
        assert_eq!(cpu.registers.get(2), 0);
        assert_eq!(cpu.registers.pc, 8);

        table.step(&mut cpu, &mut memory, encoder::slti(2, 1, 6));
        assert_eq!(cpu.registers.get(1), 5);
        assert_eq!(cpu.registers.get(2), 1);
        assert_eq!(cpu.registers.pc, 12);

        // Test max int
        cpu.registers.set(1, 2047 as u32);
        table.step(&mut cpu, &mut memory, encoder::slti(2, 1, 2047));
        assert_eq!(cpu.registers.get(2), 0);
        assert_eq!(cpu.registers.pc, 16);

        cpu.registers.set(1, 2046 as u32);
        table.step(&mut cpu, &mut memory, encoder::slti(2, 1, 2047));
        assert_eq!(cpu.registers.get(2), 1);
        assert_eq!(cpu.registers.pc, 20);

        // Test negative
        cpu.registers.set(1, (-5000i32) as u32);
        table.step(
            &mut cpu,
            &mut memory,
            encoder::slti(2, 1, pack_negative_into_12b(-2048)),
        );
        assert_eq!(cpu.registers.get(2), 1);
        assert_eq!(cpu.registers.pc, 24);

        cpu.registers.set(1, (-2000i32) as u32);
        table.step(
            &mut cpu,
            &mut memory,
            encoder::slti(2, 1, pack_negative_into_12b(-2048)),
        );
        assert_eq!(cpu.registers.get(2), 0);
        assert_eq!(cpu.registers.pc, 28);
    }

    #[test]
    fn execute_sltiu() {
        let (mut cpu, mut memory, table) = test_args();

        cpu.registers.set(1, 0);
        table.step(&mut cpu, &mut memory, encoder::sltiu(2, 1, 0));
        assert_eq!(cpu.registers.get(2), 0);
        assert_eq!(cpu.registers.pc, 4);

        cpu.registers.set(1, 0);
        table.step(&mut cpu, &mut memory, encoder::sltiu(2, 1, 1));
        assert_eq!(cpu.registers.get(2), 1);
        assert_eq!(cpu.registers.pc, 8);

        cpu.registers.set(1, 2);
        table.step(&mut cpu, &mut memory, encoder::sltiu(2, 1, 1));
        assert_eq!(cpu.registers.get(2), 0);
        assert_eq!(cpu.registers.pc, 12);

        cpu.registers.set(1, 2048);
        table.step(&mut cpu, &mut memory, encoder::sltiu(2, 1, 2047));
        assert_eq!(cpu.registers.get(2), 0);
        assert_eq!(cpu.registers.pc, 16);

        cpu.registers.set(1, 2046);
        table.step(&mut cpu, &mut memory, encoder::sltiu(2, 1, 2047));
        assert_eq!(cpu.registers.get(2), 1);
        assert_eq!(cpu.registers.pc, 20);
    }

    #[test]
    fn execute_andi() {
        let (mut _cpu, mut _memory, _table) = test_args();
        unimplemented!();
    }

    #[test]
    fn execute_ori() {
        let (mut _cpu, mut _memory, _table) = test_args();
        unimplemented!();
    }

    #[test]
    fn execute_xori() {
        let (mut _cpu, mut _memory, _table) = test_args();
        unimplemented!();
    }

    #[test]
    fn execute_add() {
        let (mut _cpu, mut _memory, _table) = test_args();
        unimplemented!();
    }

    #[test]
    fn execute_sub() {
        let (mut _cpu, mut _memory, _table) = test_args();
        unimplemented!();
    }

    #[test]
    fn execute_slt() {
        let (mut _cpu, mut _memory, _table) = test_args();
        unimplemented!();
    }

    #[test]
    fn execute_sltu() {
        let (mut _cpu, mut _memory, _table) = test_args();
        unimplemented!();
    }

    #[test]
    fn execute_and() {
        let (mut _cpu, mut _memory, _table) = test_args();
        unimplemented!();
    }

    #[test]
    fn execute_or() {
        let (mut _cpu, mut _memory, _table) = test_args();
        unimplemented!();
    }

    #[test]
    fn execute_xor() {
        let (mut _cpu, mut _memory, _table) = test_args();
        unimplemented!();
    }

    #[test]
    fn execute_sll() {
        let (mut _cpu, mut _memory, _table) = test_args();
        unimplemented!();
    }

    #[test]
    fn execute_srl() {
        let (mut _cpu, mut _memory, _table) = test_args();
        unimplemented!();
    }

    #[test]
    fn execute_sra() {
        let (mut _cpu, mut _memory, _table) = test_args();
        unimplemented!();
    }

    #[test]
    fn execute_lui() {
        let (mut _cpu, mut _memory, _table) = test_args();
        unimplemented!();
    }

    #[test]
    fn execute_auipc() {
        let (mut _cpu, mut _memory, _table) = test_args();
        unimplemented!();
    }

    #[test]
    fn execute_jal() {
        let (mut _cpu, mut _memory, _table) = test_args();
        unimplemented!();
    }

    #[test]
    fn execute_jalr() {
        let (mut _cpu, mut _memory, _table) = test_args();
        unimplemented!();
    }

    #[test]
    fn execute_jalr_result_is_misaligned() {
        let (mut _cpu, mut _memory, _table) = test_args();
        unimplemented!();
    }

    #[test]
    fn execute_beq() {
        let (mut _cpu, mut _memory, _table) = test_args();
        unimplemented!();
    }

    #[test]
    fn execute_bne() {
        let (mut _cpu, mut _memory, _table) = test_args();
        unimplemented!();
    }

    #[test]
    fn execute_blt() {
        let (mut _cpu, mut _memory, _table) = test_args();
        unimplemented!();
    }

    #[test]
    fn execute_bge() {
        let (mut _cpu, mut _memory, _table) = test_args();
        unimplemented!();
    }

    #[test]
    fn execute_bltu() {
        let (mut _cpu, mut _memory, _table) = test_args();
        unimplemented!();
    }

    #[test]
    fn execute_bgeu() {
        let (mut _cpu, mut _memory, _table) = test_args();
        unimplemented!();
    }
}
