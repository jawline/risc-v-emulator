use super::util::{
    extract, C_10_BITS, C_11_BITS, C_3_BITS, C_4_BITS, C_5_BITS, C_6_BITS, C_7_BITS, C_8_BITS,
    SIGN_BIT,
};

/// Extract the opcode (lowest 7 bits of the 32 bits)
pub const fn opcode(instruction: u32) -> u8 {
    extract(instruction, 0, C_7_BITS) as u8
}

/// Extract the 'rd' part of the instruction (5 bits from bits 7 to 11 inclusive)
pub const fn rd(instruction: u32) -> u8 {
    extract(instruction, 7, C_5_BITS) as u8
}

/// Extract the 'rs1' (register source 1) part of the instruction (5 bits from 7 to 19 inclusive)
pub const fn rs1(instruction: u32) -> u8 {
    extract(instruction, 15, C_5_BITS) as u8
}

/// Extract the 'rs2' (register source 2) part of the instruction (5 bits from 20 to 24 inclusive)
pub const fn rs2(instruction: u32) -> u8 {
    extract(instruction, 20, C_5_BITS) as u8
}

/// Extract the 'funct3' (function 3 bits) part of the instruction (3 bits from 12 to 14
/// inclusive)
pub const fn funct3(instruction: u32) -> u8 {
    extract(instruction, 12, C_3_BITS) as u8
}

/// Extract the 'funct7' (function 7 bits) part of the instruction (7 bits from 25 to 31
/// inclusive)
pub const fn funct7(instruction: u32) -> u8 {
    extract(instruction, 25, C_7_BITS) as u8
}

/// Extract a sign extended u type immediate (u-type immediates are
/// packed such that instruction[31:12] = immediate[31:12] with the
/// lowest bits being zero).
///
/// All immediates are sign extended but in this case the sign bit is
/// naturally in the correct position so we just cast it to an i32.
pub const fn u_type_immediate(instruction: u32) -> i32 {
    (instruction & 0b1111_1111_1111_1111_1110_0000_0000_0000) as i32
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

/// I-type immediates are sign extended 12 bit immediates packed into the last 12 bits of
/// the 32-bit instruction.
pub const fn i_type_immediate_32(instruction: u32) -> i32 {
    // The sign bit is already at bit 31 so we just extract it with a mask. We need to shift
    // the other 12 bits.
    let sign_bit = extract(instruction, 0, SIGN_BIT) != 0;
    let raw_value = extract(instruction, 20, C_11_BITS);

    // Sign extend the result (fill the extended area with 1s if negative and 0s if positive).
    sign_extend_32(raw_value, 11, sign_bit)
}

/// S-type immediates are sign extended 12 bit immediates packed into bits (7-11) for imm(0:4)
/// and bits 25-31 for bits 5-11 (inclusive).
pub const fn s_type_immediate_32(instruction: u32) -> i32 {
    // The sign bit is already at bit 31 so we just extract it with a mask. We need to shift
    // the other 12 bits.
    let sign_bit = extract(instruction, 0, SIGN_BIT) != 0;
    let upper_6_bits = extract(instruction, 25, C_6_BITS) << 5;
    let lower_5_bits = extract(instruction, 7, C_5_BITS);
    sign_extend_32(upper_6_bits | lower_5_bits, 11, sign_bit)
}

/// B-type immediates are sign extended 13 bit immediates packed into bits (8-11) for imm(1:4),
/// bits 25-30 for imm(5:10), bit 7 for imm(11) and bit 31 for imm(12). imm(0) is always 0
/// (inclusive).
pub const fn b_type_immediate_32(instruction: u32) -> i32 {
    let sign_bit = instruction & SIGN_BIT != 0;
    let bit_11 = extract(instruction, 7, 0b1) << 11;
    let bits_10_to_5 = extract(instruction, 25, C_6_BITS) << 5;
    let bits_4_to_1 = extract(instruction, 8, C_4_BITS) << 1;
    sign_extend_32(bit_11 | bits_10_to_5 | bits_4_to_1, 12, sign_bit)
}

/// J-type the upper 20 bits of a 21-bit value (sign extended) in the order
/// bits 21-30 => imm(1:10), bit 20 => imm(11), bits 12-19 => imm(12:19), bits 31 => imm(20)
/// (The sign extend bit).
pub const fn j_type_immediate_32(instruction: u32) -> i32 {
    let sign_bit = instruction & SIGN_BIT != 0;
    let bit_11 = extract(instruction, 20, 0b1) << 11;
    let bits_10_to_1 = extract(instruction, 21, C_10_BITS) << 1;
    let bits_12_to_19 = extract(instruction, 12, C_8_BITS) << 12;
    sign_extend_32(bits_12_to_19 | bit_11 | bits_10_to_1, 20, sign_bit)
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
            // We OR this with zero to set some unrelated bits, testing
            // that we're not accidentally just coming to the same value.
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
        const fn pack_signed(v: i32) -> u32 {
            pack(v as u32)
        }

        const MINUS_ONE: u32 = pack_signed(-1);
        const MINUS_TWO_HUNDRED_AND_FIFTY: u32 = pack_signed(-250);
        const MIN_NEGATIVE_VALUE: u32 = pack_signed(-2048);

        assert_eq!(i_type_immediate_32(MINUS_TWO_HUNDRED_AND_FIFTY), -250);
        assert_eq!(i_type_immediate_32(MINUS_ONE), -1);
        assert_eq!(i_type_immediate_32(MIN_NEGATIVE_VALUE), -2048);
    }

    #[test]
    fn test_s_type_immediate() {
        const ZERO: u32 = 0b0000_0001_0101_1111_0001_0000_0101_0110;
        const fn pack(v: u32) -> u32 {
            let v = v & 0b1111_1111_1111;
            let higher_7_bits = v & 0b1111_1110_0000;
            let lower_5_bits = v & 0b1_1111;

            // We OR this with zero to set some unrelated bits, testing
            // that we're not accidentally just coming to the same value.
            ZERO | ((higher_7_bits >> 5) << 25) | (lower_5_bits << 7)
        }

        // Test positive values
        const ONE: u32 = pack(1);
        const TWO_HUNDRED_AND_FIFTY: u32 = pack(250);
        const MAX_POSITIVE_VALUE: u32 = pack(2047);

        assert_eq!(s_type_immediate_32(ZERO), 0);
        assert_eq!(s_type_immediate_32(ONE), 1);
        assert_eq!(s_type_immediate_32(TWO_HUNDRED_AND_FIFTY), 250);
        assert_eq!(s_type_immediate_32(MAX_POSITIVE_VALUE), 2047);

        const fn pack_signed(v: i32) -> u32 {
            pack(v as u32)
        }

        const MINUS_ONE: u32 = pack_signed(-1);
        const MINUS_TWO_HUNDRED_AND_FIFTY: u32 = pack_signed(-250);
        const MIN_NEGATIVE_VALUE: u32 = pack_signed(-2048);

        assert_eq!(s_type_immediate_32(MINUS_TWO_HUNDRED_AND_FIFTY), -250);
        assert_eq!(s_type_immediate_32(MINUS_ONE), -1);
        assert_eq!(s_type_immediate_32(MIN_NEGATIVE_VALUE), -2048);
    }

    #[test]
    fn test_b_type_immediate() {
        const ZERO: u32 = 0b0000_0001_0101_1111_0001_0000_0101_0110;

        const fn pack(v: u32) -> u32 {
            if v & 1 != 0 {
                panic!("impossible to pick, lowest bit is set");
            }

            // The input is 13 bits compressed into 12 (assuming v & 1 == 0)
            let v = (v & 0b1_1111_1111_1111) >> 1;

            let bit_twelve = (v & 0b1000_0000_0000) >> 11;
            let bit_eleven = (v & 0b0100_0000_0000) >> 10;
            let bits_ten_to_five = (v & 0b0011_1111_0000) >> 4;
            let lower_four_bits = v & 0b1111;

            ZERO | (bit_twelve << 31)
                | (bit_eleven << 7)
                | (lower_four_bits << 8)
                | (bits_ten_to_five << 25)
        }

        // Test positive values
        const TWO: u32 = pack(2);
        const TWO_HUNDRED_AND_FIFTY: u32 = pack(250);
        const MAX_POSITIVE_VALUE: u32 = pack(4094);

        assert_eq!(b_type_immediate_32(ZERO), 0);
        assert_eq!(b_type_immediate_32(TWO), 2);
        assert_eq!(b_type_immediate_32(TWO_HUNDRED_AND_FIFTY), 250);
        assert_eq!(b_type_immediate_32(MAX_POSITIVE_VALUE), 4094);

        const fn pack_signed(v: i32) -> u32 {
            pack(v as u32)
        }

        // Test negative values
        const MINUS_TWO: u32 = pack_signed(-2);
        const MINUS_TWO_HUNDRED_AND_FIFTY: u32 = pack_signed(-250);
        const MIN_NEGATIVE_VALUE: u32 = pack_signed(-4096);

        assert_eq!(b_type_immediate_32(MINUS_TWO_HUNDRED_AND_FIFTY), -250);
        assert_eq!(b_type_immediate_32(MINUS_TWO), -2);
        assert_eq!(b_type_immediate_32(MIN_NEGATIVE_VALUE), -4096);
    }

    #[test]
    fn test_j_type_immediate() {
        const ZERO: u32 = 0b0000_0000_0000_0000_0000_0110_0101_0110;

        const fn pack(v: u32) -> u32 {
            if v & 1 != 0 {
                panic!("impossible to pick, lowest bit is set");
            }

            // The input is 21 bits compressed into 20 (assuming v & 1 == 0)
            let v = (v & 0b1_1111_1111_1111_1111_1110) >> 1;

            let bit_twenty = (v & 0b1000_0000_0000_0000_0000) >> 19;
            let bit_eleven = (v & 0b0000_0000_0100_0000_0000) >> 10;
            let bits_ten_to_one = (v & 0b11_1111_1111);
            let bits_19_to_12 = (v & 0b0111_1111_1000_0000_0000) >> 11;

            ZERO | (bit_twenty << 31)
                | (bit_eleven << 20)
                | (bits_ten_to_one << 21)
                | (bits_19_to_12 << 12)
        }

        // Test positive values
        const TWO: u32 = pack(2);
        const TWO_HUNDRED_AND_FIFTY: u32 = pack(250);
        const MAX_POSITIVE_VALUE: u32 = pack(1048574);

        assert_eq!(j_type_immediate_32(ZERO), 0);
        assert_eq!(j_type_immediate_32(TWO), 2);
        assert_eq!(j_type_immediate_32(TWO_HUNDRED_AND_FIFTY), 250);
        assert_eq!(j_type_immediate_32(MAX_POSITIVE_VALUE), 1048574);

        const fn pack_signed(v: i32) -> u32 {
            pack(v as u32)
        }

        // Test negative values
        const MINUS_TWO: u32 = pack_signed(-2);
        const MINUS_TWO_HUNDRED_AND_FIFTY: u32 = pack_signed(-250);
        const MIN_NEGATIVE_VALUE: u32 = pack_signed(-1048576);

        assert_eq!(j_type_immediate_32(MIN_NEGATIVE_VALUE), -1048576);
        assert_eq!(j_type_immediate_32(MINUS_TWO_HUNDRED_AND_FIFTY), -250);
        assert_eq!(j_type_immediate_32(MINUS_TWO), -2);
    }
}
