pub trait Setbits {
    fn setbits(bits: usize) -> Self;
}

impl Setbits for u32 {
    fn setbits(bits: usize) -> u32 {
        u32::MAX >> (u32::BITS - (bits as u32))
    }
}

impl Setbits for u64 {
    fn setbits(bits: usize) -> u64 {
        u64::MAX >> (u64::BITS - (bits as u32))
    }
}

impl Setbits for i32 {
    fn setbits(bits: usize) -> i32 {
        u32::setbits(bits) as i32
    }
}

impl Setbits for i64 {
    fn setbits(bits: usize) -> i64 {
        u64::setbits(bits) as i64
    }
}
