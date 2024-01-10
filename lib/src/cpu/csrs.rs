use crate::util::Setbits;

#[derive(Debug)]
pub struct Csrs {
    pub rdcycle: u64,
    pub instret: u64,
    pub rdtime: u64,
}

struct IllegalCsrAddress;

fn lower(x: u64) -> u32 {
    x as u32
}

fn upper(x: u64) -> u32 {
    (x >> 32) as u32
}

fn setlower(x: u64, value: u32) -> u64 {
    (x & !u64::setbits(32)) | (value as u64)
}

fn setupper(x: u64, value: u32) -> u64 {
    (x & u64::setbits(32)) | ((value as u64) << 32)
}

impl Csrs {
    pub fn new() -> Self {
        Csrs {
            rdcycle: 0,
            instret: 0,
            rdtime: 0,
        }
    }

    pub fn get(&self, address: usize) -> Result<u32, IllegalCsrAddress> {
        match address {
            0xC00 => Ok(lower(self.rdcycle)),
            0xC80 => Ok(upper(self.rdcycle)),
            0xC01 => Ok(lower(self.rdtime)),
            0xC81 => Ok(upper(self.rdtime)),
            0xC02 => Ok(lower(self.instret)),
            0xC82 => Ok(upper(self.instret)),
            _ => Err(IllegalCsrAddress),
        }
    }

    pub fn set(&mut self, address: usize, value: u32) -> Result<(), IllegalCsrAddress> {
        match address {
            _ => Err(IllegalCsrAddress),
        }
    }
}

#[cfg(test)]
mod test {

    #[test]
    fn test_cycle() {
        unimplemented!();
    }

    #[test]
    fn test_time() {
        unimplemented!();
    }

    #[test]
    fn test_instret() {
        unimplemented!();
    }
}
