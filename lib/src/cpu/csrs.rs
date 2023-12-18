#[derive(Debug)]
pub struct Csrs {
    pub rdcycle: u64,
    pub instret: u64,
    pub rdtime: u64,
}

impl Csrs {
    pub fn new() -> Self {
        Csrs {
            rdcycle: 0,
            instret: 0,
            rdtime: 0,
        }
    }
}
