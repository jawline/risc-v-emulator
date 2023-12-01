use super::util::construct_opcode;

pub const OP: u8 = construct_opcode(0b100, 0b01);
pub const JALR: u8 = construct_opcode(0b001, 0b11);
