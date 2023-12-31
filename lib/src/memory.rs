#[derive(Debug, PartialEq)]
pub enum MemoryError {
    OutOfBounds,
}

pub struct Memory(Vec<u8>);

impl Memory {
    pub fn new(sz: usize) -> Self {
        Self(vec![0; sz])
    }

    pub fn get8(&self, addr: usize) -> Result<u8, MemoryError> {
        self.0.get(addr).copied().ok_or(MemoryError::OutOfBounds)
    }

    pub fn set8(&mut self, addr: usize, val: u8) -> Result<(), MemoryError> {
        match self.0.get_mut(addr) {
            Some(elem) => {
                *elem = val;
                Ok(())
            }
            None => Err(MemoryError::OutOfBounds),
        }
    }

    pub fn get16(&self, addr: usize) -> Result<u16, MemoryError> {
        let bytes = [self.get8(addr)?, self.get8(addr + 1)?];
        Ok(u16::from_le_bytes(bytes))
    }

    pub fn set16(&mut self, addr: usize, val: u16) -> Result<(), MemoryError> {
        let bytes = val.to_le_bytes();
        self.set8(addr, bytes[0])?;
        self.set8(addr + 1, bytes[1])?;
        Ok(())
    }

    pub fn get32(&self, addr: usize) -> Result<u32, MemoryError> {
        let bytes = [
            self.get8(addr)?,
            self.get8(addr + 1)?,
            self.get8(addr + 2)?,
            self.get8(addr + 3)?,
        ];
        Ok(u32::from_le_bytes(bytes))
    }

    pub fn set32(&mut self, addr: usize, val: u32) -> Result<(), MemoryError> {
        let bytes = val.to_le_bytes();
        self.set8(addr, bytes[0])?;
        self.set8(addr + 1, bytes[1])?;
        self.set8(addr + 2, bytes[2])?;
        self.set8(addr + 3, bytes[3])?;
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn simple_8_bit_memory_tests() {
        let mut mem = Memory::new(256);

        for i in 0..256 {
            assert_eq!(mem.get8(i), Ok(0));
        }

        for i in 0..256 {
            mem.set8(i, 255u8 - (i as u8)).unwrap();
        }

        for i in 0..256 {
            assert_eq!(mem.get8(i), Ok(255u8 - (i as u8)));
        }

        for i in 0..256 {
            mem.set8(i, 0).unwrap();
        }

        for i in 0..256 {
            assert_eq!(mem.get8(i), Ok(0));
        }
    }

    #[test]
    fn simple_8_bit_memory_boundry_tests() {
        let mut mem = Memory::new(256);

        if let Err(_) = mem.set8(256, 0) {
        } else {
            panic!("expected write to fail");
        }
        if let Err(_) = mem.get8(256) {
        } else {
            panic!("expected read to fail");
        }

        if let Err(_) = mem.set8(4096, 0) {
        } else {
            panic!("expected write to fail");
        }
        if let Err(_) = mem.get8(4096) {
        } else {
            panic!("expected read to fail");
        }
    }

    #[test]
    fn simple_16_bit_memory_tests() {
        let mut mem = Memory::new(256);

        for i in (0..256).step_by(2) {
            assert_eq!(mem.get16(i), Ok(0));
        }

        for i in (0..256).step_by(2) {
            mem.set16(i, 4096_u16 - (i as u16)).unwrap();
        }

        for i in (0..256).step_by(2) {
            assert_eq!(mem.get16(i), Ok(4096_u16 - (i as u16)));
        }

        for i in (0..256).step_by(2) {
            mem.set16(i, 0).unwrap();
        }

        for i in (0..256).step_by(2) {
            assert_eq!(mem.get16(i), Ok(0));
        }
    }

    #[test]
    fn simple_16_bit_memory_boundry_tests() {
        let mut mem = Memory::new(256);

        if let Err(_) = mem.set16(255, 0) {
        } else {
            panic!("expected write to fail");
        }

        if let Err(_) = mem.get16(255) {
        } else {
            panic!("expected read to fail");
        }

        if let Err(_) = mem.set16(4096, 0) {
        } else {
            panic!("expected write to fail");
        }
        if let Err(_) = mem.get16(4096) {
        } else {
            panic!("expected read to fail");
        }
    }

    #[test]
    fn simple_32_bit_memory_tests() {
        let mut mem = Memory::new(256);
        for i in (0..256).step_by(4) {
            assert_eq!(mem.get32(i), Ok(0));
        }

        for i in (0..256).step_by(4) {
            mem.set32(i, 4096_u32 - (i as u32)).unwrap();
        }

        for i in (0..256).step_by(4) {
            assert_eq!(mem.get32(i), Ok(4096_u32 - (i as u32)));
        }

        for i in (0..256).step_by(4) {
            mem.set32(i, 0).unwrap();
        }

        for i in (0..256).step_by(4) {
            assert_eq!(mem.get32(i), Ok(0));
        }
    }

    #[test]
    fn simple_32_bit_memory_boundry_tests() {
        let mut mem = Memory::new(256);

        if let Err(_) = mem.set32(254, 0) {
        } else {
            panic!("expected write to fail");
        }

        if let Err(_) = mem.get32(254) {
        } else {
            panic!("expected read to fail");
        }

        if let Err(_) = mem.set32(4096, 0) {
        } else {
            panic!("expected write to fail");
        }
        if let Err(_) = mem.get32(4096) {
        } else {
            panic!("expected read to fail");
        }
    }
}
