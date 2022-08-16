pub mod apu;
pub mod cassette;
pub mod cpu;
pub mod interrupts;
pub mod ppu;
pub mod render;
pub mod ram;
pub mod game;
pub mod optable;

extern crate sdl2;

use crate::nes::apu::*;
use crate::nes::cpu::*;
use crate::nes::interrupts::*;
use crate::nes::game::*;
use crate::nes::ppu::*;
use crate::nes::render::*;
use crate::nes::ram::Ram;
use crate::nes::cassette::Cassette;

const WRAM_SIZE: usize = 0x0800; // 2KiB
const VRAM_SIZE: usize = 0x0800; // 2KiB

pub fn run(cassette_path: &str) {
    let mut wram: Ram = Ram::new(WRAM_SIZE);
    let mut vram: Ram = Ram::new(VRAM_SIZE);
    let cas: Cassette = Cassette::new(cassette_path);
    let mut interrupts: Interrupts = Interrupts::new();
    let mut apu: Apu = Apu::new();
    let mut ppu: Ppu = Ppu::new(&cas, &mut vram);
    let mut cpu: Cpu = Cpu::new(&cas, &mut wram);
    let mut game: Game = Game::new().unwrap();
    let mut render: Render = Render::new();

    cpu.reset(&mut ppu, &mut apu, &mut interrupts);
    loop {
        let status: GameStatus =
            game.check_key(&mut cpu).unwrap();
        let cycle: u64 = cpu.run(&mut ppu, &mut apu, &mut interrupts);
        let is_render_ready: bool = ppu.run(cycle, &mut interrupts);
        apu.run(cycle, &mut interrupts);        

        if is_render_ready {
            render.render(&ppu.image);
            game.update(&render.data).unwrap();
        }
        if status == GameStatus::Exit {
            println!("Exit...");
            break;
        }
    }
}