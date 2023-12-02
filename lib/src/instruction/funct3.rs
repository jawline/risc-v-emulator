pub mod op_imm {
    pub const ADDI: u8 = 0b000;
    pub const SLLI: u8 = 0b001;
    pub const SLTI: u8 = 0b010;
    pub const XORI: u8 = 0b100;
    pub const SLTIU: u8 = 0b011;
    pub const ORI: u8 = 0b110;
    pub const ANDI: u8 = 0b111;

    /// Depending on the upper 7 bits of the imm this is either SRAI or SRLI
    pub const SRLI: u8 = 0b101;
}
