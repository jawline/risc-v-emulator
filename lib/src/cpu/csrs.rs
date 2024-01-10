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
            0xC00 =>
            //Read the lower 32 bits of rdcycle
            {
                Ok(lower(self.rdcycle))
            }
            0xC80 => Ok(upper(self.rdcycle)),
            _ => Err(IllegalCsrAddress),
        }
    }

    pub fn set(&mut self, address: usize, value: u32) -> Result<(), IllegalCsrAddress> {
        match address {
            0xC00 => {
                // Set the lower 32 bits
                self.rdcycle = setlower(self.rdcycle, value);
                Ok(())
            }
            0xC80 => {
                // Set the upper 32 bits
                self.rdcycle = setupper(self.rdcycle, value);
                Ok(())
            }
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
}
