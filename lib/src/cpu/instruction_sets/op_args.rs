use crate::cpu::base::CpuState;
use crate::instruction::decoder;
use crate::memory::Memory;
use std::default::Default;
use std::marker::Copy;

pub struct OpArgs<'a, 'b, T: Default + Copy, const N: usize> {
    pub state: &'a mut CpuState<T, N>,
    pub memory: &'b mut Memory,
    pub instruction: u32,
}

impl<'a, 'b, T: Default + Copy, const N: usize> OpArgs<'a, 'b, T, N> {
    pub fn i_imm(&self) -> i32 {
        decoder::i_type_immediate_32(self.instruction)
    }

    pub fn b_imm(&self) -> i32 {
        decoder::b_type_immediate_32(self.instruction)
    }

    pub fn s_imm(&self) -> i32 {
        decoder::s_type_immediate_32(self.instruction)
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
