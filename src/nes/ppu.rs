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

pub static OP_TABLE: Lazy<HashMap<u8, OpInfo>> = Lazy::new(|| {
    let mut m:HashMap<u8, OpInfo> = HashMap::new();
    m.insert(0x00, OpInfo {cycle:7, mode: AddrModes::IMPL, opcode: OpCodes::BRK});
    m.insert(0x01, OpInfo {cycle:6, mode: AddrModes::INDX, opcode: OpCodes::ORA});
    m.insert(0x02, OpInfo {cycle:2, mode: AddrModes::IMPL, opcode: OpCodes::NOP});
    m.insert(0x03, OpInfo {cycle:8, mode: AddrModes::INDX, opcode: OpCodes::SLO});
    m.insert(0x04, OpInfo {cycle:3, mode: AddrModes::IMPL, opcode: OpCodes::NOPD});
    m.insert(0x05, OpInfo {cycle:3, mode: AddrModes::ZPG, opcode: OpCodes::ORA});
    m.insert(0x06, OpInfo {cycle:5, mode: AddrModes::ZPG, opcode: OpCodes::ASL});
    m.insert(0x07, OpInfo {cycle:5, mode: AddrModes::ZPG, opcode: OpCodes::SLO});
    m.insert(0x08, OpInfo {cycle:3, mode: AddrModes::IMPL, opcode: OpCodes::PHP});
    m.insert(0x09, OpInfo {cycle:2, mode: AddrModes::IMD, opcode: OpCodes::ORA});
    m.insert(0x0A, OpInfo {cycle:2, mode: AddrModes::ACM, opcode: OpCodes::ASL});
    m.insert(0x0C, OpInfo {cycle:4, mode: AddrModes::IMPL, opcode: OpCodes::NOPI});
    m.insert(0x0D, OpInfo {cycle:4, mode: AddrModes::ABS, opcode: OpCodes::ORA});
    m.insert(0x0E, OpInfo {cycle:6, mode: AddrModes::ABS, opcode: OpCodes::ASL});
    m.insert(0x0F, OpInfo {cycle:6, mode: AddrModes::ABS, opcode: OpCodes::SLO});
    m.insert(0x10, OpInfo {cycle:2, mode: AddrModes::REL, opcode: OpCodes::BPL});
    m.insert(0x11, OpInfo {cycle:5, mode: AddrModes::INDY, opcode: OpCodes::ORA});
    m.insert(0x12, OpInfo {cycle:2, mode: AddrModes::IMPL, opcode: OpCodes::NOP});
    m.insert(0x13, OpInfo {cycle:8, mode: AddrModes::INDY, opcode: OpCodes::SLO});
    m.insert(0x14, OpInfo {cycle:4, mode: AddrModes::IMPL, opcode: OpCodes::NOPD});
    m.insert(0x15, OpInfo {cycle:4, mode: AddrModes::ZPGX, opcode: OpCodes::ORA});
    m.insert(0x16, OpInfo {cycle:6, mode: AddrModes::ZPGX, opcode: OpCodes::ASL});
    m.insert(0x17, OpInfo {cycle:6, mode: AddrModes::ZPGX, opcode: OpCodes::SLO});
    m.insert(0x18, OpInfo {cycle:2, mode: AddrModes::IMPL, opcode: OpCodes::CLC});
    m.insert(0x19, OpInfo {cycle:4, mode: AddrModes::ABSY, opcode: OpCodes::ORA});
    m.insert(0x1A, OpInfo {cycle:2, mode: AddrModes::IMPL, opcode: OpCodes::NOP});
    m.insert(0x1B, OpInfo {cycle:7, mode: AddrModes::ABSY, opcode: OpCodes::SLO});
    m.insert(0x1C, OpInfo {cycle:4, mode: AddrModes::IMPL, opcode: OpCodes::NOPI});
    m.insert(0x1D, OpInfo {cycle:4, mode: AddrModes::ABSX, opcode: OpCodes::ORA});
    m.insert(0x1E, OpInfo {cycle:6, mode: AddrModes::ABSX, opcode: OpCodes::ASL});
    m.insert(0x1F, OpInfo {cycle:7, mode: AddrModes::ABSX, opcode: OpCodes::SLO});
    m.insert(0x20, OpInfo {cycle:6, mode: AddrModes::ABS, opcode: OpCodes::JSR});
    m.insert(0x21, OpInfo {cycle:6, mode: AddrModes::INDX, opcode: OpCodes::AND});
    m.insert(0x22, OpInfo {cycle:2, mode: AddrModes::IMPL, opcode: OpCodes::NOP});
    m.insert(0x23, OpInfo {cycle:8, mode: AddrModes::INDX, opcode: OpCodes::RLA});
    m.insert(0x24, OpInfo {cycle:3, mode: AddrModes::ZPG, opcode: OpCodes::BIT});
    m.insert(0x25, OpInfo {cycle:3, mode: AddrModes::ZPG, opcode: OpCodes::AND});
    m.insert(0x26, OpInfo {cycle:5, mode: AddrModes::ZPG, opcode: OpCodes::ROL});
    m.insert(0x27, OpInfo {cycle:5, mode: AddrModes::ZPG, opcode: OpCodes::RLA});
    m.insert(0x28, OpInfo {cycle:4, mode: AddrModes::IMPL, opcode: OpCodes::PLP});
    m.insert(0x29, OpInfo {cycle:2, mode: AddrModes::IMD, opcode: OpCodes::AND});
    m.insert(0x2A, OpInfo {cycle:2, mode: AddrModes::ACM, opcode: OpCodes::ROL});
    m.insert(0x2C, OpInfo {cycle:4, mode: AddrModes::ABS, opcode: OpCodes::BIT});
    m.insert(0x2D, OpInfo {cycle:4, mode: AddrModes::ABS, opcode: OpCodes::AND});
    m.insert(0x2E, OpInfo {cycle:6, mode: AddrModes::ABS, opcode: OpCodes::ROL});
    m.insert(0x2F, OpInfo {cycle:6, mode: AddrModes::ABS, opcode: OpCodes::RLA});
    m.insert(0x30, OpInfo {cycle:2, mode: AddrModes::REL, opcode: OpCodes::BMI});
    m.insert(0x31, OpInfo {cycle:5, mode: AddrModes::INDY, opcode: OpCodes::AND});
    m.insert(0x32, OpInfo {cycle:2, mode: AddrModes::IMPL, opcode: OpCodes::NOP});
    m.insert(0x33, OpInfo {cycle:8, mode: AddrModes::INDY, opcode: OpCodes::RLA});
    m.insert(0x34, OpInfo {cycle:4, mode: AddrModes::IMPL, opcode: OpCodes::NOPD});
    m.insert(0x35, OpInfo {cycle:4, mode: AddrModes::ZPGX, opcode: OpCodes::AND});
    m.insert(0x36, OpInfo {cycle:6, mode: AddrModes::ZPGX, opcode: OpCodes::ROL});
    m.insert(0x37, OpInfo {cycle:6, mode: AddrModes::ZPGX, opcode: OpCodes::RLA});
    m.insert(0x38, OpInfo {cycle:2, mode: AddrModes::IMPL, opcode: OpCodes::SEC});
    m.insert(0x39, OpInfo {cycle:4, mode: AddrModes::ABSY, opcode: OpCodes::AND});
    m.insert(0x3A, OpInfo {cycle:2, mode: AddrModes::IMPL, opcode: OpCodes::NOP});
    m.insert(0x3B, OpInfo {cycle:7, mode: AddrModes::ABSY, opcode: OpCodes::RLA});
    m.insert(0x3C, OpInfo {cycle:4, mode: AddrModes::IMPL, opcode: OpCodes::NOPI});
    m.insert(0x3D, OpInfo {cycle:4, mode: AddrModes::ABSX, opcode: OpCodes::AND});
    m.insert(0x3E, OpInfo {cycle:6, mode: AddrModes::ABSX, opcode: OpCodes::ROL});
    m.insert(0x3F, OpInfo {cycle:7, mode: AddrModes::ABSX, opcode: OpCodes::RLA});
    m.insert(0x40, OpInfo {cycle:6, mode: AddrModes::IMPL, opcode: OpCodes::RTI});
    m.insert(0x41, OpInfo {cycle:6, mode: AddrModes::INDX, opcode: OpCodes::EOR});
    m.insert(0x42, OpInfo {cycle:2, mode: AddrModes::IMPL, opcode: OpCodes::NOP});
    m.insert(0x43, OpInfo {cycle:8, mode: AddrModes::INDX, opcode: OpCodes::SRE});
    m.insert(0x44, OpInfo {cycle:3, mode: AddrModes::IMPL, opcode: OpCodes::NOPD});
    m.insert(0x45, OpInfo {cycle:3, mode: AddrModes::ZPG, opcode: OpCodes::EOR});
    m.insert(0x46, OpInfo {cycle:5, mode: AddrModes::ZPG, opcode: OpCodes::LSR});
    m.insert(0x47, OpInfo {cycle:5, mode: AddrModes::ZPG, opcode: OpCodes::SRE});
    m.insert(0x48, OpInfo {cycle:3, mode: AddrModes::IMPL, opcode: OpCodes::PHA});
    m.insert(0x49, OpInfo {cycle:2, mode: AddrModes::IMD, opcode: OpCodes::EOR});
    m.insert(0x4A, OpInfo {cycle:2, mode: AddrModes::ACM, opcode: OpCodes::LSR});
    m.insert(0x4C, OpInfo {cycle:3, mode: AddrModes::ABS, opcode: OpCodes::JMP});
    m.insert(0x4D, OpInfo {cycle:4, mode: AddrModes::ABS, opcode: OpCodes::EOR});
    m.insert(0x4E, OpInfo {cycle:6, mode: AddrModes::ABS, opcode: OpCodes::LSR});
    m.insert(0x4F, OpInfo {cycle:6, mode: AddrModes::ABS, opcode: OpCodes::SRE});
    m.insert(0x50, OpInfo {cycle:2, mode: AddrModes::REL, opcode: OpCodes::BVC});
    m.insert(0x51, OpInfo {cycle:5, mode: AddrModes::INDY, opcode: OpCodes::EOR});
    m.insert(0x52, OpInfo {cycle:2, mode: AddrModes::IMPL, opcode: OpCodes::NOP});
    m.insert(0x53, OpInfo {cycle:8, mode: AddrModes::INDY, opcode: OpCodes::SRE});
    m.insert(0x54, OpInfo {cycle:4, mode: AddrModes::IMPL, opcode: OpCodes::NOPD});
    m.insert(0x55, OpInfo {cycle:4, mode: AddrModes::ZPGX, opcode: OpCodes::EOR});
    m.insert(0x56, OpInfo {cycle:6, mode: AddrModes::ZPGX, opcode: OpCodes::LSR});
    m.insert(0x57, OpInfo {cycle:6, mode: AddrModes::ZPGX, opcode: OpCodes::SRE});
    m.insert(0x58, OpInfo {cycle:2, mode: AddrModes::IMPL, opcode: OpCodes::CLI});
    m.insert(0x59, OpInfo {cycle:4, mode: AddrModes::ABSY, opcode: OpCodes::EOR});
    m.insert(0x5A, OpInfo {cycle:2, mode: AddrModes::IMPL, opcode: OpCodes::NOP});
    m.insert(0x5B, OpInfo {cycle:7, mode: AddrModes::ABSY, opcode: OpCodes::SRE});
    m.insert(0x5C, OpInfo {cycle:4, mode: AddrModes::IMPL, opcode: OpCodes::NOPI});
    m.insert(0x5D, OpInfo {cycle:4, mode: AddrModes::ABSX, opcode: OpCodes::EOR});
    m.insert(0x5E, OpInfo {cycle:6, mode: AddrModes::ABSX, opcode: OpCodes::LSR});
    m.insert(0x5F, OpInfo {cycle:7, mode: AddrModes::ABSX, opcode: OpCodes::SRE});
    m.insert(0x60, OpInfo {cycle:6, mode: AddrModes::IMPL, opcode: OpCodes::RTS});
    m.insert(0x61, OpInfo {cycle:6, mode: AddrModes::INDX, opcode: OpCodes::ADC});
    m.insert(0x62, OpInfo {cycle:2, mode: AddrModes::IMPL, opcode: OpCodes::NOP});
    m.insert(0x63, OpInfo {cycle:8, mode: AddrModes::INDX, opcode: OpCodes::RRA});
    m.insert(0x64, OpInfo {cycle:3, mode: AddrModes::IMPL, opcode: OpCodes::NOPD});
    m.insert(0x65, OpInfo {cycle:3, mode: AddrModes::ZPG, opcode: OpCodes::ADC});
    m.insert(0x66, OpInfo {cycle:5, mode: AddrModes::ZPG, opcode: OpCodes::ROR});
    m.insert(0x67, OpInfo {cycle:5, mode: AddrModes::ZPG, opcode: OpCodes::RRA});
    m.insert(0x68, OpInfo {cycle:4, mode: AddrModes::IMPL, opcode: OpCodes::PLA});
    m.insert(0x69, OpInfo {cycle:2, mode: AddrModes::IMD, opcode: OpCodes::ADC});
    m.insert(0x6A, OpInfo {cycle:2, mode: AddrModes::ACM, opcode: OpCodes::ROR});
    m.insert(0x6C, OpInfo {cycle:5, mode: AddrModes::ABSIND, opcode: OpCodes::JMP});
    m.insert(0x6D, OpInfo {cycle:4, mode: AddrModes::ABS, opcode: OpCodes::ADC});
    m.insert(0x6E, OpInfo {cycle:6, mode: AddrModes::ABS, opcode: OpCodes::ROR});
    m.insert(0x6F, OpInfo {cycle:6, mode: AddrModes::ABS, opcode: OpCodes::RRA});
    m.insert(0x70, OpInfo {cycle:2, mode: AddrModes::REL, opcode: OpCodes::BVS});
    m.insert(0x71, OpInfo {cycle:5, mode: AddrModes::INDY, opcode: OpCodes::ADC});
    m.insert(0x72, OpInfo {cycle:2, mode: AddrModes::IMPL, opcode: OpCodes::NOP});
    m.insert(0x73, OpInfo {cycle:8, mode: AddrModes::INDY, opcode: OpCodes::RRA});
    m.insert(0x74, OpInfo {cycle:4, mode: AddrModes::IMPL, opcode: OpCodes::NOPD});
    m.insert(0x75, OpInfo {cycle:4, mode: AddrModes::ZPGX, opcode: OpCodes::ADC});
    m.insert(0x76, OpInfo {cycle:6, mode: AddrModes::ZPGX, opcode: OpCodes::ROR});
    m.insert(0x77, OpInfo {cycle:6, mode: AddrModes::ZPGX, opcode: OpCodes::RRA});
    m.insert(0x78, OpInfo {cycle:2, mode: AddrModes::IMPL, opcode: OpCodes::SEI});
    m.insert(0x79, OpInfo {cycle:4, mode: AddrModes::ABSY, opcode: OpCodes::ADC});
    m.insert(0x7A, OpInfo {cycle:2, mode: AddrModes::IMPL, opcode: OpCodes::NOP});
    m.insert(0x7B, OpInfo {cycle:7, mode: AddrModes::ABSY, opcode: OpCodes::RRA});
    m.insert(0x7C, OpInfo {cycle:4, mode: AddrModes::IMPL, opcode: OpCodes::NOPI});
    m.insert(0x7D, OpInfo {cycle:4, mode: AddrModes::ABSX, opcode: OpCodes::ADC});
    m.insert(0x7E, OpInfo {cycle:6, mode: AddrModes::ABSX, opcode: OpCodes::ROR});
    m.insert(0x7F, OpInfo {cycle:7, mode: AddrModes::ABSX, opcode: OpCodes::RRA});
    m.insert(0x80, OpInfo {cycle:2, mode: AddrModes::IMPL, opcode: OpCodes::NOPD});
    m.insert(0x81, OpInfo {cycle:6, mode: AddrModes::INDX, opcode: OpCodes::STA});
    m.insert(0x82, OpInfo {cycle:2, mode: AddrModes::IMPL, opcode: OpCodes::NOPD});
    m.insert(0x83, OpInfo {cycle:6, mode: AddrModes::INDX, opcode: OpCodes::SAX});
    m.insert(0x84, OpInfo {cycle:3, mode: AddrModes::ZPG, opcode: OpCodes::STY});
    m.insert(0x85, OpInfo {cycle:3, mode: AddrModes::ZPG, opcode: OpCodes::STA});
    m.insert(0x86, OpInfo {cycle:3, mode: AddrModes::ZPG, opcode: OpCodes::STX});
    m.insert(0x87, OpInfo {cycle:3, mode: AddrModes::ZPG, opcode: OpCodes::SAX});
    m.insert(0x88, OpInfo {cycle:2, mode: AddrModes::IMPL, opcode: OpCodes::DEY});
    m.insert(0x89, OpInfo {cycle:2, mode: AddrModes::IMPL, opcode: OpCodes::NOPD});
    m.insert(0x8A, OpInfo {cycle:2, mode: AddrModes::IMPL, opcode: OpCodes::TXA});
    m.insert(0x8C, OpInfo {cycle:4, mode: AddrModes::ABS, opcode: OpCodes::STY});
    m.insert(0x8D, OpInfo {cycle:4, mode: AddrModes::ABS, opcode: OpCodes::STA});
    m.insert(0x8E, OpInfo {cycle:4, mode: AddrModes::ABS, opcode: OpCodes::STX});
    m.insert(0x8F, OpInfo {cycle:4, mode: AddrModes::ABS, opcode: OpCodes::SAX});
    m.insert(0x90, OpInfo {cycle:2, mode: AddrModes::REL, opcode: OpCodes::BCC});
    m.insert(0x91, OpInfo {cycle:6, mode: AddrModes::INDY, opcode: OpCodes::STA});
    m.insert(0x92, OpInfo {cycle:2, mode: AddrModes::IMPL, opcode: OpCodes::NOP});
    m.insert(0x94, OpInfo {cycle:4, mode: AddrModes::ZPGX, opcode: OpCodes::STY});
    m.insert(0x95, OpInfo {cycle:4, mode: AddrModes::ZPGX, opcode: OpCodes::STA});
    m.insert(0x96, OpInfo {cycle:4, mode: AddrModes::ZPGY, opcode: OpCodes::STX});
    m.insert(0x97, OpInfo {cycle:4, mode: AddrModes::ZPGY, opcode: OpCodes::SAX});
    m.insert(0x98, OpInfo {cycle:2, mode: AddrModes::IMPL, opcode: OpCodes::TYA});
    m.insert(0x99, OpInfo {cycle:5, mode: AddrModes::ABSY, opcode: OpCodes::STA});
    m.insert(0x9A, OpInfo {cycle:2, mode: AddrModes::IMPL, opcode: OpCodes::TXS});
    m.insert(0x9D, OpInfo {cycle:4, mode: AddrModes::ABSX, opcode: OpCodes::STA});
    m.insert(0xA0, OpInfo {cycle:2, mode: AddrModes::IMD, opcode: OpCodes::LDY});
    m.insert(0xA1, OpInfo {cycle:6, mode: AddrModes::INDX, opcode: OpCodes::LDA});
    m.insert(0xA2, OpInfo {cycle:2, mode: AddrModes::IMD, opcode: OpCodes::LDX});
    m.insert(0xA3, OpInfo {cycle:6, mode: AddrModes::INDX, opcode: OpCodes::LAX});
    m.insert(0xA4, OpInfo {cycle:3, mode: AddrModes::ZPG, opcode: OpCodes::LDY});
    m.insert(0xA5, OpInfo {cycle:3, mode: AddrModes::ZPG, opcode: OpCodes::LDA});
    m.insert(0xA6, OpInfo {cycle:3, mode: AddrModes::ZPG, opcode: OpCodes::LDX});
    m.insert(0xA7, OpInfo {cycle:3, mode: AddrModes::ZPG, opcode: OpCodes::LAX});
    m.insert(0xA8, OpInfo {cycle:2, mode: AddrModes::IMPL, opcode: OpCodes::TAY});
    m.insert(0xA9, OpInfo {cycle:2, mode: AddrModes::IMD, opcode: OpCodes::LDA});
    m.insert(0xAA, OpInfo {cycle:2, mode: AddrModes::IMPL, opcode: OpCodes::TAX});
    m.insert(0xAC, OpInfo {cycle:4, mode: AddrModes::ABS, opcode: OpCodes::LDY});
    m.insert(0xAD, OpInfo {cycle:4, mode: AddrModes::ABS, opcode: OpCodes::LDA});
    m.insert(0xAE, OpInfo {cycle:4, mode: AddrModes::ABS, opcode: OpCodes::LDX});
    m.insert(0xAF, OpInfo {cycle:4, mode: AddrModes::ABS, opcode: OpCodes::LAX});
    m.insert(0xB0, OpInfo {cycle:2, mode: AddrModes::REL, opcode: OpCodes::BCS});
    m.insert(0xB1, OpInfo {cycle:5, mode: AddrModes::INDY, opcode: OpCodes::LDA});
    m.insert(0xB2, OpInfo {cycle:2, mode: AddrModes::IMPL, opcode: OpCodes::NOP});
    m.insert(0xB3, OpInfo {cycle:5, mode: AddrModes::INDY, opcode: OpCodes::LAX});
    m.insert(0xB4, OpInfo {cycle:4, mode: AddrModes::ZPGX, opcode: OpCodes::LDY});
    m.insert(0xB5, OpInfo {cycle:4, mode: AddrModes::ZPGX, opcode: OpCodes::LDA});
    m.insert(0xB6, OpInfo {cycle:4, mode: AddrModes::ZPGY, opcode: OpCodes::LDX});
    m.insert(0xB7, OpInfo {cycle:4, mode: AddrModes::ZPGY, opcode: OpCodes::LAX});
    m.insert(0xB8, OpInfo {cycle:2, mode: AddrModes::IMPL, opcode: OpCodes::CLV});
    m.insert(0xB9, OpInfo {cycle:4, mode: AddrModes::ABSY, opcode: OpCodes::LDA});
    m.insert(0xBA, OpInfo {cycle:2, mode: AddrModes::IMPL, opcode: OpCodes::TSX});
    m.insert(0xBC, OpInfo {cycle:4, mode: AddrModes::ABSX, opcode: OpCodes::LDY});
    m.insert(0xBD, OpInfo {cycle:4, mode: AddrModes::ABSX, opcode: OpCodes::LDA});
    m.insert(0xBE, OpInfo {cycle:4, mode: AddrModes::ABSY, opcode: OpCodes::LDX});
    m.insert(0xBF, OpInfo {cycle:4, mode: AddrModes::ABSY, opcode: OpCodes::LAX});
    m.insert(0xC0, OpInfo {cycle:2, mode: AddrModes::IMD, opcode: OpCodes::CPY});
    m.insert(0xC1, OpInfo {cycle:6, mode: AddrModes::INDX, opcode: OpCodes::CMP});
    m.insert(0xC2, OpInfo {cycle:2, mode: AddrModes::IMPL, opcode: OpCodes::NOPD});
    m.insert(0xC3, OpInfo {cycle:8, mode: AddrModes::INDX, opcode: OpCodes::DCP});
    m.insert(0xC4, OpInfo {cycle:3, mode: AddrModes::ZPG, opcode: OpCodes::CPY});
    m.insert(0xC5, OpInfo {cycle:3, mode: AddrModes::ZPG, opcode: OpCodes::CMP});
    m.insert(0xC6, OpInfo {cycle:5, mode: AddrModes::ZPG, opcode: OpCodes::DEC});
    m.insert(0xC7, OpInfo {cycle:5, mode: AddrModes::ZPG, opcode: OpCodes::DCP});
    m.insert(0xC8, OpInfo {cycle:2, mode: AddrModes::IMPL, opcode: OpCodes::INY});
    m.insert(0xC9, OpInfo {cycle:2, mode: AddrModes::IMD, opcode: OpCodes::CMP});
    m.insert(0xCA, OpInfo {cycle:2, mode: AddrModes::IMPL, opcode: OpCodes::DEX});
    m.insert(0xCC, OpInfo {cycle:4, mode: AddrModes::ABS, opcode: OpCodes::CPY});
    m.insert(0xCD, OpInfo {cycle:4, mode: AddrModes::ABS, opcode: OpCodes::CMP});
    m.insert(0xCE, OpInfo {cycle:6, mode: AddrModes::ABS, opcode: OpCodes::DEC});
    m.insert(0xCF, OpInfo {cycle:6, mode: AddrModes::ABS, opcode: OpCodes::DCP});
    m.insert(0xD0, OpInfo {cycle:2, mode: AddrModes::REL, opcode: OpCodes::BNE});
    m.insert(0xD1, OpInfo {cycle:5, mode: AddrModes::INDY, opcode: OpCodes::CMP});
    m.insert(0xD2, OpInfo {cycle:2, mode: AddrModes::IMPL, opcode: OpCodes::NOP});
    m.insert(0xD3, OpInfo {cycle:8, mode: AddrModes::INDY, opcode: OpCodes::DCP});
    m.insert(0xD4, OpInfo {cycle:4, mode: AddrModes::IMPL, opcode: OpCodes::NOPD});
    m.insert(0xD5, OpInfo {cycle:4, mode: AddrModes::ZPGX, opcode: OpCodes::CMP});
    m.insert(0xD6, OpInfo {cycle:6, mode: AddrModes::ZPGX, opcode: OpCodes::DEC});
    m.insert(0xD7, OpInfo {cycle:6, mode: AddrModes::ZPGX, opcode: OpCodes::DCP});
    m.insert(0xD8, OpInfo {cycle:2, mode: AddrModes::IMPL, opcode: OpCodes::CLD});
    m.insert(0xD9, OpInfo {cycle:4, mode: AddrModes::ABSY, opcode: OpCodes::CMP});
    m.insert(0xDA, OpInfo {cycle:2, mode: AddrModes::IMPL, opcode: OpCodes::NOP});
    m.insert(0xDB, OpInfo {cycle:7, mode: AddrModes::ABSY, opcode: OpCodes::DCP});
    m.insert(0xDC, OpInfo {cycle:4, mode: AddrModes::IMPL, opcode: OpCodes::NOPI});
    m.insert(0xDD, OpInfo {cycle:4, mode: AddrModes::ABSX, opcode: OpCodes::CMP});
    m.insert(0xDE, OpInfo {cycle:7, mode: AddrModes::ABSX, opcode: OpCodes::DEC});
    m.insert(0xDF, OpInfo {cycle:7, mode: AddrModes::ABSX, opcode: OpCodes::DCP});
    m.insert(0xE0, OpInfo {cycle:2, mode: AddrModes::IMD, opcode: OpCodes::CPX});
    m.insert(0xE1, OpInfo {cycle:6, mode: AddrModes::INDX, opcode: OpCodes::SBC});
    m.insert(0xE2, OpInfo {cycle:3, mode: AddrModes::IMPL, opcode: OpCodes::NOPD});
    m.insert(0xE3, OpInfo {cycle:8, mode: AddrModes::INDX, opcode: OpCodes::ISB});
    m.insert(0xE4, OpInfo {cycle:3, mode: AddrModes::ZPG, opcode: OpCodes::CPX});
    m.insert(0xE5, OpInfo {cycle:3, mode: AddrModes::ZPG, opcode: OpCodes::SBC});
    m.insert(0xE6, OpInfo {cycle:5, mode: AddrModes::ZPG, opcode: OpCodes::INC});
    m.insert(0xE7, OpInfo {cycle:5, mode: AddrModes::ZPG, opcode: OpCodes::ISB});
    m.insert(0xE8, OpInfo {cycle:2, mode: AddrModes::IMPL, opcode: OpCodes::INX});
    m.insert(0xE9, OpInfo {cycle:2, mode: AddrModes::IMD, opcode: OpCodes::SBC});
    m.insert(0xEA, OpInfo {cycle:2, mode: AddrModes::IMPL, opcode: OpCodes::NOP});
    m.insert(0xEB, OpInfo {cycle:2, mode: AddrModes::IMD, opcode: OpCodes::SBC});
    m.insert(0xEC, OpInfo {cycle:4, mode: AddrModes::ABS, opcode: OpCodes::CPX});
    m.insert(0xED, OpInfo {cycle:4, mode: AddrModes::ABS, opcode: OpCodes::SBC});
    m.insert(0xEE, OpInfo {cycle:6, mode: AddrModes::ABS, opcode: OpCodes::INC});
    m.insert(0xEF, OpInfo {cycle:6, mode: AddrModes::ABS, opcode: OpCodes::ISB});
    m.insert(0xF0, OpInfo {cycle:2, mode: AddrModes::REL, opcode: OpCodes::BEQ});
    m.insert(0xF1, OpInfo {cycle:5, mode: AddrModes::INDY, opcode: OpCodes::SBC});
    m.insert(0xF2, OpInfo {cycle:2, mode: AddrModes::IMPL, opcode: OpCodes::NOP});
    m.insert(0xF3, OpInfo {cycle:8, mode: AddrModes::INDY, opcode: OpCodes::ISB});
    m.insert(0xF4, OpInfo {cycle:4, mode: AddrModes::IMPL, opcode: OpCodes::NOPD});
    m.insert(0xF5, OpInfo {cycle:4, mode: AddrModes::ZPGX, opcode: OpCodes::SBC});
    m.insert(0xF6, OpInfo {cycle:6, mode: AddrModes::ZPGX, opcode: OpCodes::INC});
    m.insert(0xF7, OpInfo {cycle:6, mode: AddrModes::ZPGX, opcode: OpCodes::ISB});
    m.insert(0xF8, OpInfo {cycle:2, mode: AddrModes::IMPL, opcode: OpCodes::SED});
    m.insert(0xF9, OpInfo {cycle:4, mode: AddrModes::ABSY, opcode: OpCodes::SBC});
    m.insert(0xFA, OpInfo {cycle:2, mode: AddrModes::IMPL, opcode: OpCodes::NOP});
    m.insert(0xFB, OpInfo {cycle:7, mode: AddrModes::ABSY, opcode: OpCodes::ISB});
    m.insert(0xFC, OpInfo {cycle:4, mode: AddrModes::IMPL, opcode: OpCodes::NOPI});
    m.insert(0xFD, OpInfo {cycle:4, mode: AddrModes::ABSX, opcode: OpCodes::SBC});
    m.insert(0xFE, OpInfo {cycle:7, mode: AddrModes::ABSX, opcode: OpCodes::INC});
    m.insert(0xFF, OpInfo {cycle:7, mode: AddrModes::ABSX, opcode: OpCodes::ISB});
    m
});

#[derive(Debug, Clone, Copy, PartialEq)]
enum AddrModes {
    IMPL,
    ACM,
    IMD,
    ZPG,
    ZPGX,
    ZPGY,
    ABS,
    ABSX,
    ABSY,
    REL,
    INDX,
    INDY,
    ABSIND,
}

impl fmt::Display for AddrModes {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug, Clone, Copy)]
enum OpCodes {
    ADC,
    SBC,
    AND,
    ORA,
    EOR,
    ASL,
    LSR,
    ROL,
    ROR,
    BCC,
    BCS,
    BEQ,
    BNE,
    BVC,
    BVS,
    BPL,
    BMI,
    BIT,
    JMP,
    JSR,
    RTS,
    BRK,
    RTI,
    CMP,
    CPX,
    CPY,
    INC,
    DEC,
    INX,
    DEX,
    INY,
    DEY,
    CLC,
    SEC,
    CLI,
    SEI,
    CLD,
    SED,
    CLV,
    LDA,
    LDX,
    LDY,
    STA,
    STX,
    STY,
    TAX,
    TXA,
    TAY,
    TYA,
    TSX,
    TXS,
    PHA,
    PLA,
    PHP,
    PLP,
    NOPD,
    NOPI,
    NOP,
    LAX,
    SAX,
    DCP,
    ISB,
    SLO,
    RLA,
    SRE,
    RRA,
}

impl fmt::Display for OpCodes {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
#[derive(Debug, Clone, Copy)]
pub struct OpInfo {
    cycle: u8,
    mode: AddrModes,
    opcode: OpCodes,
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
    cycle: u16,
    has_branched: bool,
    reg: Register,
    ctx: &'a mut Context<'a>,
}

impl<'a> Cpu<'a> {
    pub fn new(ctx: &'a mut Context<'a>) -> Cpu<'a> {
        Cpu {
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
    fn bread(&mut self, addr: u16) -> u8 {
        self.read(addr)
    }
    fn wread(&mut self, addr: u16) -> u16 {
        self.read(addr) as u16 + ((self.read(addr +1) as u16) << 8)
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
    fn bfetch(&mut self) -> u8{
        let data: u8 = self.bread(self.reg.pc);
        self.reg.pc += 1;
        data
    }
    fn wfetch(&mut self) -> u16{
        let data: u16 = self.wread(self.reg.pc);
        self.reg.pc += 2;
        data
    }
    fn fetch_op(&mut self) -> FetchedOp{
        let pc = self.reg.pc;
        let index: u8 = self.bfetch();
        let op = OP_TABLE.get(&index).unwrap();
        let mut data: u32 = 0;
        let mut add_cycle: u8 = 0;
        match op.mode {
            AddrModes::ACM | AddrModes::IMPL => (),
            AddrModes::IMD | AddrModes::ZPG => data = self.bfetch() as u32,
            AddrModes::REL => {
                let addr: u32 = self.bfetch() as u32;
                data = ((addr + self.reg.pc as u32) - if addr < 0x80 {1} else {0x100}) as u32;
            },
            AddrModes::ZPGX => data = ((self.reg.x + self.bfetch()) & 0xFF) as u32,
            AddrModes::ZPGY => data = ((self.reg.y + self.bfetch()) & 0xFF) as u32,
            AddrModes::ABS => data = self.wfetch() as u32,
            AddrModes::ABSX => {
                let addr: u32 = self.wfetch() as u32;
                data = self.reg.x as u32 + addr;
                add_cycle = if ((data ^ addr) & 0xFF00) > 0 {1} else {0};
            },
            AddrModes::ABSY => {
                let addr: u32 = self.wfetch() as u32;
                data = self.reg.y as u32 + addr;
                add_cycle = if ((data ^ addr) & 0xFF00) > 0 {1} else {0};
            },
            AddrModes::INDX => {
                let baddr: u16 = (self.reg.x + self.bfetch()) as u16 & 0xFF;
                let baddr_: u16 = (baddr + 1) & 0xFF;
                data = self.bread(baddr) as u32 + (self.bread(baddr_) as u32) << 8;
            },
            AddrModes::INDY => {
                let baddr: u16 = self.bfetch() as u16;
                let baddr_: u16 = (baddr + 1) & 0xFF;
                data = self.bread(baddr) as u32 + (self.bread(baddr_) as u32) << 8;
                let data_: u32 = self.reg.y as u32;
                add_cycle = if ((data ^ data_) & 0xFF00) > 0 {1} else {0};
            },
            AddrModes::ABSIND => {
                let baddr: u16 = self.wfetch();
                let baddr_: u16 = (baddr & 0xFF00) + (baddr + 1) & 0xFF;
                data = self.bread(baddr) as u32 + (self.bread(baddr_) as u32) << 8;
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
    fn push(&mut self, data: u8) {
        self.write(self.reg.sp & 0xFF | 0x100, data);
        self.reg.sp -= 1;
    }
    fn push_pc(&mut self) {
        self.push((self.reg.pc >> 8) as u8);
        self.push((self.reg.pc & 0xFF) as u8);
    }
    fn push_reg_status(&mut self) {
        self.push(self.reg.p);
    }
    fn pop(&mut self) -> u8 {
        self.reg.sp += 1;
        self.bread(self.reg.sp & 0xFF | 0x100)
    }
    fn pop_pc(&mut self) {
        self.reg.pc = self.pop() as u16;
        self.reg.pc += ((self.pop() as u16) << 8);
    }
    fn pop_reg_status(&mut self) {
        self.reg.p = self.pop();
    }
    fn exec(&mut self, fop: &mut FetchedOp) {
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
                self.push((pc >> 8) as u8 & 0xFF);
                self.push(pc as u8 & 0xFF);
                self.reg.pc = data;
            },
            OpCodes::RTS => {
                self.pop_pc();
                self.reg.pc += 1;
            },
            // interrupt
            // comp
            // inc/dec
            OpCodes::INC => {
                let data_ :u8 = (self.bread(data) as u16 + 1) as u8;
                self.write(data, data_);
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
                let data_ :u8 = self.bread(data) - 1;
                self.write(data, data_);
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
                    _ => self.bread(data)
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
            OpCodes::STA => self.write(data, self.reg.a),
            OpCodes::STX => self.write(data, self.reg.x),
            OpCodes::STY => self.write(data, self.reg.y),
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
        println!("{:04} {:#05X} {:3} {:4} {:04X} A:{:02X} X:{:02X} Y:{:02X} P:{:02X} SP:{:04X} ",
            i, self.reg.pc, op.opcode.to_string(), op.mode.to_string(), fop.data,
            self.reg.a, self.reg.x, self.reg.y, self.reg.p, self.reg.sp);
    }
    pub fn run(&mut self) -> u16 {
        let mut fetched_op: FetchedOp = self.fetch_op();
        self.show_op(&fetched_op);
        self.exec(&mut fetched_op);
        let cycle: u16 = 
            (fetched_op.op.cycle + fetched_op.add_cycle) as u16;
        self.cycle += cycle;
        cycle
    }
}