use std::fs::File;
use std::io::prelude::*;
use std::str;

pub const PROG_ROM_MAX_SIZE: usize = 0x8000;
pub const CHAR_ROM_MAX_SIZE: usize = 0x2000;
pub const PROG_ROM_UNIT_SIZE: usize = 0x4000;
pub const CHAR_ROM_UNIT_SIZE: usize = 0x2000;
pub const NES_HSIZE: usize = 0x0010;

#[derive(Debug)]
pub struct Cassette {
    path: String,
    rom_size: u64,
    pub is_horizontal_mirror: bool,
    pub mapper: u8,
    pub prog_size: usize,
    pub char_size: usize,
    pub prog_rom: Vec<u8>,
    pub char_rom: Vec<u8>
}

impl Cassette {
    pub fn new(path: &str) -> Cassette{
        let mut f = File::open(path).expect("file not found");

        let mut buf = Vec::new();
        let size = f.read_to_end(&mut buf);

        let temp = &buf[0..3].to_vec();
        let magic = str::from_utf8(temp).unwrap();
        if magic != "NES" {
            panic!("invalid rom file!");
        }
        let prog_size: usize = (buf[4] as usize) * PROG_ROM_UNIT_SIZE;
        let char_size: usize = (buf[5] as usize) * CHAR_ROM_UNIT_SIZE;
        let prog_rom_s: usize = NES_HSIZE;
        let char_rom_s: usize = prog_rom_s + prog_size;
        let prog_rom: Vec<u8> = buf[prog_rom_s..(prog_rom_s + prog_size)].to_vec();
        let char_rom: Vec<u8> = buf[char_rom_s..(char_rom_s + char_size)].to_vec();

        let metadata = std::fs::metadata(path);
        let rom_size: u64 = metadata.unwrap().len();

        let is_horizontal_mirror = (buf[6] & 0x01) != 0x01;
        let mapper = ((buf[6] & 0xF0) >> 4) | buf[7] & 0xF0;

        println!("prog rom [{:#06X}:{:#06X}], {:#X}",
            prog_rom_s, prog_rom_s + prog_size, prog_size);
        println!("char rom [{:#06X}:{:#06X}], {:#X}",
            char_rom_s, char_rom_s + char_size, char_size);
        println!("is_horizontal_mirror: {:?}", is_horizontal_mirror);
        println!("mapper: {:?}", mapper);

        println!("{:?}", &buf[0..16]);

        Cassette {
            path: path.to_string(),
            rom_size: rom_size,
            is_horizontal_mirror: is_horizontal_mirror,
            mapper: mapper,
            prog_size: prog_size,
            char_size: char_size,
            prog_rom: prog_rom,
            char_rom: char_rom
        }        
    }

    pub fn prog_rom_read(&self, addr: u16) -> u8 {
        self.prog_rom[addr as usize]
    }

    pub fn char_rom_read(&self, addr: u16) -> u8 {
        self.char_rom[addr as usize]
    }
}