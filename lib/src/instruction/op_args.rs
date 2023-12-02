use crate::cpu_r32i::CpuState;
use crate::instruction::decoder;
use crate::memory::Memory;

pub struct OpArgs<'a, 'b> {
    pub state: &'a mut CpuState<u32, 32>,
    pub memory: &'b mut Memory,
    pub instruction: u32,
}

impl<'a, 'b> OpArgs<'a, 'b> {
    pub fn i_imm(&self) -> i32 {
        decoder::i_type_immediate_32(self.instruction)
    }

    pub fn u_imm(&self) -> i32 {
        decoder::u_type_immediate(self.instruction)
    }

    pub fn j_imm(&self) -> i32 {
        decoder::j_type_immediate_32(self.instruction)
    }

    pub fn rd(&self) -> usize {
        decoder::rd(self.instruction)
    }

    pub fn rs1(&self) -> usize {
        decoder::rs1(self.instruction)
    }

    pub fn rs2(&self) -> usize {
        decoder::rs2(self.instruction)
    }

    pub fn funct3(&self) -> u8 {
        decoder::funct3(self.instruction)
    }

    pub fn funct7(&self) -> u8 {
        decoder::funct7(self.instruction)
    }
}
