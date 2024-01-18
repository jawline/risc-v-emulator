use crate::util::Setbits;

#[derive(Debug)]
pub struct Csrs {
    pub rdcycle: u64,
    pub instret: u64,
    pub rdtime: u64,
}

#[derive(Debug)]
pub struct IllegalCsrAddress;

fn lower(x: u64) -> u32 {
    x as u32
}

fn upper(x: u64) -> u32 {
    (x >> 32) as u32
}

fn _setlower(x: u64, value: u32) -> u64 {
    (x & !u64::setbits(32)) | (value as u64)
}

fn _setupper(x: u64, value: u32) -> u64 {
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

    pub fn set(&mut self, address: usize, _value: u32) -> Result<(), IllegalCsrAddress> {
        match address {
            _ => Err(IllegalCsrAddress),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn setup() -> Csrs {
        let mut csrs = Csrs::new();
        csrs.rdcycle = (550 << 32) | 500;
        csrs.rdtime = (55000 << 32) | 50000;
        csrs.instret = (255 << 32) | 250;
        csrs
    }

    #[test]
    fn test_cycle() {
        let csrs = setup();
        assert_eq!(csrs.get(0xC00).unwrap(), 500);
        assert_eq!(csrs.get(0xC80).unwrap(), 550);
    }

    #[test]
    fn test_time() {
        let csrs = setup();
        assert_eq!(csrs.get(0xC01).unwrap(), 50000);
        assert_eq!(csrs.get(0xC81).unwrap(), 55000);
    }

    #[test]
    fn test_instret() {
        let csrs = setup();
        assert_eq!(csrs.get(0xC02).unwrap(), 250);
        assert_eq!(csrs.get(0xC82).unwrap(), 255);
    }
}
