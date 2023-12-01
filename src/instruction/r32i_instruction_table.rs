use crate::cpu_r32i::CpuState;
use crate::memory::Memory;

type R32iCpuState = CpuState<u32, 32>;

type OpcodeHandler = fn(&mut CpuState<u32, 32>, &mut Memory);

pub struct Instruction_table {
    // Total map from opcode to handler
    handlers: [OpcodeHandler; 32],
}

fn exception(c: &mut R32iCpuState, memory: &mut Memory) {
    unimplemented!()
}

impl Instruction_table {
    pub const fn r32i() -> Self {
        Self {
            handlers: [exception; 32],
        }
    }
}
