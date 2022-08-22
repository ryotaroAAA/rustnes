// extern crate test;

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
const VRAM_SIZE: usize = 0x2000; // 2KiB?

use std::time::{Duration, Instant};

pub fn run(cassette_path: &str, is_test: bool) {
    let mut wram: Ram = Ram::new(WRAM_SIZE);
    let mut vram: Ram = Ram::new(VRAM_SIZE);
    let cas: Cassette = Cassette::new(cassette_path);
    let mut interrupts: Interrupts = Interrupts::new();
    let mut image: Image = Image::new();
    let mut apu: Apu = Apu::new();
    let mut ppu: Ppu = Ppu::new(&cas, &mut vram);
    let mut cpu: Cpu = Cpu::new(&cas, &mut wram);
    let mut game: Game = Game::new().unwrap();
    // let mut debug_bg: Game = Game::new().unwrap();
    let mut render: Render = Render::new();

    cpu.reset(&mut ppu, &mut apu, &mut interrupts);

    // let mut start = Instant::now();
    // let mut end = start.elapsed();
    loop {
        let status: GameStatus =
            game.check_key(&mut cpu).unwrap();
        let cycle: u64 = cpu.run(&mut ppu, &mut apu, &mut interrupts);
        let is_render_ready: bool = ppu.run(cycle, &mut image, &mut interrupts);
        apu.run(cycle, &mut interrupts);    

        if is_render_ready {
            render.render(&mut image);
            game.update(&render.data,
                UpdateMode::Game).unwrap();
            game.update(&render.dbg_bg_data,
                UpdateMode::NameTable).unwrap();
            game.update(&render.dbg_pattern_data,
                UpdateMode::PatternTable).unwrap();
            // debug_bg.update(&render.data).unwrap();
            // end = start.elapsed();
            // let erapsed: f32 = end.subsec_nanos() as f32 / 1_000_000_000 as f32;
            // println!("fps:{}, sec:{}", 1.0 / erapsed, erapsed);
            // start = Instant::now();
        }
        if status == GameStatus::Exit {
            println!("Exit...");
            break;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test::Bencher;

    #[bench]
    fn bench_nes(b: &mut Bencher) {
        b.iter(|| run("rom/firedemo.nes", true));
    }
    
    // ...
}