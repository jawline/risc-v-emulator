use crate::cpu::csrs::Csrs;
use std::default::Default;

#[derive(Debug)]
struct General<T: Default + Copy, const N: usize> {
    registers: [T; N],
}

impl<T: Default + Copy, const N: usize> General<T, N> {
    fn new() -> Self {
        Self {
            registers: [T::default(); N],
        }
    }

    fn get(&self, slot: usize) -> T {
        self.registers[slot]
    }

    fn set(&mut self, slot: usize, value: T) {
        self.registers[slot] = value;

        // We reset the register 0 to T::default() which will be zero for u32 or u64
        self.registers[0] = T::default();
    }
}

#[derive(Debug)]
pub struct Registers<T: Default + Copy, const N: usize> {
    general: General<T, N>,
    pub pc: T,
    pub csrs: Csrs,
}

// Convenience functions that proxy through to general. pc can be used directly.
impl<T: Default + Copy, const N: usize> Registers<T, N> {
    pub fn new() -> Self {
        Self {
            pc: T::default(),
            general: General::<T, N>::new(),
            csrs: Csrs::new(),
        }
    }

    pub fn get(&self, slot: usize) -> T {
        self.general.get(slot)
    }

    pub fn set(&mut self, slot: usize, value: T) {
        self.general.set(slot, value)
    }
}

impl Registers<u32, 32> {
    pub fn geti(&self, slot: usize) -> i32 {
        self.get(slot) as i32
    }

    pub fn seti(&mut self, slot: usize, val: i32) {
        self.set(slot, val as u32)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn get_and_set() {
        let mut registers = Registers::<u32, 64>::new();

        for i in 1..32 {
            assert_eq!(registers.get(i), 0);
            registers.set(i, 50);
            assert_eq!(registers.get(i), 50);
            registers.set(i, 100);
            assert_eq!(registers.get(i), 100);
        }
    }

    #[test]
    fn get_and_set_r0() {
        let mut registers = Registers::<u32, 64>::new();
        assert_eq!(registers.get(0), 0);
        registers.set(0, 50);
        assert_eq!(registers.get(0), 0);
        registers.set(0, 100);
        assert_eq!(registers.get(0), 0);
    }
}
