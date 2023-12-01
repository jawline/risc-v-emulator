use crate::memory::Memory;
use crate::registers::Registers;

pub struct CpuState<T: Default + Copy, const N: usize> {
    pub registers: Registers<T, N>,
    // TODO: Shift this out for real traps
    pub exception: bool,
}

impl<T: Default + Copy, const N: usize> CpuState<T, N> {
    pub fn new() -> Self {
        Self {
            registers: Registers::<T, N>::new(),
            exception: false,
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
