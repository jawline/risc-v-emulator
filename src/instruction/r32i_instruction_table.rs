use super::{decoder, encoder, funct3};
use crate::cpu_r32i::Cpu;
use crate::instruction::opcodes;
use crate::memory::Memory;

const INSTRUCTION_SIZE: u32 = 4;

type CpuState = crate::cpu_r32i::CpuState<u32, 32>;

type OpcodeHandler = fn(&mut CpuState, &mut Memory, /* instruction */ u32);

fn panic_dump_state(reason: &str, instruction: u32, c: &mut CpuState) {
    panic!("{reason} {instruction:032b} {c:?}")
}

fn trap_opcode(c: &mut CpuState, memory: &mut Memory, instruction: u32) {
    panic_dump_state(
        "Trap handler called. The emulated CPU encountered an illegal opcode",
        instruction,
        c,
    );
}

fn op_imm(c: &mut CpuState, memory: &mut Memory, instruction: u32) {
    match decoder::funct3(instruction) {
        funct3::ADDI => {
            let source_register = decoder::rs1(instruction);
            let destination_register = decoder::rd(instruction);

            let immediate = decoder::i_type_immediate_32(instruction);
            let source_value = c.registers.get(source_register) as i32;
            let new_value = source_value + immediate;

            c.registers.set(destination_register, new_value as u32);
        }
        funct3::SLTI => {
            let source_register = decoder::rs1(instruction);
            let destination_register = decoder::rd(instruction);

            let immediate = decoder::i_type_immediate_32(instruction);
            let source_value = c.registers.get(source_register) as i32;

            let new_value = if source_value < immediate { 1 } else { 0 };
            c.registers.set(destination_register, new_value as u32);
        }
        funct3::SLTIU => {

            // This is the same as SLTI but the immediate is sign extended and then treated as an
            // unsigned and the comparison is done as an unsigned.
        
            let source_register = decoder::rs1(instruction);
            let destination_register = decoder::rd(instruction);

            let immediate = decoder::i_type_immediate_32(instruction) as u32;
            let source_value = c.registers.get(source_register);

            let new_value = if source_value < immediate { 1 } else { 0 };
            c.registers.set(destination_register, new_value as u32);
        }
        _ => panic!("funct3 parameter should not be > 0b111"),
    };

    c.registers.pc += INSTRUCTION_SIZE;
}

pub struct InstructionTable {
    // Total map from opcode to handler
    handlers: [OpcodeHandler; 32],
}

impl InstructionTable {
    pub const fn new() -> Self {
        let mut handlers: [OpcodeHandler; 32] = [trap_opcode; 32];
        handlers[opcodes::OP_IMM] = op_imm;
        Self { handlers }
    }

    pub fn step(&self, cpu_state: &mut CpuState, memory: &mut Memory, instruction: u32) {
        (self.handlers[decoder::opcode(instruction)])(cpu_state, memory, instruction)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    const fn pack_negative_into_12b(val: i16) -> u16 {
        (val as u16) & 0b0000_1111_1111_1111
    }

    #[test]
    fn create_table() {
        let table = InstructionTable::new();
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

        panic!("Test max / min int");
    }

    #[test]
    fn execute_sltiu() {
        let (mut cpu, mut memory, table) = test_args();
        panic!("unimplemented");
    }
}
