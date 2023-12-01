use crate::memory::Memory;
use crate::registers::Registers;

pub struct CpuState<T: Default + Copy, const N: usize> {
    pub registers: Registers<T, N>,
    // TODO: Shift this out for real traps
    pub exception: bool,
}

pub struct Cpu(CpuState<u32, 32>);

impl Cpu {
    pub fn new() -> Self {
        Self(CpuState {
            registers: Registers::<u32, 32>::new(),
            exception: false,
        })
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
