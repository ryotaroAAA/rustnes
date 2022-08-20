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
    pub data: Vec<Vec<u64>>,
}

impl Render {
    pub fn new() -> Render {
        Render {
            data: vec![vec![0; H_SIZE]; V_SIZE],
        }
    }

    pub fn render(&mut self, image: &Image) {
        self.render_background(image);
        self.render_sprite(image);
    }

    fn should_pixel_hide(&self, image: &Image, x: u8, y: u8) -> bool{
        // let tile_x: u8 = (x / 8) % H_SPRITE_NUM as u8;
        // let tile_y: u8 = (y / 8) % V_SPRITE_NUM as u8;
        let tile_x: u8 = (x / 8);
        let tile_y: u8 = (y / 8);
        (image.background[tile_y as usize][tile_x as usize]
            .sprite.data[(y % 8) as usize][(x % 8) as usize] % 4) > 0
    }

    fn render_background(&mut self, image: &Image) {
        for j in 0..V_SPRITE_NUM {
            for i in 0..H_SPRITE_NUM {
                self.render_tile(image, i as u8, j as u8);
            }
        }
    }

    fn render_tile(
        &mut self, 
        image: &Image,
        tile_x: u8,
        tile_y: u8
    ) {
        // println!("{} {}", &image.background.len(), &image.background[0].len());
        let tile:&Tile = &image
            .background[tile_y as usize][tile_x as usize];
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
                        if j == 0 || i == 0 {0} else {COLORS[color_id as usize]};
                }
                // if tile_y == 30 && off_y > 0 {
                // // if off_y > 0 {
                //     println!("x:{} y:{} tile_x:{} tile_y:{} i:{} j:{} off_x:{} off_y:{} {} {} {}",
                //         x, y, tile_x, tile_y, i, j, off_x, off_y, y as u8 % 240 , y as u8 % 224, y as u8);
                // }
            }
        }
    }

    fn render_sprite(&mut self, image: &Image) {
        for sprite in image.sprite.iter() {
            let palette:[u8; PALETTE_SIZE] = image.palette;
            let is_vertical_reverse = sprite.attr & 0x80 > 0;
            let is_horizontal_reverse = sprite.attr & 0x40 > 0;
            let is_low_priority = sprite.attr & 0x20 > 0;
            let palette_id = sprite.attr & 0x03;

            for i in 0..8 {
                for j in 0..8 {
                    let x = (sprite.x as i16 + if is_horizontal_reverse {7-j} else {j}) as u8;
                    let y = (sprite.y as i16 + if is_vertical_reverse {7-i} else {i}) as u8;
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
