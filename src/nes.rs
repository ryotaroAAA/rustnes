pub mod cassette;
pub mod cpu;
pub mod ppu;
pub mod render;
pub mod ram;
pub mod optable;

use crate::nes::cpu::*;
use crate::nes::ppu::*;
use crate::nes::render::*;
use crate::nes::ram::Ram;
use crate::nes::cassette::Cassette;

const WRAM_SIZE: usize = 0x0800;
const VRAM_SIZE: usize = 0x0800;

#[derive(Debug)]
pub struct Context<'a> {
    cas: &'a Cassette,
    wram: &'a mut Ram,
    vram: &'a mut Ram,
    image: &'a mut Image,
    // cpu: &'a mut Cpu<'a>,
}

impl<'a> Context<'a> {
    pub fn new (
        cas: &'a Cassette,
        wram: &'a mut Ram,
        vram: &'a mut Ram,
        image: &'a mut Image,
        // cpu: &'a mut Cpu<'a>
    ) -> Context<'a> {
        Context {
            cas,
            wram,
            vram,
            image,
        }
    }
}

pub fn run(cassette_path: &str) {
    let mut wram: Ram = Ram::new(WRAM_SIZE);
    let mut vram: Ram = Ram::new(VRAM_SIZE);
    let cas: Cassette = Cassette::new(cassette_path);
    let mut image: Image = Image::new();
    let mut ctx: Context = Context::new(
        &cas, &mut wram, &mut vram, &mut image
    );

    let mut cpu: Cpu =Cpu::new(
        &ctx.cas, &mut ctx.wram);
    cpu.reset();

    let mut ppu: Ppu = Ppu::new(
        &cas, &mut ctx.vram, &mut ctx.image);

    let mut count: usize = 0;
    loop {
        let cycle: u16 = cpu.run();
        count += 1;
        if count > 200 {
            println!("break");
            break;
        }
    }
}