#![feature(associated_type_bounds)]
#![allow(unused_variables)]

use core::panic;
use std::collections::HashMap;
// use super::Cassette;
// use super::Ram;
use super::ppu::*;
use super::render::Render;
// use super::Context;
use std::rc::*;
use std::cell::*;

extern crate sdl2;

use sdl2::Sdl;
use sdl2::rect::Rect;
use sdl2::render::*;
use sdl2::video::*;
use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::gfx::framerate::FPSManager;


pub const PAD_DELAY: usize = 10;
pub const PAD_INTERVAL: usize = 10;

// #[derive(Debug)]
pub struct Video {
    scale: u8,
    canvas: sdl2::render::Canvas<Window>,
    sdl_context: Sdl
}

impl Video {
    pub fn new(scale: u8) -> Result<Video, Box<dyn std::error::Error>> {
        let sdl_context: Sdl = sdl2::init()?;
        let video_subsystem = sdl_context.video()?;
      
        let window = video_subsystem
          .window("rustness", scale as u32 * H_SIZE as u32, scale as u32 * V_SIZE as u32)
          .position_centered()
          .build()
          .map_err(|e| e.to_string())?;
        let mut canvas = window
          .into_canvas()
          .software()
          .build()
          .map_err(|e| e.to_string())?;
      
        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.clear();
        canvas.present();

        Ok(Video {
            scale: scale,
            canvas: canvas,
            sdl_context: sdl_context
        })  
    }

    pub fn run(
        &mut self, data: &Vec<Vec<u64>>)
    -> Result<(), Box<dyn std::error::Error>> {
        let mut count: u128 = 0;
        // 'mainloop: loop {
            for event in self.sdl_context.event_pump()?.poll_iter() {
                match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Option::Some(Keycode::Escape),
                    ..
                } => {},
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
                        (self.scale as usize * j) as i32,
                        (self.scale as usize * i) as i32,
                        self.scale as u32,
                        self.scale as u32));
                }
            }
            self.canvas.present();
            // count += 1;
            if count % 1000 == 0 {
                let fps_manager = FPSManager::new();
                dbg!(fps_manager.get_framerate());
                // dbg!(fps_manager.get_frame_count());
            }
        // }
        Ok(())
    }
}