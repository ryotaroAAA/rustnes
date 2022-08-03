#[derive(Debug)]
pub struct Ram {
    size: usize,
    data: Vec<u8>
}

impl Ram {
    pub fn new(size: usize) -> Ram{
        Ram {
            size: size,
            data: vec![0; size],
        }
    }
}
