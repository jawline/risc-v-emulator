use super::util::construct_opcode;

pub const OP: usize = construct_opcode(0b100, 0b01);
pub const OP_IMM: usize = construct_opcode(0b100, 0b00);
pub const JALR: usize = construct_opcode(0b001, 0b11);
