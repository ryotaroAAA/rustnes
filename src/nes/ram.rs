#[derive(Debug)]
pub struct Ram {
    pub size: usize,
    pub data: Vec<u8>
}

impl Ram {
    pub fn new(size: usize) -> Ram{
        Ram {
            size: size,
            data: vec![0; size],
        }
    }

    pub fn read(&self, addr: u16) -> u8 {
        self.data[addr as usize]
    }

    pub fn write(&mut self, addr: u16, data: u8) {
        self.data[addr as usize] = data;
    }
}
