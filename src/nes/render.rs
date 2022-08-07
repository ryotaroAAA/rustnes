#![allow(unused_variables)]

use core::panic;
use std::collections::HashMap;
use super::Cassette;
use super::Ram;
use super::ppu::*;
use super::Context;
use std::rc::*;
use std::cell::*;

const COLORS: [u64; 64] = [
    0x808080, 0x003DA6, 0x0012B0, 0x440096,
    0xA1005E, 0xC70028, 0xBA0600, 0x8C1700,
    0x5C2F00, 0x104500, 0x054A00, 0x00472E,
    0x004166, 0x000000, 0x050505, 0x050505,
    0xC7C7C7, 0x0077FF, 0x2155FF, 0x8237FA,
    0xEB2FB5, 0xFF2950, 0xFF2200, 0xD63200,
    0xC46200, 0x358000, 0x058F00, 0x008A55,
    0x0099CC, 0x212121, 0x090909, 0x090909,
    0xFFFFFF, 0x0FD7FF, 0x69A2FF, 0xD480FF,
    0xFF45F3, 0xFF618B, 0xFF8833, 0xFF9C12,
    0xFABC20, 0x9FE30E, 0x2BF035, 0x0CF0A4,
    0x05FBFF, 0x5E5E5E, 0x0D0D0D, 0x0D0D0D,
    0xFFFFFF, 0xA6FCFF, 0xB3ECFF, 0xDAABEB,
    0xFFA8F9, 0xFFABB3, 0xFFD2B0, 0xFFEFA6,
    0xFFF79C, 0xD7E895, 0xA6EDAF, 0xA2F2DA,
    0x99FFFC, 0xDDDDDD, 0x111111, 0x111111
];

#[derive(Debug)]
pub struct Render<'a> {
    pub data: Vec<Vec<u64>>,
    image: &'a Image
}

impl<'a> Render<'a> {
    pub fn new(image: &Image) -> Render {
        Render {
            data: vec![vec![0; H_SIZE]; V_SIZE],
            image: image,
        }
    }

    pub fn render(&mut self) {
        self.render_background();
        self.render_sprite();
    }

    fn should_pixel_hide(&self, x: u8, y: u8) -> bool{
        let tile_x: u8 = x / 8;
        let tile_y: u8 = y / 8;
        // let index: u8 = tile_y * 32 + tile_x;
        self.image.background[tile_y as usize][tile_x as usize]
            .sprite.data[y as usize][x as usize] > 0
    }

    fn render_background(&mut self) {
        for i in 0..V_SPRITE_NUM {
            for j in 0..H_SPRITE_NUM {
                let x: u8 = (j as u8 % 32) * 8;
                let y: u8 = (j as u8 / 32) * 8;
                self.render_tile(i as u8, j as u8, x, y);
            }
        }
    }

    fn render_tile(&mut self, sprite_x: u8, sprite_y: u8, tile_x: u8, tile_y: u8) {
        // dbg!(sprite_x, sprite_y);
        // dbg!(sprite_x, sprite_y, tile_x, tile_y);
        let tile:&Tile = &self
            .image
            .background[sprite_x as usize][sprite_y as usize];
        let palette_id: u16 = tile.palette_id;
        // let data = tile.sprite.data;
        for i in 0..8 {
            for j in 0..8 {
                let color_id: u8 = self.image.palette[(palette_id * 4 +
                    tile.sprite.data[i as usize][j as usize] as u16 + 0x10) as usize];
                let x: u8 = (tile_x + j as u8 - tile.scroll_x);
                let y: u8 = (tile_y + i as u8 - tile.scroll_y) % V_SIZE as u8;
                self.data[y as usize % V_SIZE][x as usize % H_SIZE] =
                    COLORS[color_id as usize];
            }
        }
    }

    fn render_sprite(&mut self) {
        for sprite in self.image.sprite.iter() {
            let palette:[u8; PALETTE_SIZE] = self.image.palette;
            let is_vertical_reverse = sprite.attr & 0x80 > 0;
            let is_horizontal_reverse = sprite.attr & 0x40 > 0;
            let is_low_priority = sprite.attr & 0x20 > 0;
            let palette_id = sprite.attr & 0x03;

            for i in 0..8 {
                for j in 0..8 {
                    let x = sprite.x + if is_horizontal_reverse {7-j} else {j};
                    let y = sprite.y + if is_horizontal_reverse {7-i} else {i};
                    if is_low_priority && self.should_pixel_hide(x, y) {
                        continue;
                    }
                    if sprite.data[i as usize][j as usize] > 0 {
                        let color_id = palette[(palette_id * 4 +
                            sprite.data[i as usize][j as usize] + 0x10) as usize];
                        self.data[y as usize % V_SIZE][x as usize % H_SIZE] =
                            COLORS[color_id as usize];
                    }
                }
            }
        }
    }
}
