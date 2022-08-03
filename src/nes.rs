pub mod cassette;
pub mod cpu;
pub mod ram;

use crate::nes::cpu::*;
use crate::nes::ram::Ram;
use crate::nes::cassette::Cassette;

const WRAM_SIZE: usize = 0x0800;
const VRAM_SIZE: usize = 0x0800;

#[derive(Debug)]
pub struct Nes {
    cas: Cassette,
    cpu: Cpu,
    wram: Ram,
}

impl Nes {
    pub fn new(cassette: &str) -> Nes {
        Nes {
            cas: Cassette::new(cassette),
            cpu: Cpu::new(),
            wram: Ram::new(WRAM_SIZE)
        }
    }
}
