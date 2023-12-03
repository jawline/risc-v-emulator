pub mod op_imm {
    pub const ADDI: u8 = 0b000;
    pub const SLLI: u8 = 0b001;
    pub const SLTI: u8 = 0b010;
    pub const XORI: u8 = 0b100;
    pub const SLTIU: u8 = 0b011;
    pub const ORI: u8 = 0b110;
    pub const ANDI: u8 = 0b111;

    /// Depending on the upper 7 bits of the imm this is either SRAI or SRLI
    pub const SRLI_OR_SRAI: u8 = 0b101;
}

pub mod op {

    // These seem identical to the op_imm versions but I'll leave it for clarity in use.
    pub const ADD_OR_SUB: u8 = 0b000;
    pub const SLL: u8 = 0b001;
    pub const SLT: u8 = 0b010;
    pub const XOR: u8 = 0b100;
    pub const SLTU: u8 = 0b011;
    pub const OR: u8 = 0b110;
    pub const AND: u8 = 0b111;

    /// Depending on the upper 7 bits of the imm this is either SRAI or SRLI
    pub const SRL_OR_SRA: u8 = 0b101;
}

pub mod branch {
    pub const BEQ: u8 = 0b000;
    pub const BNE: u8 = 0b001;
    pub const BLT: u8 = 0b100;
    pub const BGE: u8 = 0b101;
    pub const BLTU: u8 = 0b110;
    pub const BGEU: u8 = 0b111;
}
