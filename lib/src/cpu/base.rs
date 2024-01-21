use crate::cpu::registers::Registers;
use crate::memory::Memory;
use crate::cpu::instruction_sets::rv32i::InstructionSet;

#[derive(Debug)]
pub struct CpuState<T: Default + Copy, const N: usize> {
    pub registers: Registers<T, N>,
}

impl<T: Default + Copy, const N: usize> CpuState<T, N> {
    pub fn new() -> Self {
        Self {
            registers: Registers::<T, N>::new(),
        }
    }
}

pub struct Cpu {
    pub state: CpuState<u32, 32>,
    tbl: InstructionSet,
}

impl Cpu {
    pub fn new() -> Self {
        Self {
            state: CpuState::new(),
            tbl: InstructionSet::new(),
        }
    }

    pub fn step(&mut self, memory: &mut Memory) {
        let next_instruction = memory.get32(self.state.registers.pc as usize).expect("PC out of range");
        self.tbl.step(&mut self.state, memory, next_instruction, |_| { });
    }
}

#[cfg(test)]
mod basic_tests {
    use super::*;
    use crate::instruction::encoder;

    #[test]
    fn test_create() {
        let _cpu = Cpu::new();
    }

    #[test]
    fn test_step() {
        let mut cpu = Cpu::new();
        let mut memory = Memory::new(8);
        memory.set32(0, encoder::no_op().encode()).unwrap();
        cpu.step(&mut memory);
        assert_eq!(cpu.state.registers.pc, 4);
    }
}
