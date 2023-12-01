use super::{encoder, decoder, funct3};
use crate::cpu_r32i::Cpu;
use crate::instruction::opcodes;
use crate::memory::Memory;

type CpuState = crate::cpu_r32i::CpuState<u32, 32>;

type OpcodeHandler = fn(&mut CpuState, &mut Memory, /* instruction */ u32);

fn exception(c: &mut CpuState, memory: &mut Memory, _: u32) {
    unimplemented!()
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
            c.registers.pc += 4;

            println!("WARNING: TODO check overflow");
        
            panic!("exceptions are not yet correctly handled");
        }
        _ => panic!("funct3 parameter should not be > 0b111"),
    }
}

pub struct InstructionTable {
    // Total map from opcode to handler
    handlers: [OpcodeHandler; 32],
}

impl InstructionTable {
    pub const fn new() -> Self {
        let mut handlers: [OpcodeHandler; 32] = [exception; 32];
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
    }
}
