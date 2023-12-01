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
const LOWEST_3_BITS: u32 = 0b111;

mod decoder {
    use super::{extract, LOWEST_3_BITS, LOWEST_5_BITS, LOWEST_7_BITS};

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

    /// Extract the 'funct3' (function 3 bits) part of the instruction (3 bits from 12 to 14
    /// inclusive)
    pub const fn funct3(instruction: u32) -> u8 {
        extract(instruction, 12, LOWEST_3_BITS) as u8
    }

    /// Extract the 'funct7' (function 7 bits) part of the instruction (7 bits from 25 to 31
    /// inclusive)
    pub const fn funct7(instruction: u32) -> u8 {
        extract(instruction, 25, LOWEST_7_BITS) as u8
    }

    /// Extract a sign extended u type immediate (u-type immediates are
    /// packed such that instruction[31:12] = immediate[31:12] with the
    /// lowest bits being zero).
    ///
    /// All immediates are sign extended but in this case the sign bit is
    /// naturally in the correct position so we just cast it to an i32.
    pub const fn u_type_immediate(instruction: u32) -> i32 {
        extract(instruction, 0, 0b1111_1111_1111_1111_1110_0000_0000_0000) as i32
    }

    /// Sign extend an input to 32 bits given a sign bit and the unsigned portion.
    const fn sign_extend_32(raw_value: u32, bits: usize, sign_bit: bool) -> i32 {
        // This mask will be all 1s for the bits in raw_values range.
        let bit_mask = (1 << bits) - 1;

        // This mask will be all 1's for the bits between bits and 32 and all 0's for bit x < bits.
        let sign_mask = !bit_mask;

        // The sign extension should be all 1s if the sign bit is 1 or all 0s if the sign bit is
        // zero, so we either or the raw value with the sign mask if the sign bit is set or do
        // nothing.
        let extension = if sign_bit { sign_mask } else { 0 };
        ((raw_value & bit_mask) | extension) as i32
    }

    /// The i-type immediates are sign extended 12 bit immediates packed into the last 12 bits of
    /// the 32-bit instruction.
    pub const fn i_type_immediate_32(instruction: u32) -> i32 {
        // The sign bit is already at bit 31 so we just extract it with a mask. We need to shift
        // the other 12 bits.
        let sign_bit = extract(instruction, 0, 0b1000_0000_0000_0000_0000_0000_0000_0000) != 0;
        let raw_value = extract(instruction, 20, 0b0111_1111_1111);

        // Sign extend the result (fill the extended area with 1s if negative and 0s if positive).
        sign_extend_32(raw_value, 11, sign_bit)
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
        fn test_funct3() {
            assert_eq!(funct3(0b1111_0000_1110_0000_1000_0000_1111_0000), 0b000);
            assert_eq!(funct3(0b1111_0000_1110_0000_1010_0000_1111_0000), 0b010);
            assert_eq!(funct3(0b1111_0000_1110_0000_1111_0000_1111_0000), 0b111);
        }

        #[test]
        fn test_funct7() {
            assert_eq!(
                funct7(0b1111_0000_1111_0000_1111_0000_1111_0000),
                0b1111_000
            );
            assert_eq!(
                funct7(0b1010_1010_1111_0000_1111_0000_1111_0000),
                0b1010_101
            );
            assert_eq!(
                funct7(0b0000_0000_1111_0000_1111_0000_1111_0000),
                0b0000_000
            );
            assert_eq!(
                funct7(0b1111_1110_1111_0000_1111_0000_1111_0000),
                0b1111_111
            );
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

        #[test]
        fn test_sign_extend() {
            assert_eq!(sign_extend_32(1, 1, false), 1);
            assert_eq!(sign_extend_32(1, 1, true), -1);
            assert_eq!(sign_extend_32(127, 7, false), 127);
            assert_eq!(sign_extend_32(0b011_1111, 7, true), -65);
        }

        #[test]
        fn test_u_type_immediate() {
            // Test zero
            assert_eq!(
                u_type_immediate(0b0000_0000_0000_0000_0001_0101_0101_0110),
                0
            );

            // Test highest positive value (2^31 - 1) - (0b1_1111_1111_1111 the part of the range
            // we can't hold)
            assert_eq!(
                u_type_immediate(0b0111_1111_1111_1111_1110_0101_0101_0110),
                2147483647 - 8191
            );

            // Test the sign bit. Since the lower bits are always zero the least negative number
            // we can represent is -8912 (-2^31 from bit 31 being set + bits 30-12 set).
            assert_eq!(
                u_type_immediate(0b1111_1111_1111_1111_1110_0010_0111_0000),
                -(0b1_1111_1111_1111 as i32) - 1
            );
        }

        #[test]
        fn test_i_type_immediate() {

            const ZERO: u32 = 0b0000_0000_0000_1111_0001_0101_0101_0110;

            const fn pack(v: u32) -> u32 {
                ZERO | (v << 20)
            }

            const ONE: u32 = pack(1);
            const TWO_HUNDRED_AND_FIFTY: u32 = pack(250);
            const MAX_POSITIVE_VALUE: u32 = pack(2047);

            // Positive values
            assert_eq!(i_type_immediate_32(ZERO), 0);
            assert_eq!(i_type_immediate_32(ONE), 1);
            assert_eq!(i_type_immediate_32(TWO_HUNDRED_AND_FIFTY), 250);
            assert_eq!(i_type_immediate_32(MAX_POSITIVE_VALUE), 2047);

            // Negative values

            // Sign-aware truncation is the same as truncation for negative
            // values within the smaller range, so this should work for values
            // < 12 bits.
            const fn pack_negative(v: i32) -> u32 {
                pack(v as u32)
            }

            const MINUS_ONE: u32 = pack_negative(-1);
            const MINUS_TWO_HUNDRED_AND_FIFTY: u32 = pack_negative(-250);
            const MIN_NEGATIVE_VALUE: u32 = pack_negative(-2048);

            assert_eq!(i_type_immediate_32(MINUS_TWO_HUNDRED_AND_FIFTY), -250);
            assert_eq!(i_type_immediate_32(MINUS_ONE), -1);
            assert_eq!(i_type_immediate_32(MIN_NEGATIVE_VALUE), -2048);
        }
    }
}
