use super::util::construct_opcode;

pub const OP: usize = construct_opcode(0b100, 0b01);
pub const OP_IMM: usize = construct_opcode(0b100, 0b00);
pub const JAL: usize = construct_opcode(0b011, 0b11);
pub const JALR: usize = construct_opcode(0b001, 0b11);
pub const LUI: usize = construct_opcode(0b101, 0b01);
pub const AUIPC: usize = construct_opcode(0b101, 0b00);
pub const BRANCH: usize = construct_opcode(0b000, 0b11);
pub const LOAD: usize = construct_opcode(0b000, 0b00);
pub const STORE: usize = construct_opcode(0b000, 0b01);
pub const FENCE: usize = construct_opcode(0b011, 0b00);
pub const SYSTEM: usize = construct_opcode(0b100, 0b11);
