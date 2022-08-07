#![allow(unused_variables)]

use super::Cassette;
use super::Ram;
use super::Context;
use super::optable::{AddrModes, OpCodes, OpInfo, OP_TABLE};
use super::ppu::*;

const CARRY: u8 = 1 << 0;
const ZERO: u8 = 1 << 1;
const INTERRUPT: u8 = 1 << 2;
const DECIMAL: u8 = 1 << 3;
const BREAK: u8 = 1 << 4;
const RESERVED: u8 = 1 << 5;
const OVERFLOW: u8 = 1 << 6;
const NEGATIVE: u8 = 1 << 7;

#[derive(Debug)]
pub struct Register {
    a: u8,
    x: u8,
    y: u8,
    sp: u16,
    pc: u16,
    p: u8, // flags
}

impl Register {
    fn new() -> Register {
        Register {
            a: 0,
            x: 0,
            y: 0,
            sp: 0x01fd,
            pc: 0xc000,
            p: 0x24,
        }
    }
    pub fn reset(&mut self) {
        self.a = 0;
        self.x = 0;
        self.y = 0;
        self.sp = 0x01fd;
        self.pc = 0xc000;
        self.p = 0x24;
    }
}

struct FetchedOp {
    index: u8,
    op: OpInfo,
    data: u16,
    add_cycle: u8
}

#[derive(Debug)]
pub struct Cpu<'a> {
    cycle: u16,
    has_branched: bool,
    reg: Register,
    cas: &'a Cassette,
    wram: &'a mut Ram,
}

impl<'a> Cpu<'a> {
    pub fn new(cas: &'a Cassette, wram: &'a mut Ram) -> Cpu<'a> {
        Cpu {
            cycle: 0,
            has_branched: false,
            reg: Register::new(),
            cas: cas,
            wram: wram,
        }
    }
    pub fn reset(&mut self, ppu:&mut Ppu) {
        self.cycle = 0;
        self.has_branched = false;
        self.reg.reset();
        self.reg.pc = self.wread(ppu, 0xFFFC);
    }
    fn bread(&mut self, ppu: &mut Ppu, addr: u16) -> u8 {
        self.read(ppu, addr)
    }
    fn wread(&mut self, ppu: &mut Ppu, addr: u16) -> u16 {
        self.read(ppu, addr) as u16 + ((self.read(ppu, addr +1) as u16) << 8)
    }
    fn read(&mut self, ppu: &mut Ppu, addr: u16) -> u8 {
        match addr {
            0x0000 ..= 0x1FFF => self.wram.read(addr),
            0x2000 ..= 0x3FFF => ppu.read((addr - 0x2000) % 8), // ppu read
            0x4016 => 0, // joypad 1
            0x4017 => 0, // joypad 1
            0x4000 ..= 0x401F => 0, // apu
            0x6000 ..= 0x7FFF => 0, // extram
            0x8000 ..= 0xBFFF => self.cas.prog_rom_read(addr - 0x8000),
            0xC000 ..= 0xFFFF => {
                if self.cas.prog_size <= 0x4000 {
                    self.cas.prog_rom_read(addr - 0xC000)
                } else {
                    self.cas.prog_rom_read(addr - 0x8000)
                }
            },
            _ => panic!("invalid addr {:#X}", addr)
        }
    }
    fn write(&mut self, ppu: &mut Ppu, addr: u16, data: u8) {
        match addr {
            0x0000 ..= 0x1FFF => self.wram.write(addr, data),
            0x2000 ..= 0x2007 => ppu.write(addr - 0x2000, data), // ppu write
            0x4014 => {
                let ram_addr_s: u16 = (data * SPRITE_RAM_SIZE as u8) as u16;
                ppu.write_sprite_ram_addr(0);
                for i in 0..SPRITE_RAM_SIZE {
                    ppu.write_sprite_ram_data(self.wram.read(ram_addr_s + i as u16));
                }
            }, // dma 
            0x4016 => (), // keypad 1p
            0x4017 => (), // keypad 2p
            0x6000 ..= 0x7FFF => self.wram.write(addr - 0x8000, data),
            _ => panic!("invalid addr {:#X}", addr)
        }
    }
    fn bfetch(&mut self, ppu: &mut Ppu) -> u8{
        let data: u8 = self.bread(ppu, self.reg.pc);
        self.reg.pc += 1;
        data
    }
    fn wfetch(&mut self, ppu: &mut Ppu) -> u16{
        let data: u16 = self.wread(ppu, self.reg.pc);
        self.reg.pc += 2;
        data
    }
    fn fetch_op(&mut self, ppu: &mut Ppu) -> FetchedOp{
        let pc = self.reg.pc;
        let index: u8 = self.bfetch(ppu);
        let op = OP_TABLE.get(&index).unwrap();
        let mut data: u32 = 0;
        let mut add_cycle: u8 = 0;
        match op.mode {
            AddrModes::ACM | AddrModes::IMPL => (),
            AddrModes::IMD | AddrModes::ZPG => data = self.bfetch(ppu) as u32,
            AddrModes::REL => {
                let addr: u32 = self.bfetch(ppu) as u32;
                data = ((addr + self.reg.pc as u32) - if addr < 0x80 {1} else {0x100}) as u32;
            },
            AddrModes::ZPGX => data = ((self.reg.x + self.bfetch(ppu)) & 0xFF) as u32,
            AddrModes::ZPGY => data = ((self.reg.y + self.bfetch(ppu)) & 0xFF) as u32,
            AddrModes::ABS => data = self.wfetch(ppu) as u32,
            AddrModes::ABSX => {
                let addr: u32 = self.wfetch(ppu) as u32;
                data = self.reg.x as u32 + addr;
                add_cycle = if ((data ^ addr) & 0xFF00) > 0 {1} else {0};
            },
            AddrModes::ABSY => {
                let addr: u32 = self.wfetch(ppu) as u32;
                data = self.reg.y as u32 + addr;
                add_cycle = if ((data ^ addr) & 0xFF00) > 0 {1} else {0};
            },
            AddrModes::INDX => {
                let baddr: u16 = (self.reg.x + self.bfetch(ppu)) as u16 & 0xFF;
                let baddr_: u16 = (baddr + 1) & 0xFF;
                data = self.bread(ppu, baddr) as u32 + (self.bread(ppu, baddr_) as u32) << 8;
            },
            AddrModes::INDY => {
                let baddr: u16 = self.bfetch(ppu) as u16;
                let baddr_: u16 = (baddr + 1) & 0xFF;
                data = self.bread(ppu, baddr) as u32 + (self.bread(ppu, baddr_) as u32) << 8;
                let data_: u32 = self.reg.y as u32;
                add_cycle = if ((data ^ data_) & 0xFF00) > 0 {1} else {0};
            },
            AddrModes::ABSIND => {
                let baddr: u16 = self.wfetch(ppu);
                let baddr_: u16 = (baddr & 0xFF00) + (baddr + 1) & 0xFF;
                data = self.bread(ppu, baddr) as u32 + (self.bread(ppu, baddr_) as u32) << 8;
            },
            _=> panic!("invelid mode {:?}", op.mode),
        }
        FetchedOp {
            index: index,
            op: *op,
            data: data as u16,
            add_cycle: add_cycle,
        }
    }
    fn set_flag_after_calc(&mut self, result: u8) {
        if (result & 0x80) > 0 {
            self.reg.p |= NEGATIVE;
        } else {
            self.reg.p &= !NEGATIVE;
        }
        if result == 0 {
            self.reg.p |= ZERO;
        } else {
            self.reg.p &= !ZERO;
        }
    }
    fn branch(&mut self, addr: u16) {
        self.reg.pc = addr;
        self.has_branched = true;
    }
    fn push(&mut self, ppu: &mut Ppu, data: u8) {
        self.write(ppu, self.reg.sp & 0xFF | 0x100, data);
        self.reg.sp -= 1;
    }
    fn push_pc(&mut self, ppu: &mut Ppu) {
        self.push(ppu, (self.reg.pc >> 8) as u8);
        self.push(ppu, (self.reg.pc & 0xFF) as u8);
    }
    fn push_reg_status(&mut self, ppu: &mut Ppu) {
        self.push(ppu, self.reg.p);
    }
    fn pop(&mut self, ppu: &mut Ppu) -> u8 {
        self.reg.sp += 1;
        self.bread(ppu, self.reg.sp & 0xFF | 0x100)
    }
    fn pop_pc(&mut self, ppu: &mut Ppu) {
        self.reg.pc = self.pop(ppu) as u16;
        self.reg.pc += (self.pop(ppu) as u16) << 8;
    }
    fn pop_reg_status(&mut self, ppu: &mut Ppu) {
        self.reg.p = self.pop(ppu);
    }
    fn exec(&mut self, ppu: &mut Ppu, fop: &mut FetchedOp) {
        let opcode: OpCodes = fop.op.opcode;
        let mode: AddrModes = fop.op.mode;
        let data: u16 = fop.data;
        match opcode {
            // op
            // bit op
            // shift/rotation
            // conditional branch
            OpCodes::BCS => {
                if (self.reg.p & CARRY) > 0 {
                    self.branch(data);
                }
            },
            OpCodes::BCC => {
                if (self.reg.p & CARRY) == 0 {
                    self.branch(data);
                }
            },
            OpCodes::BEQ => {
                if (self.reg.p & ZERO) > 0 {
                    self.branch(data);
                }
            },
            OpCodes::BNE => {
                if (self.reg.p & ZERO) == 0 {
                    self.branch(data);
                }
            },
            OpCodes::BMI => {
                if (self.reg.p & NEGATIVE) > 0 {
                    self.branch(data);
                }
            },
            OpCodes::BPL => {
                if (self.reg.p & NEGATIVE) == 0 {
                    self.branch(data);
                }
            },
            OpCodes::BVS => {
                if (self.reg.p & OVERFLOW) > 0 {
                    self.branch(data);
                }
            },
            OpCodes::BVC => {
                if (self.reg.p & OVERFLOW) == 0 {
                    self.branch(data);
                }
            },
            // bit check
            // jump
            OpCodes::JMP => self.reg.pc = data,
            OpCodes::JSR => {
                let pc: u16 = self.reg.pc - 1;
                self.push(ppu, (pc >> 8) as u8 & 0xFF);
                self.push(ppu, pc as u8 & 0xFF);
                self.reg.pc = data;
            },
            OpCodes::RTS => {
                self.pop_pc(ppu);
                self.reg.pc += 1;
            },
            // interrupt
            // comp
            // inc/dec
            OpCodes::INC => {
                let data_ :u8 = (self.bread(ppu, data) as u16 + 1) as u8;
                self.write(ppu, data, data_);
                self.set_flag_after_calc(data_);
            },
            OpCodes::INX => {
                self.reg.x = (self.reg.x as u16 + 1) as u8;
                self.set_flag_after_calc(self.reg.x);
            },
            OpCodes::INY =>  {
                self.reg.y = (self.reg.y as u16 + 1) as u8;
                self.set_flag_after_calc(self.reg.y);
            },
            OpCodes::DEC => {
                let data_ :u8 = self.bread(ppu, data) - 1;
                self.write(ppu, data, data_);
                self.set_flag_after_calc(data_);
            },
            OpCodes::DEX => {
                self.reg.x = self.reg.x - 1;
                self.set_flag_after_calc(self.reg.x);
            },
            OpCodes::DEY => {
                self.reg.y = self.reg.y - 1;
                self.set_flag_after_calc(self.reg.y);
            },
            // flag control
            OpCodes::CLD => self.reg.p &= !DECIMAL,
            OpCodes::CLC => self.reg.p &= !CARRY,
            OpCodes::CLI => self.reg.p &= !INTERRUPT,
            OpCodes::CLV => self.reg.p &= !OVERFLOW,
            OpCodes::SEC => self.reg.p |= CARRY,
            OpCodes::SEI => self.reg.p |= INTERRUPT,
            OpCodes::SED => self.reg.p |= DECIMAL,
            // load
            OpCodes::LDA | OpCodes::LDX | OpCodes::LDY => {
                let data_: u8 = match mode {
                    AddrModes::IMD => data as u8,
                    _ => self.bread(ppu, data)
                };
                match opcode {
                    OpCodes::LDA => self.reg.a = data_,
                    OpCodes::LDX => self.reg.x = data_,
                    OpCodes::LDY => self.reg.y = data_,
                    _ => panic!("invalid opcode {}", opcode)
                }
                self.set_flag_after_calc(data_);
            },
            // store
            OpCodes::STA => self.write(ppu, data, self.reg.a),
            OpCodes::STX => self.write(ppu, data, self.reg.x),
            OpCodes::STY => self.write(ppu, data, self.reg.y),
            // transfer
            OpCodes::TAX => {
                self.reg.x = self.reg.a;
                self.set_flag_after_calc(self.reg.x);
            },
            OpCodes::TAY => {
                self.reg.y = self.reg.a;
                self.set_flag_after_calc(self.reg.y);
            },
            OpCodes::TSX => {
                self.reg.x = self.reg.sp as u8;
                self.set_flag_after_calc(self.reg.x);
            },
            OpCodes::TXA => {
                self.reg.a = self.reg.x;
                self.set_flag_after_calc(self.reg.a);
            },
            OpCodes::TXS => {
                self.reg.sp = self.reg.x as u16 + 0x0100;
            },
            OpCodes::TYA => {
                self.reg.a = self.reg.y;
                self.set_flag_after_calc(self.reg.a);
            },
            // stack
            // nop
            OpCodes::NOP => (),
            // unofficial
            _=> panic!("invelid opcode {:?}", opcode),
        }
    }
    fn show_op(&self, fop: &FetchedOp) {
        let i: u8 = fop.index;
        let op: OpInfo = fop.op;
        println!("{:04} {:#05X} {:3} {:4} {:04X} A:{:02X} X:{:02X} Y:{:02X} P:{:02X} SP:{:04X} {}",
            i, self.reg.pc, op.opcode.to_string(), op.mode.to_string(), fop.data,
            self.reg.a, self.reg.x, self.reg.y, self.reg.p, self.reg.sp, self.cycle);
    }
    pub fn run(&mut self, ppu: &mut Ppu) -> u16 {
        let mut fetched_op: FetchedOp = self.fetch_op(ppu);
        // self.show_op(&fetched_op);
        self.exec(ppu, &mut fetched_op);
        let cycle: u16 = 
            (fetched_op.op.cycle + fetched_op.add_cycle) as u16;
        self.cycle += cycle;
        cycle
    }
}