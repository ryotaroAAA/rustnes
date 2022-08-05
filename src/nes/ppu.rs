#![allow(unused_variables)]

use core::panic;
use std::collections::HashMap;
use std::fmt;
use once_cell::sync::Lazy;
use super::Cassette;
use super::Ram;
use super::Context;

const CARRY: u8 = 1 << 0;
const ZERO: u8 = 1 << 1;
const INTERRUPT: u8 = 1 << 2;
const DECIMAL: u8 = 1 << 3;
const BREAK: u8 = 1 << 4;
const RESERVED: u8 = 1 << 5;
const OVERFLOW: u8 = 1 << 6;
const NEGATIVE: u8 = 1 << 7;

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

const PALETTE_SIZE: u16 = 0x20;
const VRAM_SIZE: u16 = 0x0800;
const TILE_SIZE: u16 = 8;
const H_SPRITE_NUM: u16 = 32;
const V_SPRITE_NUM: u16 = 30;
const V_SIZE_WITH_VBLANK: u16 = 262;
const CYCLE_PER_LINE: u16 = 341;

#[derive(Debug)]
pub struct Sprite {
    x: u8,
    y: u8,
    attr: u8,
    data: [[u8; H_SPRITE_NUM]; V_SPRITE_NUM],
}

impl Sprite {
    fn new() -> Sprite {
        Sprite {
            x: 0,
            y: 0,
            attr: 0,
            data: [[0; H_SPRITE_NUM]; V_SPRITE_NUM]
        }
    }
}

#[derive(Debug)]
pub struct Tile {
    scroll_x: u8,
    scroll_y: u8,
    sprite_id: u8,
    pallete_id: u8,
    sprite: Sprite,
}

impl Tile {
    fn new() -> Tile {
        Tile {
            scroll_x: 0,
            scroll_y: 0,
            sprite_id: 0,
            pallete_id: 0,
            sprite: Sprite::new(),
        }
    }
}

#[derive(Debug)]
struct Image {
    sprite: Vec<Sprite>,
    background: [Tile; H_SPRITE_NUM],
    pallette: [u8; PALETTE_SIZE],
}

#[derive(Debug)]
pub struct Ppu<'a> {
    cycle: u16,
    has_branched: bool,
    reg: Register,
    ctx: &'a mut Context<'a>,
}

impl<'a> Ppu<'a> {
    pub fn new(ctx: &'a mut Context<'a>) -> Cpu<'a> {
        Ppu {
            cycle: 0,
            has_branched: false,
            reg: Register::new(),
            ctx: ctx
        }
    }
    pub fn reset(&mut self) {
        self.cycle = 0;
        self.has_branched = false;
        self.reg.reset();
        self.reg.pc = self.wread(0xFFFC);
    }
    fn read(&mut self, addr: u16) -> u8 {
        match addr {
            0x0000 ..= 0x1FFF => self.ctx.wram.read(addr),
            0x2000 ..= 0x3FFF => 0, // ppu read
            0x4016 => 0, // joypad 1
            0x4017 => 0, // joypad 1
            0x4000 ..= 0x401F => 0, // apu
            0x6000 ..= 0x7FFF => 0, // extram
            0x8000 ..= 0xBFFF => self.ctx.cas.prog_rom_read(addr - 0x8000),
            0xC000 ..= 0xFFFF => {
                if self.ctx.cas.prog_size <= 0x4000 {
                    self.ctx.cas.prog_rom_read(addr - 0xC000)
                } else {
                    self.ctx.cas.prog_rom_read(addr - 0x8000)
                }
            },
            _ => panic!("invalid addr {:#X}", addr)
        }
    }
    fn write(&mut self, addr: u16, data: u8) {
        match addr {
            0x0000 ..= 0x1FFF => self.ctx.wram.write(addr, data),
            0x2000 ..= 0x2007 => (), // ppu write
            0x4014 => (), // dma 
            0x4016 => (), // keypad 1p
            0x4017 => (), // keypad 2p
            0x6000 ..= 0x7FFF => self.ctx.wram.write(addr - 0x8000, data),
            _ => panic!("invalid addr {:#X}", addr)
        }
    }
}