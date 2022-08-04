pub mod cassette;
pub mod cpu;
pub mod ram;

use crate::nes::cpu::*;
use crate::nes::ram::Ram;
use crate::nes::cassette::Cassette;

const WRAM_SIZE: usize = 0x0800;
const VRAM_SIZE: usize = 0x0800;

#[derive(Debug)]
pub struct Context<'a> {
    cas: &'a Cassette,
    wram: &'a mut Ram,
    vram: &'a mut Ram,
    // cpu: &'a mut Cpu<'a>,
}

impl<'a> Context<'a> {
    pub fn new (
        cas: &'a Cassette,
        wram: &'a mut Ram,
        vram: &'a mut Ram,
        // cpu: &'a mut Cpu<'a>
    ) -> Context<'a> {
        Context {
            cas,
            wram,
            vram,
            // cpu
        }
    }
}

pub fn run(cassette_path: &str) {
    let mut count: usize = 0;
    let mut wram: Ram = Ram::new(WRAM_SIZE);
    let mut vram: Ram = Ram::new(VRAM_SIZE);
    let cas: Cassette = Cassette::new(cassette_path);
    let mut ctx: Context = Context::new(&cas, &mut wram, &mut vram);

    let mut cpu: Cpu = Cpu::new(&mut ctx);
    cpu.reset();

    loop {
        let cycle: u16 = cpu.run();
        count += 1;
        if count > 100 {
            println!("break");
            break;
        }
    }
}