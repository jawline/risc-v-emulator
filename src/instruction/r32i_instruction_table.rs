use super::decoder;
use super::funct3;
use crate::cpu_r32i::CpuState;
use crate::instruction::opcodes;
use crate::memory::Memory;

type R32iCpuState = CpuState<u32, 32>;

type OpcodeHandler = fn(&mut CpuState<u32, 32>, &mut Memory, /* instruction */ u32);

fn exception(c: &mut R32iCpuState, memory: &mut Memory, _: u32) {
    unimplemented!()
}

fn op_imm(c: &mut R32iCpuState, memory: &mut Memory, instruction: u32) {
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
        }
        _ => panic!("funct3 parameter should not be > 0b111"),
    }
}

pub struct InstructionTable {
    // Total map from opcode to handler
    handlers: [OpcodeHandler; 32],
}

impl InstructionTable {
    pub const fn r32i() -> Self {
        let mut handlers = [exception; 32];
        handlers[opcodes::OP_IMM] = op_imm;
        Self { handlers }
    }
}
