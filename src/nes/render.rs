#![allow(unused_variables)]

use super::ppu::*;

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
pub struct Render {
    is_pattern_rendered: bool,
    pub data: Vec<Vec<u64>>,
    pub dbg_bg_data: Vec<Vec<u64>>,
    pub dbg_pattern_data: Vec<Vec<u64>>,
    pub temp: u16,
}

impl Render {
    pub fn new() -> Render {
        Render {
            is_pattern_rendered: false,
            data: vec![vec![0; H_SIZE]; V_SIZE],
            dbg_bg_data: vec![vec![0; 2*H_SIZE]; 2*V_SIZE],
            dbg_pattern_data: vec![vec![0; H_SIZE]; V_SIZE],
            temp: 0,
        }
    }

    pub fn render(&mut self, image: &Image) {
        self.render_background(image);
        self.render_dbg_background(image);
        self.render_sprite(image);
        // if !self.is_pattern_rendered {
            self.render_pattern(image);
        // }
    }

    fn should_pixel_hide(&self, image: &Image, x: u8, y: u8) -> bool{
        let tile_x: u8 = (x / 8) % H_SPRITE_NUM as u8;
        let tile_y: u8 = (y / 8) % V_SPRITE_NUM as u8;
        // let tile_x: u8 = (x / 8);
        // let tile_y: u8 = (y / 8);
        (image.background[tile_y as usize][tile_x as usize]
            .sprite.data[(y % 8) as usize][(x % 8) as usize] % 4) > 0
    }

    fn render_background(&mut self, image: &Image) {
        for i in 0..V_SPRITE_NUM {
            for j in 0..H_SPRITE_NUM {
                if image.background[i][j].is_background_enable {
                    self.render_tile(image, j as u8, i as u8);
                }
            }
        }
    }

    fn render_dbg_tile(
        &mut self, 
        image: &Image,
        tile_x: u16,
        tile_y: u16
    ) {
        let tile:&Tile = &image.dbg_bg[tile_y as usize][tile_x as usize];
        if !tile.is_need_update {
            return;
        }
        let current_x: u16 = (image.current_x as u16) as u16;
        let current_y: u16 = (image.current_y as u16) as u16;
        if self.temp != image.current_y as u16 {
            // dbg!(image.current_y, current_y);
            self.temp = image.current_y as u16;
        }

        let palette_id: u16 = tile.palette_id;
        for j in 0..8 {
            for i in 0..8 {
                let x: u16 = 8 * tile_x as u16 + i as u16;
                let y: u16 = 8 * tile_y as u16 + j as u16;
                if x < 2*H_SIZE as u16 && y < 2*V_SIZE as u16 {
                    let color_id: u8 = image.palette[(palette_id * 4 +
                        tile.sprite.data[j as usize][i as usize] as u16) as usize];
                    self.dbg_bg_data[(y % (2*V_SIZE) as u16) as usize][(x % (2*H_SIZE) as u16) as usize] =
                        if i == 0 && j == 0 ||
                                x == 0 || y == 0 ||
                                x == (256 - 1) || y == (240 - 1) ||
                                x == (2 * 256 - 1) || y == (2 * 240 - 1) {
                            0xFF00FF
                        } else if x == current_x || x == current_x + H_SIZE as u16 ||
                                y == current_y || y == current_y + V_SIZE as u16 {
                            0
                        } else {
                            COLORS[color_id as usize]
                        };
                }
            }
        }
    }

    fn render_dbg_background(&mut self, image: &Image) {
        for j in 0..2*V_SPRITE_NUM {
            for i in 0..2*H_SPRITE_NUM {
                if image.dbg_bg[j][i].is_background_enable {
                    self.render_dbg_tile(image, i as u16, j as u16);
                }
            }
        }
    }

    fn render_tile(
        &mut self, 
        image: &Image,
        tile_x: u8,
        tile_y: u8
    ) {
        let tile:&Tile = &image.background[tile_y as usize][tile_x as usize];
        // if !tile.is_need_update {
        //     return;
        // }
        // if tile.sprite_id > 0 {
        //     for a in &tile.sprite.data {
        //         for b in a.iter() {
        //             print!("{:?}", b);
        //         }
        //         print!("\n");
        //     }
        // }
        let palette_id: u16 = tile.palette_id;
        let off_x: i16 = (tile.scroll_x % 8) as i16;
        let off_y: i16 = (tile.scroll_y % 8) as i16;
        for j in 0..8 {
            for i in 0..8 {
                let x: i16 = 8 * tile_x as i16 + i as i16 - off_x;
                let y: i16 = 8 * tile_y as i16 + j as i16 - off_y;
                if 0 <= x && x < H_SIZE as i16 && 0 <= y && y < V_SIZE as i16 {
                    let color_id: u8 = image.palette[(palette_id * 4 +
                        tile.sprite.data[j as usize][i as usize] as u16) as usize];
                    self.data[((y as u8) % 240) as usize][(x % 256) as usize] =
                        if i == 0 && j == 0 ||
                                x == 0 || y == 0 ||
                                x == 255 || y == 239 {
                            0x00FF00
                        } else {
                            COLORS[color_id as usize]
                        };
                }
            }
        }
    }

    fn render_pattern(&mut self, image: &Image) {
        for (i, sprite) in image.dbg_pattern.iter().enumerate() {
            let palette:[u8; PALETTE_SIZE] = image.palette;
            let palette_id = 0;
            let h = sprite.data.len();
            for i in 0..h {
                let y = sprite.y + i as u8;
                for j in 0..8 {
                    let x = sprite.x + j as u8;
                    let color_id = palette[(palette_id * 4 +
                        sprite.data[i as usize][j as usize] + 0x10) as usize];
                    self.dbg_pattern_data[y as usize % V_SIZE][x as usize % H_SIZE] =
                    if i == 0 && j == 0 ||
                            x == 0 || y == 0 || y == 64 ||
                            x == 255 || y == 239 {
                        0x0000FF
                    } else {
                        COLORS[color_id as usize]
                    };
                }
            }
        }
        // self.is_pattern_rendered = true;
    }

    fn render_sprite(&mut self, image: &Image) {
        for sprite in image.sprite.iter() {
            let palette:[u8; PALETTE_SIZE] = image.palette;
            let is_vertical_reverse = sprite.attr & 0x80 > 0;
            let is_horizontal_reverse = sprite.attr & 0x40 > 0;
            let is_low_priority = sprite.attr & 0x20 > 0;
            let palette_id = sprite.attr & 0x03;
            let h = sprite.data.len();
            for i in 0..h {
                let y = (sprite.y as i16 +
                    (if is_vertical_reverse {h-1-i} else {i}) as i16) as u8;
                for j in 0..8 {
                    let x = (sprite.x as i16 +
                        if is_horizontal_reverse {7-j} else {j}) as u8;
                    if is_low_priority && self.should_pixel_hide(image, x, y) {
                        continue;
                    }
                    if sprite.data[i as usize][j as usize] > 0 &&
                            !(y >= V_SIZE as u8) {
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
