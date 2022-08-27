#![allow(unused_variables)]
#![allow(unused_variables)]
#![allow(unused_variables)]

use std::process;

use super::Apu;
use super::Cassette;
use super::Ram;
use super::interrupts::Interrupts;
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
pub struct KeyPadRegister {
    pub a: bool,
    pub b: bool,
    pub start: bool,
    pub select: bool,
    pub up: bool,
    pub down: bool,
    pub left: bool,
    pub right: bool,
    addr: u16,
    io_reg: i8,
    pub wait: bool
}

impl KeyPadRegister {
    fn new() -> KeyPadRegister {
        KeyPadRegister {
            a: false,
            b: false,
            start: false,
            select: false,
            up: false,
            down: false,
            left: false,
            right: false,
            addr: 0,
            io_reg: 0,
            wait: false,
        }
    }
    pub fn read(&mut self) -> u8 {
        let mut pad_val: u8 = 0;
        match self.addr {
            0 => pad_val = if self.a {1} else {0},
            1 => pad_val = if self.b {1} else {0},
            2 => pad_val = if self.start {1} else {0},
            3 => pad_val = if self.select {1} else {0},
            4 => pad_val = if self.up {1} else {0},
            5 => pad_val = if self.down {1} else {0},
            6 => pad_val = if self.left {1} else {0},
            7 => pad_val = if self.right {1} else {0},
            _ => (),
        }
        self.addr += 1;
        pad_val
    }
    pub fn write(&mut self, data: u8) {
        if self.io_reg == 0 && data & 0x01 == 1 {
            self.io_reg = 1;
        } else if self.io_reg == 1 && data & 0x01 == 0 {
            self.addr = 0;
            self.io_reg = 0;
        }
    }
}

#[derive(Debug)]
pub struct Mapper {
    mapper: u8,
    bank: u8
}

impl Mapper {
    pub fn new(mapper: u8, bank: u8) -> Mapper {
        Mapper {
            mapper: mapper,
            bank: bank
        }
    }
    pub fn get_mapper(&self) -> u8 {
        self.mapper
    }
    pub fn set_bank(&mut self, bank: u8) {
        self.bank = bank;
    }
    pub fn get_bank(&self) -> u8 {
        self.bank
    }
    pub fn get_char_ram_addr(&self, addr: u16) -> u16 {
        addr + (self.bank as u16) * 0x2000
    }
}

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
    pub index: u64,
    cycle: u64,
    has_branched: bool,
    pub exec_log: Vec<String>,
    nestest_log: Vec<String>,
    reg: Register,
    cas: &'a Cassette,
    wram: &'a mut Ram,
    mapper: Mapper,
    pub keypad1: KeyPadRegister,
    pub keypad2: KeyPadRegister,
    pub mx: u8,
}

impl<'a> Cpu<'a> {
    pub fn new(cas: &'a Cassette, wram: &'a mut Ram) -> Cpu<'a> {
        let nestest_log = "nestest.log";
        let log: String =
            std::fs::read_to_string(nestest_log).unwrap();
        let nestest_log: Vec<String> =
                log.split("\n")
                    .fold(Vec::new(),
                        |mut s,
                        i| {
            // only cpu related log
            s.push(i.to_string());
            s
        });
        Cpu {
            index: 0,
            cycle: 0,
            has_branched: false,
            exec_log: Vec::new(),
            nestest_log: nestest_log,
            reg: Register::new(),
            cas: cas,
            wram: wram,
            mapper: Mapper::new(cas.mapper, 0),
            keypad1: KeyPadRegister::new(),
            keypad2: KeyPadRegister::new(),
            mx: 0,
        }
    }
    pub fn reset(&mut self, ppu: &mut Ppu, apu: &mut Apu, interrupts: &mut Interrupts) {
        self.index = 0;
        self.cycle = 0;
        self.has_branched = false;
        self.reg.reset();
        self.reg.pc = self.wread(ppu, apu, interrupts, 0xFFFC);
        // self.reg.pc = 0xc000;
    }
    fn bread(
        &mut self,
        ppu: &mut Ppu,
        apu: &mut Apu,
        interrupts: &mut Interrupts,
        addr: u16
    ) -> u8 {
        self.read(ppu, apu, interrupts, addr)
    }
    fn wread(
        &mut self,
        ppu: &mut Ppu,
        apu: &mut Apu,
        interrupts: &mut Interrupts,
        addr: u16
    ) -> u16 {
        self.read(ppu, apu, interrupts, addr) as u16 +
            ((self.read(ppu, apu, interrupts, addr +1) as u16) << 8)
    }
    fn read(
        &mut self,
        ppu: &mut Ppu,
        apu: &mut Apu,
        interrupts: &mut Interrupts,
        addr: u16
    ) -> u8 {
        // println!(" read {:#X}", addr);
        match addr {
            0x0000 ..= 0x1FFF => self.wram.read(addr),
            0x2000 ..= 0x3FFF => {
                ppu.read(addr - 0x2000) // ppu read
            },
            0x4015 => apu.read(interrupts, addr), // apu
            0x4016 => self.keypad1.read(), // keypad 1p
            0x4017 => self.keypad2.read(), // keypad 1p
            0x4000 ..= 0x401F => 0, // apu?
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
    fn write(&mut self, ppu: &mut Ppu, apu: &mut Apu, interrupts: &mut Interrupts, addr: u16, data: u8) {
        // println!(" write {:#X} {:#X}", addr, data);
        match addr {
            0x0000 ..= 0x1FFF => self.wram.write(addr, data),
            0x2000 ..= 0x2007 => {
                if addr == 0x2005 {
                    // println!("{:#X} {} {}", addr, data, self.wram.read(0x073F));
                }
                ppu.write(addr - 0x2000, data); // ppu write
            },
            0x4014 => {
                let ram_addr_s: u16 = (data as u16 * SPRITE_RAM_SIZE as u16) as u16;
                ppu.write_sprite_ram_addr(0);
                for i in 0..SPRITE_RAM_SIZE {
                    // self.wram.read(ram_addr_s + i as u16);
                    let data: u8 =
                        self.read(ppu, apu, interrupts, ram_addr_s + i as u16);
                    ppu.write_sprite_ram_data(data);
                }
                // self.cycle += 514; // ?
            }, // dma 
            0x4016 => {
                self.keypad1.write(data); // keypad 1p
            },
            0x4017 => self.keypad2.write(data), // keypad 2p
            0x4000 ..= 0x4020 => {
                apu.write(addr, data);
            }, // apu
            0x6000 ..= 0x7FFF => self.wram.write(addr - 0x8000, data),
            0x8000 ..= 0xFFFF => {
                println!("bank : {}", data);
                self.mapper.set_bank(data);
            },
            _ => panic!("invalid addr {:#X}", addr),
        }
    }
    fn bfetch(&mut self, ppu: &mut Ppu, apu: &mut Apu, interrupts: &mut Interrupts, ) -> u8{
        let data: u8 = self.bread(ppu, apu, interrupts, self.reg.pc);
        self.reg.pc += 1;
        data
    }
    fn wfetch(&mut self, ppu: &mut Ppu, apu: &mut Apu, interrupts: &mut Interrupts, ) -> u16{
        let data: u16 = self.wread(ppu, apu, interrupts, self.reg.pc);
        self.reg.pc += 2;
        data
    }
    fn fetch_op(&mut self, ppu: &mut Ppu, apu: &mut Apu, interrupts: &mut Interrupts, ) -> FetchedOp{
        let pc = self.reg.pc;
        let index: u8 = self.bfetch(ppu, apu, interrupts);
        let op = OP_TABLE.get(&index).unwrap();
        let mut data: u32 = 0;
        let mut add_cycle: u8 = 0;
        match op.mode {
            AddrModes::ACM | AddrModes::IMPL => (),
            AddrModes::IMD | AddrModes::ZPG => {
                data = self.bfetch(ppu, apu, interrupts) as u32
            },
            AddrModes::REL => {
                let addr: u32 = self.bfetch(ppu, apu, interrupts) as u32;
                data = ((addr + self.reg.pc as u32) - 
                    if addr < 0x80 {0} else {0x100}) as u32;
            },
            AddrModes::ZPGX => data =
                ((self.reg.x as u16 + self.bfetch(ppu, apu, interrupts) as u16) & 0xFF) as u32,
            AddrModes::ZPGY => data =
                ((self.reg.y as u16 + self.bfetch(ppu, apu, interrupts) as u16) & 0xFF) as u32,
            AddrModes::ABS => data = self.wfetch(ppu, apu, interrupts) as u32,
            AddrModes::ABSX => {
                let addr: u32 = self.wfetch(ppu, apu, interrupts) as u32;
                data = self.reg.x as u32 + addr;
                add_cycle = if ((data ^ addr) & 0xFF00) > 0 {1} else {0};
            },
            AddrModes::ABSY => {
                let addr: u32 = self.wfetch(ppu, apu, interrupts) as u32;
                data = self.reg.y as u32 + addr;
                add_cycle = if ((data ^ addr) & 0xFF00) > 0 {1} else {0};
            },
            AddrModes::INDX => {
                let baddr: u16 =
                    (self.reg.x as u16 + self.bfetch(ppu, apu, interrupts) as u16) & 0xFF;
                let baddr_: u16 = (baddr + 1) & 0xFF;
                data = (self.bread(ppu, apu, interrupts, baddr) as u16 +
                    ((self.bread(ppu, apu, interrupts, baddr_) as u16) << 8)) as u32;
                },
            AddrModes::INDY => {
                let baddr: u16 = self.bfetch(ppu, apu, interrupts) as u16;
                let baddr_: u16 = (baddr + 1) & 0xFF;
                let data_ = (self.bread(ppu, apu, interrupts, baddr) as u16 +
                    ((self.bread(ppu, apu, interrupts, baddr_) as u16) << 8)) as u32;
                data = data_ as u32 + self.reg.y as u32;
                add_cycle = if ((data ^ data_) & 0xFF00) > 0 {1} else {0};
            },
            AddrModes::ABSIND => {
                let baddr: u16 = self.wfetch(ppu, apu, interrupts);
                let baddr_: u16 = (baddr & 0xFF00) + ((baddr + 1) & 0xFF);
                data = (self.bread(ppu, apu, interrupts, baddr) as u16 +
                    ((self.bread(ppu, apu, interrupts, baddr_) as u16) << 8)) as u32;
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
    fn push(&mut self, ppu: &mut Ppu, apu: &mut Apu, interrupts: &mut Interrupts, data: u8) {
        self.write(ppu, apu, interrupts, self.reg.sp & 0xFF | 0x100, data);
        self.reg.sp -= 1;
    }
    fn push_pc(&mut self, ppu: &mut Ppu, apu: &mut Apu, interrupts: &mut Interrupts) {
        self.push(ppu, apu, interrupts, (self.reg.pc >> 8) as u8);
        self.push(ppu, apu, interrupts, (self.reg.pc & 0xFF) as u8);
    }
    fn push_reg_status(&mut self, ppu: &mut Ppu, apu: &mut Apu, interrupts: &mut Interrupts) {
        self.push(ppu, apu, interrupts, self.reg.p);
    }
    fn pop(&mut self, ppu: &mut Ppu, apu: &mut Apu, interrupts: &mut Interrupts, ) -> u8 {
        self.reg.sp += 1;
        self.bread(ppu, apu, interrupts, self.reg.sp & 0xFF | 0x100)
    }
    fn pop_pc(&mut self, ppu: &mut Ppu, apu: &mut Apu, interrupts: &mut Interrupts) {
        self.reg.pc = self.pop(ppu, apu, interrupts) as u16;
        self.reg.pc += (self.pop(ppu, apu, interrupts) as u16) << 8;
    }
    fn pop_reg_status(&mut self, ppu: &mut Ppu, apu: &mut Apu, interrupts: &mut Interrupts) {
        self.reg.p = self.pop(ppu, apu, interrupts);
    }
    fn exec(&mut self, ppu: &mut Ppu, apu: &mut Apu, interrupts: &mut Interrupts, fop: &mut FetchedOp) {
        let opcode: OpCodes = fop.op.opcode;
        let mode: AddrModes = fop.op.mode;
        let data: u16 = fop.data;
        match opcode {
            // op
            OpCodes::ADC => {
                let data_: u8 = if mode == AddrModes::IMD {
                    data as u8
                } else {
                    self.bread(ppu, apu, interrupts, data) as u8
                };
                let result: u16 = self.reg.a as u16 +
                    data_ as u16 +
                    if self.reg.p & CARRY > 0 {1} else {0};
                self.reg.p = if result > 0xFF {
                    self.reg.p | CARRY
                } else {
                    self.reg.p & !CARRY
                };
                self.reg.p =
                    if ((data_ ^ result as u8) & 0x80) > 0 &&
                        ((self.reg.a ^ result as u8) & 0x80) > 0 {
                    self.reg.p | OVERFLOW
                } else {
                    self.reg.p & !OVERFLOW
                };
                self.set_flag_after_calc(result as u8);
                self.reg.a = result as u8;
            },
            OpCodes::SBC => {
                let data_: u8 = if mode == AddrModes::IMD {
                    data as u8
                } else {
                    self.bread(ppu, apu, interrupts, data) as u8
                };
                let result: i16 = self.reg.a as i16 -
                    data_ as i16 -
                    if self.reg.p & CARRY > 0 {0} else {1};
                self.reg.p = if !(result < 0) {
                    self.reg.p | CARRY
                } else {
                    self.reg.p & !CARRY
                };
                self.reg.p =
                    if ((data_ ^ result as u8) & 0x80 > 0 ||
                        (self.reg.a ^ result as u8) & 0x80 > 0) &&
                        self.reg.p & CARRY > 0 {
                    self.reg.p | OVERFLOW
                } else {
                    self.reg.p & !OVERFLOW
                };
                self.set_flag_after_calc(result as u8);
                self.reg.a = result as u8;             
            },
            // bit op
            OpCodes::AND => {
                let data_: u8 = if mode == AddrModes::IMD {
                    data as u8
                } else {
                    self.bread(ppu, apu, interrupts, data) as u8
                };
                self.reg.a &= data_;
                self.set_flag_after_calc(self.reg.a);
            },
            OpCodes::ORA => {
                let data_: u8 = if mode == AddrModes::IMD {
                    data as u8
                } else {
                    self.bread(ppu, apu, interrupts, data) as u8
                };
                self.reg.a |= data_;
                self.set_flag_after_calc(self.reg.a);
            },
            OpCodes::EOR => {
                let data_: u8 = if mode == AddrModes::IMD {
                    data as u8
                } else {
                    self.bread(ppu, apu, interrupts, data) as u8
                };
                self.reg.a ^= data_;
                self.set_flag_after_calc(self.reg.a);
            },
            // shift/rotation
            OpCodes::ASL => {
                let mut data_: u8 = if mode == AddrModes::ACM {
                    self.reg.a as u8
                } else {
                    self.bread(ppu, apu, interrupts, data) as u8
                };
                self.reg.p = if data_ & 0x80 > 0 {
                    self.reg.p | CARRY
                } else {
                    self.reg.p & !CARRY
                };
                data_ = ((data_ as u16) << 1) as u8;
                if mode == AddrModes::ACM {
                    self.reg.a = data_;
                } else {
                    self.write(ppu, apu, interrupts, data, data_);
                }
                self.set_flag_after_calc(data_);
            },
            OpCodes::LSR => {
                let mut data_: u8 = if mode == AddrModes::ACM {
                    self.reg.a as u8
                } else {
                    self.bread(ppu, apu, interrupts, data) as u8
                };
                self.reg.p = if data_ & 0x01 > 0 {
                    self.reg.p | CARRY
                } else {
                    self.reg.p & !CARRY
                };
                data_ = ((data_ as u16) >> 1) as u8;
                self.reg.p = if data_ == 0 {
                    self.reg.p | ZERO
                } else {
                    self.reg.p & !ZERO
                };
                if mode == AddrModes::ACM {
                    self.reg.a = data_;
                } else {
                    self.write(ppu, apu, interrupts, data, data_);
                }
                self.reg.p &= !NEGATIVE;
            },
            OpCodes::ROL => {
                let mut data_: u8 = if mode == AddrModes::ACM {
                    self.reg.a as u8
                } else {
                    self.bread(ppu, apu, interrupts, data) as u8
                };
                let is_carry: bool = self.reg.p & CARRY > 0;
                self.reg.p = if data_ & 0x80 > 0 {
                    self.reg.p | CARRY
                } else {
                    self.reg.p & !CARRY
                };
                data_ = ((data_ as u16) << 1) as u8;
                data_ = if is_carry {
                    data_ | 0x01 
                } else {
                    data_ & !0x01
                };
                if mode == AddrModes::ACM {
                    self.reg.a = data_;
                } else {
                    self.write(ppu, apu, interrupts, data, data_);
                }
                self.set_flag_after_calc(data_);
            },
            OpCodes::ROR => {
                let mut data_: u8 = if mode == AddrModes::ACM {
                    self.reg.a as u8
                } else {
                    self.bread(ppu, apu, interrupts, data) as u8
                };
                let is_carry: bool = self.reg.p & CARRY > 0;
                self.reg.p = if data_ & 0x01 > 0 {
                    self.reg.p | CARRY
                } else {
                    self.reg.p & !CARRY
                };
                data_ = (data_ >> 1) as u8;
                data_ = if is_carry {
                    data_ | 0x80
                } else {
                    data_ & !0x80
                };
                self.reg.p = if data_ == 0 {
                    self.reg.p | ZERO
                } else {
                    self.reg.p & !ZERO
                };
                if mode == AddrModes::ACM {
                    self.reg.a = data_;
                } else {
                    self.write(ppu, apu, interrupts, data, data_);
                }
                self.set_flag_after_calc(data_);
            },
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
            OpCodes::BIT => {
                let data_ = self.bread(ppu, apu, interrupts, data);
                self.reg.p = if data_ & 0x40 > 0 {
                    self.reg.p | OVERFLOW
                } else {
                    self.reg.p & !OVERFLOW
                };
                self.reg.p = if data_ & 0x80 > 0 {
                    self.reg.p | NEGATIVE
                } else {
                    self.reg.p & !NEGATIVE
                };
                self.reg.p = if (data_ & self.reg.a) == 0 {
                    self.reg.p | ZERO
                } else {
                    self.reg.p & !ZERO
                };
            },
            // jump
            OpCodes::JMP => self.reg.pc = data,
            OpCodes::JSR => {
                let pc: u16 = self.reg.pc - 1;
                self.push(ppu, apu, interrupts, (pc >> 8) as u8 & 0xFF);
                self.push(ppu, apu, interrupts, pc as u8 & 0xFF);
                self.reg.pc = data;
            },
            OpCodes::RTS => {
                self.pop_pc(ppu, apu, interrupts);
                self.reg.pc += 1;
            },
            // interrupt
            OpCodes::BRK => {
                self.reg.pc += 1;
                self.push_pc(ppu, apu, interrupts);
                self.push_reg_status(ppu, apu, interrupts);
                if (self.reg.p & INTERRUPT) == 0 {
                    self.reg.pc = self.wread(ppu, apu, interrupts, 0xFFFE);
                }
                self.reg.p |= INTERRUPT;
                self.reg.pc -= 1;
            },
            OpCodes::RTI => {
                let is_break: bool = self.reg.p & BREAK > 0;
                self.pop_reg_status(ppu, apu, interrupts);
                self.pop_pc(ppu, apu, interrupts);
                self.reg.p |= if is_break {BREAK} else {0};
                self.reg.p |= RESERVED;
            },
            // compare
            OpCodes::CMP => {
                let data_: u8 = if mode == AddrModes::IMD {
                    data as u8
                } else {
                    self.bread(ppu, apu, interrupts, data) as u8
                };
                let comp: i16 = self.reg.a as i16 - data_ as i16;
                self.reg.p = if comp >= 0 {
                    self.reg.p | CARRY
                } else {
                    self.reg.p & !CARRY
                };
                self.set_flag_after_calc(comp as u8);
            },
            OpCodes::CPX => {
                let data_: u8 = if mode == AddrModes::IMD {
                    data as u8
                } else {
                    self.bread(ppu, apu, interrupts, data) as u8
                };
                let comp: i16 = self.reg.x as i16 - data_ as i16;
                self.reg.p = if comp >= 0 {
                    self.reg.p | CARRY
                } else {
                    self.reg.p & !CARRY
                };
                self.set_flag_after_calc(comp as u8);
            },
            OpCodes::CPY => {
                let data_: u8 = if mode == AddrModes::IMD {
                    data as u8
                } else {
                    self.bread(ppu, apu, interrupts, data) as u8
                };
                let comp: i16 = self.reg.y as i16 - data_ as i16;
                self.reg.p = if comp >= 0 {
                    self.reg.p | CARRY
                } else {
                    self.reg.p & !CARRY
                };
                self.set_flag_after_calc(comp as u8);
            },
            // inc/dec
            OpCodes::INC => {
                let data_ :u8 = (self.bread(ppu, apu, interrupts, data) as u16 + 1) as u8;
                self.write(ppu, apu, interrupts, data, data_);
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
                let data_ :u8 = (self.bread(ppu, apu, interrupts, data) as i16 - 1) as u8;
                self.write(ppu, apu, interrupts, data, data_);
                self.set_flag_after_calc(data_);
            },
            OpCodes::DEX => {
                self.reg.x = (self.reg.x as i16 - 1) as u8;
                self.set_flag_after_calc(self.reg.x);
            },
            OpCodes::DEY => {
                self.reg.y = (self.reg.y as i16 - 1) as u8;
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
                let data_: u8 = if mode == AddrModes::IMD {
                    data as u8
                } else {
                    self.bread(ppu, apu, interrupts, data)
                };
                match opcode {
                    OpCodes::LDA => self.reg.a = data_,
                    OpCodes::LDX => self.reg.x = data_,
                    OpCodes::LDY => self.reg.y = data_,
                    _ => panic!("invalid opcode {}", opcode)
                };
                self.set_flag_after_calc(data_);
            },
            // store
            OpCodes::STA => self.write(ppu, apu, interrupts, data, self.reg.a),
            OpCodes::STX => self.write(ppu, apu, interrupts, data, self.reg.x),
            OpCodes::STY => self.write(ppu, apu, interrupts, data, self.reg.y),
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
            OpCodes::PHA => {
                self.push(ppu, apu, interrupts, self.reg.a);
            },
            OpCodes::PHP => {
                let is_break: bool = self.reg.p & BREAK > 0;
                self.reg.p |= BREAK;
                self.push_reg_status(ppu, apu, interrupts);
                self.reg.p = if is_break {
                    self.reg.p | BREAK
                } else {
                    self.reg.p & !BREAK
                };
            },
            OpCodes::PLA => {
                self.reg.a = self.pop(ppu, apu, interrupts);
                self.set_flag_after_calc(self.reg.a);
            },
            OpCodes::PLP => {
                let is_break: bool = self.reg.p & BREAK > 0;
                self.pop_reg_status(ppu, apu, interrupts);
                self.reg.p |= BREAK;
                self.reg.p = if is_break {
                    self.reg.p | BREAK
                } else {
                    self.reg.p & !BREAK
                };
                self.reg.p |= RESERVED;
            },
            // nop
            OpCodes::NOP => (),
            // unofficial
            OpCodes::NOPD => {
                self.reg.pc += 1;
            },
            OpCodes::NOPI => {
                self.reg.pc += 2;
            },
            OpCodes::LAX => {
                self.reg.a = self.bread(ppu, apu, interrupts, data);
                self.reg.x = self.reg.a;
                self.set_flag_after_calc(self.reg.a);
            },
            OpCodes::SAX => {
                self.write(ppu, apu, interrupts, data, self.reg.a & self.reg.x);
            },
            OpCodes::DCP => {
                let data_: u8 = ((self.bread(ppu, apu, interrupts, data) as i16 - 1) & 0xFF) as u8;
                let data__ =
                    (self.reg.a as i16 - data_ as i16) as u8;
                self.set_flag_after_calc(data__);
                self.write(ppu, apu, interrupts, data, data_);
            },
            OpCodes::ISB => {
                let data_: u8 =
                    ((self.bread(ppu, apu, interrupts, data) as u16 + 1) & 0xFF) as u8;
                let data__: u16 =
                    (!data_ & 0xFF) as u16 +
                    self.reg.a  as u16 + 
                    if self.reg.p & CARRY > 0 {1} else {0};
                self.reg.p = 
                    if !((self.reg.a ^ data_) & 0x80 > 0) &&
                        ((self.reg.a ^ data__ as u8) & 0x80) > 0 {
                    self.reg.p | OVERFLOW
                } else {
                    self.reg.p & !OVERFLOW
                };
                self.reg.p = if data__ > 0xFF {
                    self.reg.p | CARRY
                } else {
                    self.reg.p & !CARRY
                };
                self.set_flag_after_calc(data__ as u8);
                self.reg.a = (data__ & 0xFF) as u8;
                self.write(ppu, apu, interrupts, data, data_ as u8);
            },
            OpCodes::SLO => {
                let mut data_: u8 = self.bread(ppu, apu, interrupts, data);
                self.reg.p = if data_ & 0x80 > 0 {
                    self.reg.p | CARRY
                } else {
                    self.reg.p & !CARRY
                };
                data_ = ((data_ as u16) << 1) as u8;
                self.reg.a = (data_ | self.reg.a) & 0xFF;
                self.set_flag_after_calc(self.reg.a);
                self.write(ppu, apu, interrupts, data, data_);
            },
            OpCodes::RLA => {
                let data_: u16 =
                    ((self.bread(ppu, apu, interrupts, data) as u16) << 1) +
                    if self.reg.p & CARRY > 0 {1} else {0};
                self.reg.p = if (data_ as u16) & 0x100 > 0 {
                    self.reg.p | CARRY
                } else {
                    self.reg.p & !CARRY
                };
                self.reg.a = data_ as u8 & self.reg.a;
                self.set_flag_after_calc(self.reg.a);
                self.write(ppu, apu, interrupts, data, data_ as u8);
            },
            OpCodes::SRE => {
                let mut data_: u16 = self.bread(ppu, apu, interrupts, data) as u16;
                self.reg.p = if data_ & 0x01 > 0 {
                    self.reg.p | CARRY
                } else {
                    self.reg.p & !CARRY
                };
                data_ = data_ >> 1;
                self.reg.a = self.reg.a ^ data_ as u8;
                self.set_flag_after_calc(self.reg.a);
                self.write(ppu, apu, interrupts, data, data_ as u8);
            },
            OpCodes::RRA => {
                let mut data_: u16 = self.bread(ppu, apu, interrupts, data) as u16;
                let is_carry: bool = data_ as u8 & CARRY > 0;
                data_ = (data_ >> 1) + if self.reg.p & CARRY > 0 {0x80} else {0x00};
                let data__: u16 = data_ + self.reg.a as u16+
                    if is_carry {1} else {0};
                self.reg.p = if !((self.reg.a ^ data_ as u8) & 0x80 > 0) &&
                        ((self.reg.a ^ data__ as u8) & 0x80) > 0 {
                    self.reg.p | OVERFLOW
                } else {
                    self.reg.p & !OVERFLOW
                };
                self.set_flag_after_calc(data__ as u8);
                self.reg.a = data__ as u8;
                self.reg.p = if data__ > 0xFF {
                    self.reg.p | CARRY
                } else {
                    self.reg.p & !CARRY
                };
                self.write(ppu, apu, interrupts, data, data_ as u8);
            },
            _=> panic!("non implemented opcode {:?}", opcode),
        }
    }
    fn check_nmi(&mut self, ppu: &mut Ppu, apu: &mut Apu, interrupts: &mut Interrupts) {
        if !interrupts.get_nmi_assert(){
            return;
        }
        
        interrupts.deassert_nmi();
        self.reg.p &= !BREAK;
        self.push_pc(ppu, apu, interrupts);
        self.push_reg_status(ppu, apu, interrupts);
        self.reg.p |= INTERRUPT;
        self.reg.pc = self.wread(ppu, apu, interrupts, 0xFFFA);
    }
    fn check_irq(&mut self, ppu: &mut Ppu, apu: &mut Apu, interrupts: &mut Interrupts) {
        if !interrupts.get_irq_assert() {
            return;
        }
        if self.reg.p & INTERRUPT > 0 {
            return;
        }
        interrupts.deassert_irq();
        self.reg.p &= !BREAK;
        self.push_pc(ppu, apu, interrupts);
        self.push_reg_status(ppu, apu, interrupts);
        self.reg.p |= INTERRUPT;
        self.reg.pc = self.wread(ppu, apu, interrupts, 0xFFFE);
    }
    fn show_op(&mut self, pc: u16, fop: &FetchedOp, ppu: &Ppu) {
        let i: usize = self.index as usize;
        let op: OpInfo = fop.op;
        // if op.opcode.to_string() == "JMP" {
        //     return;
        // }
        // if i >= 50 {
        //     println!("nestest check successed!");
        //     process::exit(0);
        // }
        let fmt: String = format!(
            // "{:05} {:04X} {:3} {:4} {:04X} A:{:02X} X:{:02X} Y:{:02X} P:{:02X} SP:{:04X} PPU:{:>3},{:>3} CYC:{}",
            "{:05} {:5} {:3} {:4} {:04X} A:{:02X} X:{:02X} Y:{:02X} P:{:02X} SP:{:04X} PPU:{:>3},{:>3} CYC:{}",
            i + 1, pc, op.opcode.to_string(),
            op.mode.to_string(), fop.data,
            self.reg.a, self.reg.x,
            self.reg.y, self.reg.p,
            self.reg.sp, ppu.line, ppu.cycle,
            self.cycle
        );
        println!("{fmt}");
        self.exec_log.push(fmt);
    }
    fn nestest(&mut self, pc: u16, fop: &FetchedOp) {
        let i: usize = self.index as usize;
        let op: OpInfo = fop.op;
        if i >= 8991 {
            println!("nestest check successed!");
            process::exit(0);
        }
        let fmt = format!("{:04X} {:3} A:{:02X} X:{:02X} Y:{:02X} P:{:02X} SP:{:04X} {}",
            pc, op.opcode.to_string(),
            self.reg.a, self.reg.x,
            self.reg.y, self.reg.p,
            self.reg.sp, self.cycle);
        let exec_op = &fmt[..28];
        let exp_op = &self.nestest_log[i][..28];
        self.exec_log.push(exec_op.to_string());
        if exec_op == exp_op {
            // println!(" # exp :{}", &self.nestest_log[i]);
            // println!(" # exe :{}", &self.exec_log[i]);
        } else {
            if op.opcode == OpCodes::NOP ||
                    op.opcode == OpCodes::NOPD ||
                    op.opcode == OpCodes::NOPI {
                // println!("{:?}", op);
            } else {
                let s: usize = std::cmp::max(0, i as i32 -5) as usize;
                println!(" ### expected ###");
                for j in s..i+1 {
                    println!("{} {}", j + 1, &self.nestest_log[j]);
                }
                println!(" ### executed ###");
                for j in s..i+1 {
                    println!("{} {}", j + 1, &self.exec_log[j]);
                }
                panic!("op compare failed!");
            }
        }
    }
    pub fn run(&mut self, ppu: &mut Ppu, apu: &mut Apu, interrupts: &mut Interrupts) -> u64{
        self.check_nmi(ppu, apu, interrupts);
        self.check_irq(ppu, apu, interrupts);
        let pc = self.reg.pc;
        let mut fetched_op: FetchedOp = 
            self.fetch_op(ppu, apu, interrupts);
        // self.show_op(pc, &fetched_op, &ppu);
        // self.nestest(pc, &fetched_op);
        // if self.wram.read(0x073F) > 0 {
            // println!("{:#X} {:#X} {:#X}",
            //     self.wram.read(0x073F), // horizontal scroll
            //     self.wram.read(0x0723), // scroll rock
            //     self.wram.read(0x06ff) // player_x_scroll
            // );
        //     self.show_op(pc, &fetched_op, &ppu);
        //     self.mx = self.wram.read(0x073F); 
        // }
        self.exec(ppu, apu, interrupts, &mut fetched_op);
        let cycle: u64 = 
            (fetched_op.op.cycle + fetched_op.add_cycle) as u64 +
            if self.has_branched {1} else {0};
        self.cycle += cycle;
        self.index += 1;
        cycle
    }
}