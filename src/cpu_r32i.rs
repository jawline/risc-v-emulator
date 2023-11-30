use crate::memory::Memory;
use crate::registers::Registers;

pub struct Cpu {
    pub registers: Registers<u32, 32>,
    // TODO: Shift this out for real traps
    pub exception: bool,
}

impl Cpu {
    pub fn new() -> Self {
        Self {
            registers: Registers::<u32, 32>::new(),
            exception: false,
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
