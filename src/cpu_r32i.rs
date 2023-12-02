use crate::memory::Memory;
use crate::registers::Registers;

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
}

impl Cpu {
    pub fn new() -> Self {
        Self {
            state: CpuState::new(),
        }
    }

    pub fn step(memory: &mut Memory) {
        unimplemented!();
    }
}

#[cfg(test)]
mod basic_tests {
    use super::*;

    #[test]
    fn test_create() {
        let _cpu = Cpu::new();
    }
}
