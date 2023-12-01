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

pub const fn extract(value: u32, shift: usize, mask: u32) -> u32 {
    (value >> shift) & mask
}

const LOWEST_7_BITS: u32 = 0b111_1111;
const LOWEST_5_BITS: u32 = 0b1_1111;

mod decoder {
    use super::{extract, LOWEST_5_BITS, LOWEST_7_BITS};

    /// Extract the opcode (lowest 7 bits of the 32 bits)
    pub const fn opcode(instruction: u32) -> u8 {
        extract(instruction, 0, LOWEST_7_BITS) as u8
    }

    /// Extract the 'rd' part of the instruction (5 bits from bits 7 to 11 inclusive)
    pub const fn rd(instruction: u32) -> u8 {
        extract(instruction, 7, LOWEST_5_BITS) as u8
    }

    /// Extract the 'rs1' (register source 1) part of the instruction (5 bits from 7 to 19 inclusive)
    pub const fn rs1(instruction: u32) -> u8 {
        extract(instruction, 15, LOWEST_5_BITS) as u8
    }

    /// Extract the 'rs2' (register source 2) part of the instruction (5 bits from 20 to 24 inclusive)
    pub const fn rs2(instruction: u32) -> u8 {
        extract(instruction, 20, LOWEST_5_BITS) as u8
    }

    #[cfg(test)]
    mod test {
        use super::super::*;
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

        #[test]
        fn test_rd() {
            assert_eq!(rd(0b1111_0000_1111_0000_1111_0000_0111_0000), 0b0000_0);
            assert_eq!(rd(0b1111_0000_1111_0000_1111_1010_0111_0000), 0b1010_0);
            assert_eq!(rd(0b1111_0000_1111_0000_1111_1010_1111_0000), 0b1010_1);
        }

        #[test]
        fn test_rs1() {
            assert_eq!(rs1(0b1111_0000_1111_0000_0111_0000_1111_0000), 0b0000_0);
            assert_eq!(rs1(0b1111_0000_1111_1010_1111_0000_1111_0000), 0b1010_1);
            assert_eq!(rs1(0b1111_0000_1111_1110_1111_0000_1111_0000), 0b1110_1);
        }

        #[test]
        fn test_rs2() {
            assert_eq!(rs2(0b1111_0000_0000_0000_1111_0000_1111_0000), 0b0000_0);
            assert_eq!(rs2(0b1111_0001_0101_0000_1111_0000_1111_0000), 0b1010_1);
            assert_eq!(rs2(0b1111_0001_1111_0000_1111_0000_1111_0000), 0b1111_1);
            assert_eq!(rs2(0b1111_0001_1011_0000_1111_0000_1111_0000), 0b1101_1);
        }
    }
}
