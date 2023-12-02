use super::{
    decoder,
    funct3::{op, op_imm},
};
use crate::instruction::{op_args::OpArgs, opcodes};
use crate::memory::Memory;

const INSTRUCTION_SIZE: u32 = 4;
const FUNCT7_SWITCH: u32 = 0b0100000;

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

fn apply_op_funct7<F: Fn(i32, i32, u8) -> i32>(op: &mut OpArgs, f: F) {
    let funct7 = op.funct7();
    apply_op(op, |r1, r2| f(r1, r2, funct7))
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

fn apply_op_imm_funct7<F: Fn(u32, u32, u32) -> u32>(op: &mut OpArgs, f: F) {
    apply_op_imm(op, |r, i| {
        let i = i as u32;
        let mode = i >> 5;
        let bits = i & 0b1_1111;
        f(r as u32, bits, mode) as i32
    })
}

/// A series of instructions that operate on a source register and an I-type (12-bit) immediate,
/// placing the result in rd.
fn op_imm(op: &mut OpArgs) {
    match op.funct3() {
        op_imm::ADDI => apply_op_imm(op, |r, i| r + i),
        op_imm::SLTI => apply_op_imm(op, |r, i| if r < i { 1 } else { 0 }),
        op_imm::SLTIU =>
        // This is the same as SLTI but the immediate is sign extended and then treated as an
        // unsigned and the comparison is done as an unsigned.
        {
            apply_op_imm(op, |r, i| {
                let (r, i) = (r as u32, i as u32);
                if r < i {
                    1
                } else {
                    0
                }
            })
        }
        op_imm::ANDI => apply_op_imm(op, |r, i| r & i),
        op_imm::ORI => apply_op_imm(op, |r, i| r | i),
        op_imm::XORI => apply_op_imm(op, |r, i| r ^ i),
        op_imm::SLLI => apply_op_imm_funct7(op, |r, i, mode| {
            if mode != 0 {
                panic!("SLL mode not zero");
            }
            r << i
        }),
        op_imm::SRLI_OR_SRAI => apply_op_imm_funct7(op, |r, i, mode| {
            // If FUNCT7_SWITCH is set then this is an SRAI rather than an SRLI
            if mode == FUNCT7_SWITCH {
                // Rust will do an arithmetic right shift if the integer is signed
                ((r as i32) >> i) as u32
            } else if mode == 0b0000000 {
                r >> i
            } else {
                panic!("SRL mode must be 0b0100000 or 0b0000000");
            }
        }),
        8..=u8::MAX => panic!("funct3 parameter should not be > 0b111. This is an emulation bug."),
    };

    op.state.registers.pc += INSTRUCTION_SIZE;
}

/// A series of instructions that operate on two source registers, placing the result in rd.
fn op(op: &mut OpArgs) {
    match op.funct3() {
        op::ADD_OR_SUB => apply_op_funct7(op, |r1, r2, funct7| {
            if funct7 as u32 == FUNCT7_SWITCH {
                r1 - r2
            } else {
                r1 + r2
            }
        }),
        op::SLT => apply_op(op, |r1, r2| if r1 < r2 { 1 } else { 0 }),
        op::SLTU =>
        // This is the same as SLTI but the immediate is sign extended and then treated as an
        // unsigned and the comparison is done as an unsigned.
        {
            apply_op(op, |r1, r2| {
                let (r1, r2) = (r1 as u32, r2 as u32);
                if r1 < r2 {
                    1
                } else {
                    0
                }
            })
        }
        op::AND => apply_op(op, |r1, r2| r1 & r2),
        op::OR => apply_op(op, |r, i| r | i),
        op::XOR => apply_op(op, |r, i| r ^ i),
        op::SLL => apply_op(op, |r1, r2| ((r1 as u32) << (r2 as u32)) as i32),
        op::SRL_OR_SRA => apply_op_funct7(op, |r1, r2, funct7| {
            if funct7 as u32 == FUNCT7_SWITCH {
                ((r1 as u32) >> (r2 as u32)) as i32
            } else {
                r1 >> (r2 as u32)
            }
        }),
        8..=u8::MAX => panic!("funct3 parameter should not be > 0b111. This is an emulation bug."),
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

pub struct InstructionTable {}

impl InstructionTable {
    pub const fn new() -> Self {
        Self {}
    }

    pub fn step(&self, cpu_state: &mut CpuState, memory: &mut Memory, instruction: u32) {
        let mut op_arg = OpArgs {
            state: cpu_state,
            memory: memory,
            instruction: instruction,
        };

        match decoder::opcode(instruction) {
            opcodes::OP => op(&mut op_arg),
            opcodes::OP_IMM => op_imm(&mut op_arg),
            opcodes::LUI => lui(&mut op_arg),
            opcodes::AUIPC => auipc(&mut op_arg),
            _ => trap_opcode(&mut op_arg),
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
}
