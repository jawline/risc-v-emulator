// Construct an opcode from a 3 bit column and a 2 bit row
pub const fn construct_opcode(col: u8, row: u8) -> usize {
    let col_shifted = (col & 0b111) << 2;
    let row_shifted = (row & 0b11) << 5;
    (col_shifted | row_shifted | 0b11) as usize
}

pub const fn extract(value: u32, shift: usize, mask: u32) -> u32 {
    (value >> shift) & mask
}

pub const SIGN_BIT: u32 = 0b1000_0000_0000_0000_0000_0000_0000_0000;
pub const C_11_BITS: u32 = 0b111_1111_1111;
pub const C_10_BITS: u32 = 0b11_1111_1111;
pub const C_8_BITS: u32 = 0b1111_1111;
pub const C_7_BITS: u32 = 0b111_1111;
pub const C_6_BITS: u32 = 0b11_1111;
pub const C_5_BITS: u32 = 0b1_1111;
pub const C_4_BITS: u32 = 0b1111;
pub const C_3_BITS: u32 = 0b111;
