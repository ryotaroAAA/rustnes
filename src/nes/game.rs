#![allow(unused_variables)]

use super::cpu::*;
use super::ppu::*;

extern crate sdl2;
use sdl2::*;
use sdl2::rect::Rect;
use sdl2::video::*;
use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::gfx::framerate::FPSManager;

pub const SCALE: u32 = 2;
pub const FPS: u32 = 60;
pub const PAD_DELAY: usize = 10;
pub const PAD_INTERVAL: usize = 10;

// #[derive(Debug)]
pub struct Game {
    canvas: sdl2::render::Canvas<Window>,
    sdl_context: Sdl,
    fps_manager: FPSManager
}

#[derive(Debug, PartialEq)]
pub enum GameStatus {
    Exit,
    Ok
}

#[derive(Debug, PartialEq)]
pub enum UpdateMode {
    Game,
    NameTable,
    PatternTable,
}


impl Game {
    pub fn new() -> Result<Game, Box<dyn std::error::Error>> {
        let sdl_context: Sdl = sdl2::init()?;
        let video_subsystem = sdl_context.video()?;
        let window = video_subsystem
            .window(
                "rustness",
                3 * SCALE * H_SIZE as u32,
                2 * SCALE * V_SIZE as u32
            )
            .position_centered()
            .build()
            .map_err(|e| e.to_string())?;
        let mut canvas = window
            .into_canvas()
            .software()
            .build()
            .map_err(|e| e.to_string())?;
        let mut fps_manager = FPSManager::new();
        _ = fps_manager.set_framerate(FPS);

        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.clear();
        canvas.present();

        Ok(Game {
            canvas: canvas,
            sdl_context: sdl_context,
            fps_manager: fps_manager
        })  
    }

    pub fn check_key(
        &mut self, cpu: &mut Cpu)
    -> Result<GameStatus, Box<dyn std::error::Error>> {
        for event in self.sdl_context.event_pump()?.poll_iter() {
            match event {
                Event::Quit { .. } | Event::KeyDown {
                    keycode: Option::Some(Keycode::Escape), ..
                } => return Ok(GameStatus::Exit),
                Event::KeyDown {keycode: Option::Some(Keycode::A), ..} => {
                    cpu.keypad1.a = true;
                },
                Event::KeyUp {keycode: Option::Some(Keycode::A), ..} => {
                    cpu.keypad1.a = false;
                },
                Event::KeyDown {keycode: Option::Some(Keycode::S), ..} => {
                    cpu.keypad1.b = true;
                },
                Event::KeyUp {keycode: Option::Some(Keycode::S), ..} => {
                    cpu.keypad1.b = false;
                },
                Event::KeyDown {keycode: Option::Some(Keycode::D), ..} => {
                    cpu.keypad1.start = true;
                },
                Event::KeyUp {keycode: Option::Some(Keycode::D), ..} => {
                    cpu.keypad1.start = false;
                },
                Event::KeyDown {keycode: Option::Some(Keycode::F), ..} => {
                    cpu.keypad1.select = true;
                },
                Event::KeyUp {keycode: Option::Some(Keycode::F), ..} => {
                    cpu.keypad1.select = false;
                },
                Event::KeyDown {keycode: Option::Some(Keycode::Up), ..} => {
                    cpu.keypad1.up = true;
                },
                Event::KeyUp {keycode: Option::Some(Keycode::Up), ..} => {
                    cpu.keypad1.up = false;
                },
                Event::KeyDown {keycode: Option::Some(Keycode::Down), ..} => {
                    cpu.keypad1.down = true;
                },
                Event::KeyUp {keycode: Option::Some(Keycode::Down), ..} => {
                    cpu.keypad1.down = false;
                },
                Event::KeyDown {keycode: Option::Some(Keycode::Left), ..} => {
                    cpu.keypad1.left = true;
                },
                Event::KeyUp {keycode: Option::Some(Keycode::Left), ..} => {
                    cpu.keypad1.left = false;
                },
                Event::KeyDown {keycode: Option::Some(Keycode::Right), ..} => {
                    cpu.keypad1.right = true;
                },
                Event::KeyUp {keycode: Option::Some(Keycode::Right), ..} => {
                    cpu.keypad1.right = false;
                },
                _ => {}
            }
        }
        Ok(GameStatus::Ok)
    }

    pub fn update(
        &mut self, data: &Vec<Vec<u64>>, mode: UpdateMode
    ) -> Result<GameStatus, Box<dyn std::error::Error>> {
        let mut count: u128 = 0;

        let base = match mode {
            UpdateMode::Game => (0,0),
            UpdateMode::NameTable => (SCALE as usize * H_SIZE, 0usize),
            UpdateMode::PatternTable => (0usize, SCALE as usize * V_SIZE),
            _ => panic!("invalid mode {:?}", mode),
        };

        for i in 0..data.len(){
            for j in 0..data[0].len(){
                // print!("{}", if *b > 0x050505 {"#"} else {" "})
                let r: u8 = ((data[i][j] & 0xFF0000) >> 16) as u8;
                let g: u8 = ((data[i][j] & 0x00FF00) >> 8) as u8;
                let b: u8 = (data[i][j] & 0x0000FF) as u8;
                self.canvas.set_draw_color(Color::RGB(r, g, b));
                _ = self.canvas.fill_rect(Rect::new(
                    (base.0 + SCALE as usize * j) as i32,
                    (base.1 + SCALE as usize * i) as i32,
                    SCALE,
                    SCALE));
            }
        }
        self.canvas.present();
        self.fps_manager.delay();
        // dbg!(self.fps_manager.get_framerate());
        
        Ok(GameStatus::Ok)
    }
}