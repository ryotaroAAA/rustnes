#![allow(unused_variables)]

use core::panic;
use crate::nes::VRAM_SIZE;

use super::Cassette;
use super::Interrupts;
use super::Ram;
use super::interrupts;

/*
    [Control Register1 0x2000]
    | bit  | description                                 |
    +------+---------------------------------------------+
    |  7   | Assert NMI when VBlank 0: disable, 1:enable |
    |  6   | PPU master/slave, always 1                  |
    |  5   | Sprite size 0: 8x8, 1: 8x16                 |
    |  4   | background pattern table 0:0x0000, 1:0x1000         |
    |  3   | sprite pattern table 0:0x0000, 1:0x1000     |
    |  2   | PPU memory increment 0: +=1, 1:+=32         |
    |  1-0 | Name table 0x00: 0x2000                     |
    |      |            0x01: 0x2400                     |
    |      |            0x02: 0x2800                     |
    |      |            0x03: 0x2C00                     |

    [Control Register2 0x2001]
    | bit  | description                                 |
    +------+---------------------------------------------+
    |  7-5 | Background color  0x00: Black               |
    |      |                   0x01: Green               |
    |      |                   0x02: Blue                |
    |      |                   0x04: Red                 |
    |  4   | Enable sprite                               |
    |  3   | Enable background                           |
    |  2   | Sprite mask       render left end           |
    |  1   | Background mask   render left end           |
    |  0   | Display type      0: color, 1: mono         |

    [PPU MEMORY MAP]
    | addr           |  description               |
    +----------------+----------------------------+
    | 0x0000-0x0FFF  |  Pattern table#0           |
    | 0x1000-0x1FFF  |  Pattern table#1           |
    | 0x2000-0x23BF  |  Name table                |
    | 0x23C0-0x23FF  |  Attribute table           |
    | 0x2400-0x27BF  |  Name table                |
    | 0x27C0-0x27FF  |  Attribute table           |
    | 0x2800-0x2BBF  |  Name table                |
    | 0x2BC0-0x2BFF  |  Attribute table           |
    | 0x2C00-0x2FBF  |  Name Table                |
    | 0x2FC0-0x2FFF  |  Attribute Table           |
    | 0x3000-0x3EFF  |  mirror of 0x2000-0x2EFF   |
    | 0x3F00-0x3F0F  |  background Palette        |
    | 0x3F10-0x3F1F  |  sprite Palette            |
    | 0x3F20-0x3FFF  |  mirror of 0x3F00-0x3F1F   |
*/

pub const H_SIZE: usize = 256;
pub const V_SIZE: usize = 240;
pub const PALETTE_SIZE: usize = 0x20;
pub const H_SPRITE_NUM: usize = 32;
pub const V_SPRITE_NUM: usize = 30;
pub const SPRITE_RAM_SIZE: usize = 0x0100;
// const VRAM_SIZE: usize = 0x0800;
const TILE_SIZE: usize = 8;
const V_SIZE_WITH_VBLANK: usize = 262;
const CYCLE_PER_LINE: usize = 341;

#[derive(Debug, Clone)]
pub struct Sprite {
    pub x: u8,
    pub y: u8,
    pub attr: u8,
    pub data: Vec<Vec<u8>>,
}

impl Sprite {
    fn new() -> Sprite {
        Sprite {
            x: 0,
            y: 0,
            attr: 0,
            data: vec![vec![0; 8]; 8]
        }
    }
}

#[derive(Debug, Clone)]
pub struct Tile {
    pub scroll_x: u8,
    pub scroll_y: u8,
    pub scroll_xs: [u8; 8],
    pub scroll_ys: [u8; 8],
    pub sprite_id: u16,
    pub palette_id: u16,
    pub is_need_update: bool,
    pub is_background_enable: bool,
    pub sprite: Sprite,
}

impl Tile {
    fn new() -> Tile {
        Tile {
            scroll_x: 0,
            scroll_y: 0,
            scroll_xs: [0; 8],
            scroll_ys: [0; 8],
            sprite_id: 0,
            palette_id: 0,
            is_need_update: true,
            is_background_enable: true,
            sprite: Sprite::new(),
        }
    }
}

#[derive(Debug)]
pub struct Image {
    pub sprite: Vec<Sprite>,
    pub background: Vec<Vec<Tile>>,
    pub dbg_bg: Vec<Vec<Tile>>,
    pub dbg_pattern: Vec<Sprite>,
    pub palette: [u8; PALETTE_SIZE],
    pub current_x: u8,
    pub current_y: u8,
}

impl Image {
    pub fn new() -> Image {
        Image {
            sprite: Vec::new(),
            background: vec![vec![Tile::new(); H_SPRITE_NUM]; V_SPRITE_NUM],
            dbg_bg: vec![vec![Tile::new(); H_SPRITE_NUM*2]; V_SPRITE_NUM*2],
            dbg_pattern: vec![Sprite::new(); 512],
            palette: [0; PALETTE_SIZE],
            current_x: 0,
            current_y: 0,
        }
    }
}

#[derive(Debug)]
struct Palette {
    ram: Ram
}

impl Palette {
    fn new(size: usize) -> Palette {
        Palette {
            ram: Ram::new(size)
        }
    }
    fn read(&self) -> [u8; 32]{
        let mut palette: [u8; 32] = [0u8; PALETTE_SIZE];
        for i in 0..PALETTE_SIZE {
            palette[i] = if self.is_sprite_mirror(i as u16) {
                self.ram.read(i as u16 - 0x10)
            } else if self.is_background_mirror(i as u16) {
                self.ram.read(0x00)
            } else {
                self.ram.read(i as u16)
            };
        }
        palette
    }
    fn write(&mut self, addr: u16, data: u8) {
        let addr_: u16 = self.get_palette_addr(addr);
        self.ram.write(addr_, data);
    }
    fn is_background_mirror(&self, addr: u16) -> bool{
        match addr {
            0x04 => true,
            0x08 => true,
            0x0c => true,
            _ => false
        }
    }
    fn is_sprite_mirror(&self, addr: u16) -> bool{
        match addr {
            0x10 => true,
            0x14 => true,
            0x18 => true,
            0x1c => true,
            _ => false
        }
    }
    fn get_palette_addr(&self, addr: u16) -> u16 {
        let mirror_downed: u16 = (addr & 0xFF) % 0x20;
        if self.is_sprite_mirror(mirror_downed) {
            mirror_downed - 0x10
        } else {
            mirror_downed
        }
    }
}

#[derive(Debug)]
pub struct Ppu<'a> {
    pub cycle: u64,
    pub line: u16,
    background_index: u8,
    vram_buf: u8,
    vram_addr: u16,
    vram_offset: u16,
    sprite_ram_addr: u16,
    scroll_x: u8,
    scroll_y: u8,
    is_horizontal_mirror: bool,
    is_horizontal_scroll: bool,
    is_lower_vram_addr: bool,
    creg1: u8,
    creg2: u8,
    sreg: u8,
    is_char_rom: bool,
    sprite_0_hit_switch: bool,
    already_sprite_0_hit: bool,
    palette: Palette,
    sprite_ram: Ram,
    // char_rom: Ram,
    char_ram: Ram,
    vram: &'a mut Ram,
}

impl<'a> Ppu<'a> {
    pub fn new(cas: &Cassette, vram: &'a mut Ram) -> Ppu<'a> {
        let size = if cas.char_size > 0 {
            cas.char_size
        } else {
            0x2000
        };
        let mut char_ram = Ram::new(size);
        for i in 0..cas.char_size {
            char_ram.write(i as u16, cas.char_rom[i]);
        }
        Ppu {
            cycle: 0,
            line: 0,
            background_index: 0,
            vram_buf: 0,
            vram_addr: 0,
            vram_offset: 0,
            sprite_ram_addr: 0,
            scroll_x: 0,
            scroll_y: 0,
            is_horizontal_mirror: cas.is_horizontal_mirror,
            is_horizontal_scroll: false, // vertical scroll is first
            is_lower_vram_addr: false, // higher is first
            creg1: 0,
            creg2: 0,
            sreg: 0,
            is_char_rom: cas.char_size > 0,
            sprite_0_hit_switch: false,
            already_sprite_0_hit: false,
            palette: Palette::new(PALETTE_SIZE),
            sprite_ram: Ram::new(SPRITE_RAM_SIZE),
            // char_rom: char_rom,
            char_ram: char_ram,
            vram: vram,
        }
    }
    // Control Register 1, Main Screen assignment by name table
    fn get_name_table_id(&mut self) -> u8{
        self.creg1 & 0x03
    }
    // Control Register 1, PPU memory increment
    fn get_vram_offset(&mut self) -> u8{
        if self.creg1 & 0x04 > 0 {32} else {1}
    }
    // Control Register 1, get sprite pattern table
    fn get_sprite_table_offset(&mut self) -> u16 {
        if self.creg1 & 0x08 > 0 {0x1000} else {0x0000}
    }
    // Control Register 1, get background pattern table
    fn get_background_table_offset(&mut self) -> u16{
        if self.creg1 & 0x10 > 0 {0x1000} else {0x0000}
    }
    // Control Register 1, Sprite Size
    fn is_large_sprite(&self) -> bool{
        self.creg1 & 0x20 > 0
    }
    // Control Register 1, Assert NMI when VBlank
    fn has_vblank_irq_enabled(&mut self) -> bool{
        self.creg1 & 0x80 > 0
    }
    // Control Register 2, Enable sprite
    fn get_is_background_enable(&mut self) -> bool {
        self.creg2 & 0x08 > 0
    }
    // Control Register 2, Enable sprite
    fn get_is_sprite_enable(&mut self) -> bool {
        self.creg2 & 0x10 > 0
    }
    // PPU status register
    fn set_sprite_0_hit(&mut self) {
        let sreg = self.sreg;
        self.sreg |= 0x40;
        if sreg != self.sreg {
            // println!("set_sprite_0_hit   {:08b} {:08b} {:08b} x:{:3} y:{:3} line:{:3} cyc:{:3}",
            //     self.sreg, self.creg1, self.creg2,
            //     self.scroll_x, self.scroll_y, self.line, self.cycle);
        }
    }
    // PPU status register
    fn clear_sprite_0_hit(&mut self, line: usize) {
        let sreg = self.sreg;
        self.sreg &= 0xBF;
        self.already_sprite_0_hit = false;
        if sreg != self.sreg {
            // println!("clear_sprite_0_hit {:08b} {:08b} {:08b} x:{:3} y:{:3} line:{:3} cyc:{:3}",
            //     self.sreg, self.creg1, self.creg2,
            //     self.scroll_x, self.scroll_y, line, self.cycle);
        }
    }
    // PPU status register
    fn set_vblank(&mut self) {
        self.sreg |= 0x80;
    }
    // PPU status register
    // fn get_is_vblank(&mut self) -> bool {
    //     self.sreg & 0x80 > 0
    // }
    // PPU status register
    fn clear_vblank(&mut self) {
        self.sreg &= 0x7F;
    }
    fn is_sprite_0_hit(&mut self) -> bool {
        if self.already_sprite_0_hit {
            return false;
        }
        let y: u8 = self.sprite_ram.read(0);
        let sprite_id: u16 = self.sprite_ram.read(1) as u16;
        let x = self.sprite_ram.read(3);
        let mut is_hit = false;
        if (y as u16) <= self.line && self.line < (y as u16 + 8) {
            let mut sprite = Sprite::new();
            self.build_sprite_data(
                false,
                sprite_id,
                0,
                &mut sprite,
            );
            let base_y = self.line as u8 - y;
            let mut is_not_transparent_line = false;
            for i in 0..8 {
                if sprite.data[base_y as usize][i as usize] > 0 {
                    is_not_transparent_line = true;
                    break
                }
            }

            if !is_not_transparent_line {
                return false;
            }

            is_hit = (x as u64) <= self.cycle &&
                is_not_transparent_line;
                self.get_is_sprite_enable();
            
            self.already_sprite_0_hit = is_hit;
            self.sprite_0_hit_switch = is_hit;
        }
        is_hit
    }
    fn get_scroll_tile_x(&mut self) -> u8 {
        ((self.scroll_x as u16 +
            (self.get_name_table_id() % 2) as u16 * H_SIZE as u16) / 8) as u8
    }
    fn get_scroll_tile_y(&mut self) -> u8 { 
        ((self.scroll_y as u16 + self.line +
            (self.get_name_table_id() / 2) as u16 * V_SIZE as u16) / 8 - 1) as u8 
    }
    fn get_block_id(&mut self, x: u16, y: u16) -> u8{
        ((x % 4) / 2 + ((y % 4) / 2) * 2) as u8
    }
    fn get_vram_addr(&mut self, sprite_addr: u16) -> u16 {
        if self.is_horizontal_mirror {
            match sprite_addr {
                0x0000..=0x03FF => sprite_addr,
                0x0400..=0x07FF => sprite_addr - 0x0400,
                0x0800..=0x0BFF => sprite_addr,
                0x0C00..=0x0FFF => sprite_addr - 0x0400,
                _ => panic!("invalid sprite_addr {}", sprite_addr),
            }
        } else {
            match sprite_addr {
                0x0000..=0x03FF => sprite_addr,
                0x0400..=0x07FF => sprite_addr,
                0x0800..=0x0BFF => sprite_addr - 0x0800,
                0x0C00..=0x0FFF => sprite_addr - 0x0800,
                _ => panic!("invalid sprite_addr {}", sprite_addr),
            }
        }
    }
    // read from name_table
    fn get_sprite_id(&mut self, x: u16, y: u16, offset: u16) -> u8{
        let tile_num: u16 =  x as u16 + y as u16 * 32;
        let sprite_addr: u16 =
            self.get_vram_addr(tile_num + offset);
        self.vram.read(sprite_addr)
    }
    fn get_attribute(&mut self, x: u16, y: u16, offset: u16) -> u8{
        let addr: u16 = x as u16 / 4 +
            (y as u16/ 4) * 8 +
            0x03C0 + offset;
        let sprite_addr: u16 = self.get_vram_addr(addr);
            // if x == 0 && y == 0 {
            //     println!("{} {} {:b}", offset, sprite_addr, self.vram.read(sprite_addr));
        // }
        self.vram.read(sprite_addr)
    }
    fn get_palette(&mut self, image: &mut Image) {
        image.palette = self.palette.read();
    }
    // read by cpu
    fn vram_read(&mut self) -> u8{
        let mut vram_buf: u8 = self.vram_buf;
        self.vram_addr %= 0x4000;
        match self.vram_addr {
            // pattern table from charactor rom
            0x0000..=0x1FFF => {
                self.vram_buf = self.char_ram.read(self.vram_addr);
            },
            // name table, attr table
            0x2000..=0x3EFF => {
                self.vram_buf = self.vram.read(self.vram_addr % 0x1000);
            },
            // pallette
            0x3F00..=0x4000 => {
                // vram_buf = self.vram.read(self.vram_addr - 0x3F00);
                let addr = (self.vram_addr - 0x3F00) as usize;
                vram_buf = self.palette.read()[addr];
            },
            _ => panic!("invalid addr: {}", self.vram_addr),
        }
        self.vram_addr += self.get_vram_offset() as u16;
        vram_buf as u8
    }
    pub fn read(&mut self, addr: u16) -> u8 {
        // println!(" ppu read {:#X}", addr);
        match addr {
            /*
            | bit  | description                                 |
            +------+---------------------------------------------+
            | 7    | 1: VBlank clear by reading this register    |
            | 6    | 1: sprite hit                               |
            | 5    | 0: less than 8, 1: 9 or more                |
            | 4-0  | invalid                                     |                                 
            |      | bit4 VRAM write flag [0: success, 1: fail]  |
            */
            0x0002 => {
                // PPUSTATUS
                let status: u8 = self.sreg;
                self.clear_vblank();
                let line = self.line;
                // self.clear_sprite_0_hit(line as usize);
                self.is_lower_vram_addr = false;
                self.is_horizontal_scroll = true;
                return status;
            },
            0x0004 => {
                // OAMADDR
                return self.sprite_ram.read(self.sprite_ram_addr);
            },
            0x0007 => {
                // PPUDATA
                return self.vram_read();
            },
            _ => 0,
            // _ => panic!("invalid addr {:#X}", addr)
        }
    }
    pub fn write_sprite_ram_addr(&mut self, data: u8) {
        self.sprite_ram_addr = data as u16;
    }
    pub fn write_sprite_ram_data(&mut self, data: u8) {
        self.sprite_ram.write(self.sprite_ram_addr, data);
        self.sprite_ram_addr += 1;
    }
    fn write_scroll_data(&mut self, data: u8) {
        let (x, y) = (self.scroll_x, self.scroll_y);
        if self.is_horizontal_scroll {
            self.is_horizontal_scroll = false;
            self.scroll_x = data;
        } else {
            self.scroll_y = data;
            self.is_horizontal_scroll = true;
        }
        if x != self.scroll_x || y != self.scroll_y {
            let is_sprite_0_hit = self.already_sprite_0_hit;
            // println!("write scroll from game x:{}, y:{}, already_0hit:{} scroll_val:{} line:{}",
            //     self.scroll_x,
            //     self.scroll_y,
            //     is_sprite_0_hit,
            //     data,
            //     self.line);
        }
    }
    // write by cpu
    fn write_vram_addr(&mut self, data: u8) {
        // println!("{:X} {:X} {}", self.vram_addr, data, self.is_lower_vram_addr);
        if self.is_lower_vram_addr {
            self.vram_addr += data as u16;
            self.is_lower_vram_addr = false;
        } else {
            self.vram_addr = (data as u16) << 8;
            self.is_lower_vram_addr = true;
        }
    }
    // write by cpu
    fn write_vram_data(&mut self, data: u8) {
        // println!("write_vram_data {:#06X} {:#04X}", self.vram_addr, data);
        self.vram_addr %= 0x4000;
        match self.vram_addr {
            // pattern table from charactor rom
            0x0000..=0x1FFF => {
                // println!("write_vram_data {:#06X} {:#04X}", self.vram_addr, data);
                if self.is_char_rom {
                    return;
                } else {
                    // println!("char ram write addr:{} data:{}", self.vram_addr, data);
                    self.char_ram.write(self.vram_addr, data);
                }
            },
            // name table, attr table [0x2000:0x2FFF]
            // name table, attr table [0x3000:0x3EFF] => copy of [0x2000:0x2EFF] 
            0x2000..=0x3EFF => {
                let addr: u16 = self.vram_addr % 0x1000;
                // println!("write_vram_data {:#06X} {:#06X} {:#04X}",
                    // self.vram_addr, addr, data);
                self.vram.write(addr, data);
            },
            0x3F00..=0x4000 => {
                // pallette
                let addr: u16 = (self.vram_addr - 0x3F00);
                self.palette.write(addr, data);
            },
            _ => panic!("invalid addr: {}", self.vram_addr),
        }
        self.vram_addr += self.get_vram_offset() as u16;
    }
    pub fn write(&mut self, addr: u16, data: u8) {
        // println!(" ppu write {:#X} {:#X}:{:08b}", addr, data, data);
        match addr {
            0x0000 => self.creg1 = data,
            0x0001 => self.creg2 = data,
            // set sprite ram write addr
            0x0003 => self.write_sprite_ram_addr(data),
            // sprite ram write
            0x0004 => self.write_sprite_ram_data(data),
            // set scroll setting
            0x0005 => {
                // println!("{} {} {}", addr, data, self.line);
                self.write_scroll_data(data);
            },
            // set vram write addr (first: high 8bit, second: low 8bit)
            0x0006 => self.write_vram_addr(data),
            // sprite ram write
            0x0007 => self.write_vram_data(data),
            _ => panic!("invalid addr {:#X}", addr)
        }
    }
    fn build_sprite_data(&self, is_tile: bool, sprite_id: u16, offset: u16, sprite: &mut Sprite) {
        /*
            Bit Planes                  Pixel Pattern (return value)
            [lower bit]
            $0xx0=$41  01000001
            $0xx1=$C2  11000010
            $0xx2=$44  01000100
            $0xx3=$48  01001000
            $0xx4=$10  00010000
            $0xx5=$20  00100000         .1.....3
            $0xx6=$40  01000000         11....3.
            $0xx7=$80  10000000  =====  .1...3..
            [higher bit]                .1..3...
            $0xx8=$01  00000001  =====  ...3.22.
            $0xx9=$02  00000010         ..3....2
            $0xxA=$04  00000100         .3....2.
            $0xxB=$08  00001000         3....222
            $0xxC=$16  00010110
            $0xxD=$21  00100001
            $0xxE=$42  01000010
            $0xxF=$87  10000111

            see https:#wiki.nesdev.com/w/index.php/PPU_pattern_tables
        */
        for i in 0..8 {
            for j in 0..8 {
                sprite.data[i][j] = 0;
            }
        }
        let h = if self.is_large_sprite() && !is_tile {2} else {1};
        let mut disp = false;
        for k in 0..h {
            for i in 0..16 {
                let addr: u16 = ((sprite_id + k) * 16 + i + offset) as u16;
                if addr as usize >= self.char_ram.data.len() {
                    continue
                }
                // read from pattern table
                let ram: u8 = self.char_ram.read(addr);
                if is_tile && sprite_id == 4 {
                    // println!("{:X} {:X} {:X}", ram, addr, offset);
                    disp = true;
                }
                for j in 0..8 {
                    if ram & (0x80 >> j) > 0 {
                        sprite.data[(k * 8 + (i % 8)) as usize][j] +=
                            0x01 << (i/8);
                    }
                }
            }
        }
        if disp {
            // println!("{:X} {:X}", sprite_id, offset);
            // for a in &sprite.data{
            //     for b in a.iter(){
            //         let val = if *b > 0 {"#"} else {" "};
            //         print!("{}", val);
            //     }
            //     print!(";\n");
            // }
            // print!(";\n");
            // panic!();
        }
            // sprite
    }
    fn build_sprites(&mut self, image: &mut Image) {
        // see https:#wiki.nesdev.com/w/index.php/PPU_OAM
        for i in 0..self.sprite_ram_addr/4 {
            let j: u16 = 4 * i as u16;

            let y: u8 = self.sprite_ram.read(j);
            let sprite_id: u16 = self.sprite_ram.read(j + 1) as u16;
            let attr = self.sprite_ram.read(j + 2);
            let x = self.sprite_ram.read(j + 3);

            let mut sprite: Sprite = Sprite::new();
            sprite.y = y;
            sprite.attr = attr;
            sprite.x = x;
            // println!("{}", sprite_id);
            let (sprite_id, offset) = if self.is_large_sprite() {
                let offset: u16 = 0x1000 * (sprite_id & 0x01);
                sprite.data = (0..16).into_iter().map(|_| vec![0; 16]).collect();
                (sprite_id & 0xFE, offset)
            } else {
                (sprite_id, self.get_sprite_table_offset())
            };
            // dbg!(sprite_id, offset);
            self.build_sprite_data(
                false,
                sprite_id,
                offset,
                &mut sprite
            );
            // if sprite_id > 0 {
            //     dbg!(i, j+1, sprite_id, self.sprite_ram_addr);
            //     for a in &sprite.data{
            //         for b in a.iter(){
            //             // let val = if *b > 0 {"#"} else {" "};
            //             print!("{}", *b);
            //         }
            //         print!(";\n");
            //     }
            //     print!(";\n");
            // }
            image.sprite.push(sprite);
        }
    }
    // the element of background
    fn build_tile(&mut self, image: &mut Image, x: u8, y: u8, offset: u16, i: u8, j: u8) {
        let block_id: u8 = self.get_block_id(x as u16, y as u16);
        let sprite_id: u16 = self.get_sprite_id(x as u16, y as u16, offset) as u16;
        let attr: u16 = self.get_attribute(x as u16, y as u16, offset) as u16;
        let palette_id: u16 = (attr >> (block_id * 2)) as u16 & 0x03;
        let offset: u16 = self.get_background_table_offset();
        let tile = &mut image.background[i as usize][j as usize];
        // let is_no_update =
        //     tile.sprite_id == sprite_id &&
        //     tile.palette_id == palette_id &&
        //     tile.scroll_x == self.scroll_x &&
        //     tile.scroll_y == self.scroll_y;
        // if is_no_update {
        //     tile.is_need_update = false;
        //     return;
        // } else {
        //     tile.is_need_update = true;
        // }
        tile.sprite_id = sprite_id;
        tile.palette_id = palette_id;
        // dbg!(sprite_id, offset);
        self.build_sprite_data(
            true,
            sprite_id,
            offset,
            &mut tile.sprite
        );
        tile.scroll_x = self.scroll_x;
        tile.scroll_y = self.scroll_y;
        tile.is_background_enable = true;
        tile.is_background_enable = self.get_is_background_enable();
    }
    // draw every 8 line
    fn build_background(&mut self, image: &mut Image) {
        let i : u8 = self.background_index;
        let tile_y: u8 = self.get_scroll_tile_y() % V_SPRITE_NUM as u8;
        let y_offset: u8 =
            2 * ((self.get_scroll_tile_y() / V_SPRITE_NUM as u8) % 2);
        for j in 0..H_SPRITE_NUM as u8 {
            let x: u8 = (j + self.get_scroll_tile_x()) as u8;
            let tile_x: u8 = x % H_SPRITE_NUM as u8;
            let bg_id: u8 = (x / H_SPRITE_NUM as u8) % 2 + y_offset;
            let offset: u16 = bg_id as u16 * 0x0400;
            self.build_tile(
                image,
                tile_x,
                tile_y,
                offset,
                i, j
            );
        }
        self.background_index += 1;
    }

    fn build_dbg_patterns(&mut self, image: &mut Image) {
        for i in 0..2 {
            for j in 0..256 {
                let sprite = &mut image.dbg_pattern[j + i * 256];
                sprite.x = ((j * 8) % H_SIZE) as u8;
                sprite.y = ((i * 8 * 8 + (j / H_SPRITE_NUM) * 8) % V_SIZE) as u8;
                self.build_sprite_data(
                    true,
                    j as u16,
                    (i * 0x1000) as u16,
                    sprite
                );
            } 
        }
    }

    fn build_dbg_bg(&mut self, image: &mut Image) {
        for i in 0..2*V_SPRITE_NUM {
            for j in 0..2*H_SPRITE_NUM {
                /* bg_id
                    +--+--+
                    | 0| 1|
                    +--+--+
                    | 2| 3|
                    +--+--+
                */
                let bg_id: u16 =
                    2 * ((i / V_SPRITE_NUM) % 2) as u16 +
                    (j / H_SPRITE_NUM) as u16 % 2;
                let offset: u16 = bg_id * 0x0400;
                let block_id: u8 = self.get_block_id(
                    j as u16 % H_SPRITE_NUM as u16,
                    i as u16 % V_SPRITE_NUM as u16
                );
                let sprite_id: u16 = self.get_sprite_id(
                    j as u16 % H_SPRITE_NUM as u16,
                    i as u16 % V_SPRITE_NUM as u16,
                    offset
                ) as u16;
                let attr: u8 = self.get_attribute(
                    j as u16 % H_SPRITE_NUM as u16,
                    i as u16 % V_SPRITE_NUM as u16,
                    offset
                );
                let tile: &mut Tile = &mut image.dbg_bg[i as usize][j as usize];
                tile.palette_id = 
                    ((attr as i16 >> (block_id * 2)) as i16 & 0x03) as u16;
                // let is_no_update =
                //     image.dbg_bg[i as usize][j as usize].sprite_id == sprite_id &&
                //     image.dbg_bg[i as usize][j as usize].attr == attr;
                // if is_no_update {
                //     continue;
                // }

                let background_table_offset: u16 =
                    self.get_background_table_offset();
                // image.dbg_bg[i as usize][j as usize].attr = attr;
                self.build_sprite_data(
                    true,
                    sprite_id,
                    background_table_offset,
                    &mut tile.sprite
                );
            }   
        }
    }

    pub fn run(&mut self, cycle: u64, image: &mut Image, interrupts: &mut Interrupts) -> bool{
        self.cycle += 3 * cycle;

        if self.cycle >= CYCLE_PER_LINE as u64 {
            if self.line == 0 {
                image.sprite.resize(0, Sprite::new());
            }

            if self.is_sprite_0_hit() {
                // WA
                // self.set_sprite_0_hit();
            }

            self.cycle -= CYCLE_PER_LINE as u64;
            self.line += 1;

            if self.line <= V_SIZE as u16 &&
                    self.scroll_y <= V_SIZE as u8 &&
                    self.line % TILE_SIZE as u16 == 0 {

                self.build_background(image);
                // WA
                if self.already_sprite_0_hit {
                    self.set_sprite_0_hit();
                }
            }
            if self.line == (V_SIZE as u16 + 1) {
                self.set_vblank();
                self.clear_sprite_0_hit(self.line as usize);
                interrupts.deassert_nmi();
                if self.has_vblank_irq_enabled() {
                    interrupts.assert_nmi();
                }
            }
            if self.line >= V_SIZE_WITH_VBLANK as u16 {                
                self.clear_sprite_0_hit(self.line as usize);
                self.clear_vblank();
                interrupts.deassert_nmi();
                self.line = 0;
                self.background_index = 0;
                self.get_palette(image);
                self.build_sprites(image);
                self.build_dbg_bg(image);
                self.build_dbg_patterns(image);
                image.current_x = self.scroll_x;
                image.current_y = self.scroll_y;
                return true;
            }
        }
        return false;
    }
}
