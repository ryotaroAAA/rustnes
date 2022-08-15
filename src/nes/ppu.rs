#![allow(unused_variables)]

use core::panic;
use crate::nes::VRAM_SIZE;

use super::Cassette;
use super::Interrupts;
use super::Ram;

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
    // data: [[u8; H_SPRITE_NUM]; V_SPRITE_NUM],
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
    pub sprite_id: u16,
    pub palette_id: u16,
    pub sprite: Sprite,
}

impl Tile {
    fn new() -> Tile {
        Tile {
            scroll_x: 0,
            scroll_y: 0,
            sprite_id: 0,
            palette_id: 0,
            sprite: Sprite::new(),
        }
    }
}

#[derive(Debug)]
pub struct Image {
    pub sprite: Vec<Sprite>,
    pub background: Vec<Vec<Tile>>,
    pub palette: [u8; PALETTE_SIZE],
}

impl Image {
    pub fn new() -> Image {
        Image {
            sprite: Vec::new(),
            background: vec![vec![Tile::new(); H_SPRITE_NUM]; V_SPRITE_NUM],
            palette: [0; PALETTE_SIZE],
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
            palette[i] = if self.is_sprite_mirror(i as u16) && 
                    self.is_background_mirror(i as u16) {
                self.ram.read(i as u16 - 0x10)
            } else {
                self.ram.read(i as u16)
            };
        }
        palette
    }
    fn write(&mut self, addr: u16, data: u8) {
        let addr_: usize = self.get_palette_addr(addr) as usize;
        self.ram.data[addr_] = data;
    }
    fn is_background_mirror(&self, addr: u16) -> bool{
        match addr {
            0x00 => true,
            0x04 => true,
            0x08 => true,
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
        let mirror_downed: u16 = addr % 0x20;
        if self.is_sprite_mirror(mirror_downed) {
            mirror_downed - 0x10
        } else {
            mirror_downed
        }
    }
}

#[derive(Debug)]
pub struct Ppu<'a> {
    cycle: u64,
    line: u16,
    background_index: u8,
    vram_buf: u16,
    vram_addr: u16,
    vram_offset: u16,
    sprite_ram_addr: u16,
    scroll_x: u8,
    scroll_y: u8,
    is_horizontal_scroll: bool,
    is_lower_vram_addr: bool,
    creg1: u8,
    creg2: u8,
    sreg: u8,
    palette: Palette,
    sprite_ram: Ram,
    char_ram: Ram,
    vram: &'a mut Ram,
    pub image: Image,
}

impl<'a> Ppu<'a> {
    pub fn new(cas: &Cassette, vram: &'a mut Ram) -> Ppu<'a> {
        let mut char_ram = Ram::new(cas.char_size);
        for i in 0..cas.char_size {
            char_ram.data[i] = cas.char_rom[i];
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
            is_horizontal_scroll: false,
            is_lower_vram_addr: false,
            creg1: 0,
            creg2: 0,
            sreg: 0,
            palette: Palette::new(PALETTE_SIZE),
            sprite_ram: Ram::new(SPRITE_RAM_SIZE),
            char_ram: char_ram,
            vram: vram,
            image: Image::new(),
        }
    }
    // Control Register 1, PPU memory increment
    fn get_vram_offset(&mut self) -> u8{
        if self.creg1 & 0x04 > 0 {32} else {1}
    }
    // Control Register 1, Main Screen assignment by name table
    fn get_name_table_id(&mut self) -> u8{
        self.creg1 & 0x03
    }
    // Control Register 1, Assert NMI when VBlank
    fn has_vblank_irq_enabled(&mut self) -> bool{
        self.creg1 & 0x80 > 0
    }
    // Control Register 1, get background pattern table
    fn get_background_table_offset(&mut self) -> u16{
        if self.creg1 & 0x10 > 0 {0x1000} else {0x0000}
    }
    // Control Register 1, get sprite pattern table
    fn get_sprite_table_offset(&mut self) -> u16 {
        if self.creg1 & 0x08 > 0 {0x1000} else {0x0000}
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
    fn set_sprite_hit(&mut self) {
        self.sreg |= 0x40;
    }
    // PPU status register
    fn clear_sprite_hit(&mut self) {
        self.sreg &= 0xBF;
    }
    // PPU status register
    fn set_vblank(&mut self) {
        self.sreg |= 0x80;
    }
    // PPU status register
    fn get_is_vblank(&mut self) -> bool {
        self.sreg & 0x80 > 0
    }
    // PPU status register
    fn clear_vblank(&mut self) {
        self.sreg &= 0x7F;
    }
    fn has_sprite_hit(&mut self) -> bool {
        // main screen y
        let y: u8 = self.sprite_ram.read(0x00);
        y == self.line as u8 &&
            self.get_is_background_enable() &&
            self.get_is_sprite_enable()
    }

    fn get_scroll_tile_x(&mut self) -> u8 {
        self.scroll_x +
            (((self.get_name_table_id() % 2) as u16 * 256) / 8) as u8
    }
    fn get_scroll_tile_y(&mut self) -> u8 { 
        self.scroll_y +
            (((self.get_name_table_id() / 2) as u16 * 240) / 8) as u8
    }
    fn get_tile_y(&mut self) -> u8 {
        (self.line / 8) as u8 + self.get_scroll_tile_y() - 1
    }
    fn get_block_id(&mut self, x: u8, y: u8) -> u8{
        ((x % 4) / 2 + (y % 4) / 2) * 2
    }
    // read from name_table
    fn get_sprite_id(&mut self, x: u8, y: u8, offset: u16) -> u8{
        let tile_num: u16 = y as u16 * 32 + x as u16;
        let sprite_addr: u16 = tile_num + offset;
        self.vram.data[sprite_addr as usize]
    }
    fn get_attribute(&mut self, x: u8, y: u8, offset: u16) -> u8{
        let addr: u16 = x as u16 / 4 + (y as u16/ 4) * 8 + 0x03C0 + offset;
        // TODO
        // self.vram->read(self.mirror_down_sprite_addr(addr))
        self.vram.data[addr as usize]
    }
    fn get_palette(&mut self) {
        self.image.palette = self.palette.read();
    }
    fn calc_vram_addr(&mut self) -> u16{
        if self.vram_addr >= 0x3000 && self.vram_addr < 0x3F00 {
            self.vram_addr - 0x3000
        } else {
            self.vram_addr - 0x2000
        }
    }
    // read by cpu
    fn vram_read(&mut self) -> u8{
        if self.vram_addr >= 0x2000 {
            // name table, attribute table, pallette
            let addr = self.calc_vram_addr();
            self.vram_addr += self.get_vram_offset() as u16;
            // dprint("addr %p %d", addr, buf)
            if addr >= 0x3F00 {
                // palette
                return self.vram.read(addr);
            }
        } else {
            // pattern table from charactor rom
            self.vram_buf = self.char_ram.read(self.vram_addr) as u16;
            self.vram_addr += self.get_vram_offset() as u16;
        }
        return self.vram_buf as u8;
    }
    pub fn read(&mut self, addr: u16) -> u8 {
        /*
        | bit  | description                                 |
        +------+---------------------------------------------+
        | 7    | 1: VBlank clear by reading this register    |
        | 6    | 1: sprite hit                               |
        | 5    | 0: less than 8, 1: 9 or more                |
        | 4-0  | invalid                                     |                                 
        |      | bit4 VRAM write flag [0: success, 1: fail]  |
        */
        match addr {
            0x0002 => {
                // PPUSTATUS
                let status: u8 = self.sreg;
                self.is_horizontal_scroll = true;
                self.clear_vblank();
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
            _ => panic!("invalid addr {:#X}", addr)
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
        if self.is_horizontal_scroll {
            self.is_horizontal_scroll = false;
            self.scroll_x = data & 0xFF;
        } else {
            self.scroll_y = data & 0xFF;
            self.is_horizontal_scroll = true;
        }
    }
    // write by cpu
    fn write_vram_addr(&mut self, data: u8) {
        // println!("{:X} {:X}", self.vram_addr, data);
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
        // println!("{:X} {:X}", self.vram_addr, data);
        if self.vram_addr >= 0x2000 {
            if self.vram_addr >= 0x3F00 && self.vram_addr < 0x4000 {
                // pallette
                // dbg!(self.vram_addr, data);
                self.palette.write(self.vram_addr - 0x3F00, data);
            } else {
                // name table, attr table
                let addr: u16 = self.calc_vram_addr() % VRAM_SIZE as u16;
                // dbg!(self.vram_addr, addr, data);
                // println!(" XXX {:05X} {:05X}", addr, data);
                self.vram.write(addr, data);
            }
        } else {
            // pattern table from charactor rom
            self.char_ram.write(self.vram_addr, data);
        }
        self.vram_addr += self.get_vram_offset() as u16;
    }
    pub fn write(&mut self, addr: u16, data: u8) {
        // println!("{} {}", addr, data);
        match addr {
            0x0000 => self.creg1 = data,
            0x0001 => self.creg2 = data,
            // set sprite ram write addr
            0x0003 => self.write_sprite_ram_addr(data),
            // sprite ram write
            0x0004 => self.write_sprite_ram_data(data),
            // set scroll setting
            0x0005 => self.write_scroll_data(data),
            // set vram write addr (first: high 8bit, second: low 8bit)
            0x0006 => self.write_vram_addr(data),
            // sprite ram write
            0x0007 => self.write_vram_data(data),
            _ => panic!("invalid addr {:#X}", addr)
        }
    }
    
    fn build_sprite_data(&self, sprite_id: u16, offset: u16, sprite: &mut Sprite) {
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
        
        for i in 0..16 {
            let addr: u16 = (sprite_id * 16 + i + offset) as u16;
            let ram: u8 = self.char_ram.read(addr);

            for j in 0..8 {
                if ram & (0x80 >> j) > 0 {
                    sprite.data[(i % 8) as usize][j] += 0x01 << (i/8);
                }
            }
        }
        // for a in &sprite.data{
        //     for b in a.iter(){
        //         let val = if *b > 0 {1} else {0};
        //         print!("{:?}", val);
        //     }
        //     print!("\n");
        // }
        // sprite
    }
    fn build_objects(&mut self) {
        // see https:#wiki.nesdev.com/w/index.php/PPU_OAM
        for i in 0..self.sprite_ram_addr/4 {
            let j: u16 = 4 * i as u16;
            dbg!(i, j);
            let mut sprite: Sprite = Sprite::new();
            sprite.y = self.sprite_ram.read(j);
            let sprite_id: u16 = self.sprite_ram.read(j + 1) as u16;
            sprite.attr = self.sprite_ram.read(j + 2);
            sprite.x = self.sprite_ram.read(j + 3);
            let sprite_table_offset: u16 = self.get_sprite_table_offset();
            self.build_sprite_data(sprite_id,
                sprite_table_offset, &mut sprite);
            self.image.sprite.push(sprite);
        }
    }
    fn build_sprite_for_tile(&mut self, sprite_id: u16, offset: u16, x: u8, y: u8) {
        let background = &mut self.image.background;
        // self.build_sprite_data(sprite_id, offset,
        //     &mut background[i as usize][j as usize].sprite);
        for i in 0..8 {
            for j in 0..8 {
                background[x as usize][y as usize]
                    .sprite.data[i][j] = 0;
            }
        }
        for i in 0..16 {
            let addr: u16 = (sprite_id * 16 + i + offset) as u16;
            let ram: u8 = self.char_ram.read(addr);

            for j in 0..8 {
                if ram & (0x80 >> j) > 0 {
                    background[x as usize][y as usize].sprite.data[(i % 8) as usize][j] +=
                        0x01 << (i/8);
                }
            }
        }
        // if sprite_id > 0 {
        //     println!("{} x:{}, y:{}", sprite_id, x, y);
        //     for a in &background[x as usize][y as usize].sprite.data {
        //         for b in a.iter(){
        //             let val = if *b > 0 {1} else {0};
        //             print!("{:?}", val);
        //         }
        //         print!("\n");
        //     }
        // }
    }
    // the element of background
    fn build_tile(&mut self, x: u8, y: u8, offset: u16, i: u8, j: u8) {
        let block_id: u8 = self.get_block_id(x, y);
        let sprite_id: u16 = self.get_sprite_id(x, y, offset) as u16;
        let attr: u16 = self.get_attribute(x, y, offset) as u16;
        let background_table_offset: u16 = self.get_background_table_offset();
        // let mut background = &mut self.image.background;
        self.image.background[i as usize][j as usize].sprite_id = sprite_id;
        self.image.background[i as usize][j as usize].palette_id =
            (attr >> (block_id * 2)) as u16 & 0x03;
        // self.build_sprite_data(sprite_id, background_table_offset,
        //     &mut self.image.background[i as usize][j as usize].sprite);
        self.build_sprite_for_tile(sprite_id, background_table_offset, i, j);
        self.image.background[i as usize][j as usize].scroll_x = self.scroll_x;
        self.image.background[i as usize][j as usize].scroll_y = self.scroll_y;
    }
    // draw every 8 line
    fn build_background(&mut self) {
        let mod_y: u8 = self.get_tile_y() % V_SPRITE_NUM as u8;
        let table_id_offset: u8 =
            if (self.get_tile_y() / V_SPRITE_NUM as u8) % 2 > 0 {2} else {0};

        let i : u8 = self.background_index;
        for x in 0..H_SPRITE_NUM as u8 {
            let mod_x: u8 = x % H_SPRITE_NUM as u8;
            let name_table_id: u8 = (x / H_SPRITE_NUM as u8) % 2 + table_id_offset;
            let offset_addr_by_name_table: u16 = name_table_id as u16 * 0x0400;
            self.build_tile(mod_x, mod_y, offset_addr_by_name_table, i, x);
        }
        self.background_index += 1;
    }
    
    pub fn run(&mut self, cycle: u64, inter: &mut Interrupts) -> bool{
        self.cycle += 3 * cycle;
        
        if self.line == 0 {
            self.image.sprite.resize(0, Sprite::new());
        }
        if self.cycle >= CYCLE_PER_LINE as u64 {
            self.cycle -= CYCLE_PER_LINE as u64;
            self.line += 1;

            if self.has_sprite_hit() {
                self.set_sprite_hit();
            }
            if self.line <= V_SIZE as u16 &&
                    !(self.line % TILE_SIZE as u16 > 0) &&
                    (self.line % TILE_SIZE as u16 == 0) {
                self.build_background();
            }
            
            if self.line == V_SIZE as u16 + 1 {
                self.set_vblank();
                inter.deassert_nmi();
                if self.has_vblank_irq_enabled() {
                    inter.assert_nmi();
                }
            }
            if self.line == V_SIZE_WITH_VBLANK as u16 {
                self.build_objects();
                self.clear_vblank();
                self.clear_sprite_hit();
                self.line = 0;
                self.background_index = 0;

                self.get_palette();
                // self.interrupts.deassert_nmi();
                // return self.image
                return true;
            }
        }
        return false;
    }
}
