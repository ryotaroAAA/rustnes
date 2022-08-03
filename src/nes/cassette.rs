use std::fs::File;
use std::io::prelude::*;

pub const PROG_ROM_MAX_SIZE: usize = 0x8000;
pub const CHAR_ROM_MAX_SIZE: usize = 0x2000;
pub const PROG_ROM_UNIT_SIZE: usize = 0x4000;
pub const CHAR_ROM_UNIT_SIZE: usize = 0x2000;
pub const NES_HSIZE: usize = 0x0010;

#[derive(Debug)]
pub struct Cassette {
    path: String,
    rom_size: u64,
    prog_size: usize,
    char_size: usize,
    prog_rom: Vec<u8>,
    char_rom: Vec<u8>
}

impl Cassette {
    pub fn new(path: &str) -> Cassette{
        let mut f = File::open(path).expect("file not found");

        let mut buf = Vec::new();
        let size = f.read_to_end(&mut buf);
    
        let prog_size: usize = (buf[4] as usize) * PROG_ROM_UNIT_SIZE;
        let char_size: usize = (buf[5] as usize) * CHAR_ROM_UNIT_SIZE;
        let prog_rom_s: usize = NES_HSIZE;
        let char_rom_s: usize = prog_rom_s + prog_size;
        let prog_rom: Vec<u8> = buf[prog_rom_s..(prog_rom_s + prog_size)].to_vec();
        let char_rom: Vec<u8> = buf[char_rom_s..(char_rom_s + char_size)].to_vec();
    
        let metadata = std::fs::metadata(path);
        let rom_size: u64 = metadata.unwrap().len();

        println!("{:?}", &buf[..10]);
        println!("{:?}, {:?}, {:?}", rom_size, prog_size, char_size);
        
        Cassette {
            path: path.to_string(),
            rom_size: rom_size,
            prog_size: prog_size,
            char_size: char_size,
            prog_rom: prog_rom,
            char_rom: char_rom
        }        
    }
}