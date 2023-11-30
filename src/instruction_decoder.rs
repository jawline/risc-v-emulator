// Construct an opcode from a 3 bit column and a 2 bit row
pub const fn construct_opcode(col: u8, row: u8) -> u8 {
    let col_shifted = (col & 0b111) << 2;
    let row_shifted = (row & 0b11) << 5;
    col_shifted | row_shifted | 0b11
}

mod opcodes {
    use super::construct_opcode;
    pub const OP: u8 = construct_opcode(0b100, 0b01);
    pub const JALR: u8 = construct_opcode(0b001, 0b11);
}

/// Extract the opcode (lowest 7 bits of the 32 bits)
pub const fn opcode(instruction: u32) -> u8 {
    (instruction & 0b_0111_1111) as u8
}

/// Extract the 'rd' part of the instruction (4 bits from bits 7 to 11)
pub const fn rd(instruction: u32) -> u8 {
    ((instruction >> 7) & 0b1111) as u8
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_opcode() {
        assert_eq!(
            opcode(opcodes::OP as u32 | 0b1111_0000_1111_0000_1111_0000_0000_0000),
            opcodes::OP
        );

        assert_ne!(
            opcode(opcodes::OP as u32 | 0b1111_0000_1111_0000_1111_0000_0000_0000),
            opcodes::JALR
        );

        assert_eq!(
            opcode(opcodes::JALR as u32 | 0b1111_0000_1111_0000_1111_0000_0000_0000),
            opcodes::JALR
        );

        assert_ne!(
            opcode(opcodes::JALR as u32 | 0b1111_0000_1111_0000_1111_0000_0000_0000),
            opcodes::OP
        );
    }
}
