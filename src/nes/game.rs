#![allow(unused_variables)]

use super::ppu::*;

extern crate sdl2;
use sdl2::*;
use sdl2::rect::Rect;
use sdl2::video::*;
use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::gfx::framerate::FPSManager;

pub const SCALE: u32 = 1;
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

impl Game {
    pub fn new() -> Result<Game, Box<dyn std::error::Error>> {
        let sdl_context: Sdl = sdl2::init()?;
        let video_subsystem = sdl_context.video()?;
      
        let window = video_subsystem
          .window("rustness", SCALE * H_SIZE as u32, SCALE * V_SIZE as u32)
          .position_centered()
          .build()
          .map_err(|e| e.to_string())?;
        let mut canvas = window
          .into_canvas()
          .software()
          .build()
          .map_err(|e| e.to_string())?;

        let mut fps_manager = FPSManager::new();
        fps_manager.set_framerate(FPS);
      
        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.clear();
        canvas.present();

        Ok(Game {
            canvas: canvas,
            sdl_context: sdl_context,
            fps_manager: fps_manager
        })  
    }

    pub fn run(
        &mut self, data: &Vec<Vec<u64>>)
    -> Result<GameStatus, Box<dyn std::error::Error>> {
        let mut count: u128 = 0;
        for event in self.sdl_context.event_pump()?.poll_iter() {
            match event {
            Event::Quit { .. }
            | Event::KeyDown {
                keycode: Option::Some(Keycode::Escape),
                ..
            } => return Ok(GameStatus::Exit),
            _ => {}
            }
        }

        for i in 0..data.len(){
            for j in 0..data[0].len(){
                // print!("{}", if *b > 0x050505 {"#"} else {" "})
                let r: u8 = ((data[i][j] & 0xFF0000) >> 16) as u8;
                let g: u8 = ((data[i][j] & 0x00FF00) >> 8) as u8;
                let b: u8 = (data[i][j] & 0x0000FF) as u8;
                self.canvas.set_draw_color(Color::RGB(r, g, b));
                _ = self.canvas.fill_rect(Rect::new(
                    (SCALE as usize * j) as i32,
                    (SCALE as usize * i) as i32,
                    SCALE,
                    SCALE));
            }
        }
        self.canvas.present();
        self.fps_manager.delay();
        dbg!(self.fps_manager.get_framerate());
        
        Ok(GameStatus::Ok)
    }
}