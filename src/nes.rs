pub mod cassette;
pub mod cpu;
pub mod interrupts;
pub mod ppu;
pub mod render;
pub mod ram;
pub mod game;
pub mod optable;

extern crate sdl2;

use crate::nes::cpu::*;
use crate::nes::interrupts::*;
use crate::nes::game::*;
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
    // image: RefMut<'a, Image>,
    // cpu: &'a mut Cpu<'a>,
}

impl<'a> Context<'a> {
    pub fn new (
        cas: &'a Cassette,
        wram: &'a mut Ram,
        vram: &'a mut Ram,
        // image: RefMut<'a, Image>,
        // cpu: Test
    ) -> Context<'a> {
        Context {
            cas,
            wram,
            vram,
            // image,
        }
    }
}

pub fn run(cassette_path: &str) {
    let mut wram: Ram = Ram::new(WRAM_SIZE);
    let mut vram: Ram = Ram::new(VRAM_SIZE);
    let cas: Cassette = Cassette::new(cassette_path);
    let mut inter: Interrupts = Interrupts::new();;
    let mut ppu: Ppu = Ppu::new(&cas, &mut vram);
    let mut cpu: Cpu = Cpu::new(&cas, &mut wram);
    let mut game: Game = Game::new().unwrap();
    let mut render: Render = Render::new();

    cpu.reset(&mut ppu);

    let mut count = 0;
    loop {
        let cycle: u64 = cpu.run(&mut ppu, &mut inter);
        let is_render_ready: bool = ppu.run(cycle, &mut inter);
        
        if is_render_ready {
            render.render(&ppu.image);
            let status: GameStatus = game.run(&render.data).unwrap();
            if status == GameStatus::Exit {
                println!("Exit...");
                break;
            }
        }
        // count += 1;
        // if count > 100 {
        //     break;
        // }
    }
}