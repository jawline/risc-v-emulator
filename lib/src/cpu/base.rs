use crate::cpu::registers::Registers;

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
