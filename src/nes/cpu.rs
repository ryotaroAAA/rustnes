#![allow(unused_variables)]

pub mod ram;

use std::collections::HashMap;
use once_cell::sync::Lazy;

const CARRY: u8 = 1 << 0;
const ZERO: u8 = 1 << 1;
const IRQ: u8 = 1 << 2;
const DECIMAL: u8 = 1 << 3;
const BREAK: u8 = 1 << 4;
const OVERFLOW: u8 = 1 << 6;
const NEGATIVE: u8 = 1 << 7;

#[derive(Debug)]
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

#[derive(Debug)]
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

#[derive(Debug)]
pub struct OpInfo {
    cycle: u8,
    mode: AddrModes,
    op: OpCodes,
}

pub static OP_TABLE: Lazy<HashMap<u8, OpInfo>> = Lazy::new(|| {
    let mut m:HashMap<u8, OpInfo> = HashMap::new();
    m.insert(0x00, OpInfo {cycle:7, mode: AddrModes::IMPL, op: OpCodes::BRK});
    m.insert(0x01, OpInfo {cycle:6, mode: AddrModes::INDX, op: OpCodes::ORA});
    m.insert(0x02, OpInfo {cycle:2, mode: AddrModes::IMPL, op: OpCodes::NOP});
    m.insert(0x03, OpInfo {cycle:8, mode: AddrModes::INDX, op: OpCodes::SLO});
    m.insert(0x04, OpInfo {cycle:3, mode: AddrModes::IMPL, op: OpCodes::NOPD});
    m.insert(0x05, OpInfo {cycle:3, mode: AddrModes::ZPG, op: OpCodes::ORA});
    m.insert(0x06, OpInfo {cycle:5, mode: AddrModes::ZPG, op: OpCodes::ASL});
    m.insert(0x07, OpInfo {cycle:5, mode: AddrModes::ZPG, op: OpCodes::SLO});
    m.insert(0x08, OpInfo {cycle:3, mode: AddrModes::IMPL, op: OpCodes::PHP});
    m.insert(0x09, OpInfo {cycle:2, mode: AddrModes::IMD, op: OpCodes::ORA});
    m.insert(0x0A, OpInfo {cycle:2, mode: AddrModes::ACM, op: OpCodes::ASL});
    m.insert(0x0C, OpInfo {cycle:4, mode: AddrModes::IMPL, op: OpCodes::NOPI});
    m.insert(0x0D, OpInfo {cycle:4, mode: AddrModes::ABS, op: OpCodes::ORA});
    m.insert(0x0E, OpInfo {cycle:6, mode: AddrModes::ABS, op: OpCodes::ASL});
    m.insert(0x0F, OpInfo {cycle:6, mode: AddrModes::ABS, op: OpCodes::SLO});
    m.insert(0x10, OpInfo {cycle:2, mode: AddrModes::REL, op: OpCodes::BPL});
    m.insert(0x11, OpInfo {cycle:5, mode: AddrModes::INDY, op: OpCodes::ORA});
    m.insert(0x12, OpInfo {cycle:2, mode: AddrModes::IMPL, op: OpCodes::NOP});
    m.insert(0x13, OpInfo {cycle:8, mode: AddrModes::INDY, op: OpCodes::SLO});
    m.insert(0x14, OpInfo {cycle:4, mode: AddrModes::IMPL, op: OpCodes::NOPD});
    m.insert(0x15, OpInfo {cycle:4, mode: AddrModes::ZPGX, op: OpCodes::ORA});
    m.insert(0x16, OpInfo {cycle:6, mode: AddrModes::ZPGX, op: OpCodes::ASL});
    m.insert(0x17, OpInfo {cycle:6, mode: AddrModes::ZPGX, op: OpCodes::SLO});
    m.insert(0x18, OpInfo {cycle:2, mode: AddrModes::IMPL, op: OpCodes::CLC});
    m.insert(0x19, OpInfo {cycle:4, mode: AddrModes::ABSY, op: OpCodes::ORA});
    m.insert(0x1A, OpInfo {cycle:2, mode: AddrModes::IMPL, op: OpCodes::NOP});
    m.insert(0x1B, OpInfo {cycle:7, mode: AddrModes::ABSY, op: OpCodes::SLO});
    m.insert(0x1C, OpInfo {cycle:4, mode: AddrModes::IMPL, op: OpCodes::NOPI});
    m.insert(0x1D, OpInfo {cycle:4, mode: AddrModes::ABSX, op: OpCodes::ORA});
    m.insert(0x1E, OpInfo {cycle:6, mode: AddrModes::ABSX, op: OpCodes::ASL});
    m.insert(0x1F, OpInfo {cycle:7, mode: AddrModes::ABSX, op: OpCodes::SLO});
    m.insert(0x20, OpInfo {cycle:6, mode: AddrModes::ABS, op: OpCodes::JSR});
    m.insert(0x21, OpInfo {cycle:6, mode: AddrModes::INDX, op: OpCodes::AND});
    m.insert(0x22, OpInfo {cycle:2, mode: AddrModes::IMPL, op: OpCodes::NOP});
    m.insert(0x23, OpInfo {cycle:8, mode: AddrModes::INDX, op: OpCodes::RLA});
    m.insert(0x24, OpInfo {cycle:3, mode: AddrModes::ZPG, op: OpCodes::BIT});
    m.insert(0x25, OpInfo {cycle:3, mode: AddrModes::ZPG, op: OpCodes::AND});
    m.insert(0x26, OpInfo {cycle:5, mode: AddrModes::ZPG, op: OpCodes::ROL});
    m.insert(0x27, OpInfo {cycle:5, mode: AddrModes::ZPG, op: OpCodes::RLA});
    m.insert(0x28, OpInfo {cycle:4, mode: AddrModes::IMPL, op: OpCodes::PLP});
    m.insert(0x29, OpInfo {cycle:2, mode: AddrModes::IMD, op: OpCodes::AND});
    m.insert(0x2A, OpInfo {cycle:2, mode: AddrModes::ACM, op: OpCodes::ROL});
    m.insert(0x2C, OpInfo {cycle:4, mode: AddrModes::ABS, op: OpCodes::BIT});
    m.insert(0x2D, OpInfo {cycle:4, mode: AddrModes::ABS, op: OpCodes::AND});
    m.insert(0x2E, OpInfo {cycle:6, mode: AddrModes::ABS, op: OpCodes::ROL});
    m.insert(0x2F, OpInfo {cycle:6, mode: AddrModes::ABS, op: OpCodes::RLA});
    m.insert(0x30, OpInfo {cycle:2, mode: AddrModes::REL, op: OpCodes::BMI});
    m.insert(0x31, OpInfo {cycle:5, mode: AddrModes::INDY, op: OpCodes::AND});
    m.insert(0x32, OpInfo {cycle:2, mode: AddrModes::IMPL, op: OpCodes::NOP});
    m.insert(0x33, OpInfo {cycle:8, mode: AddrModes::INDY, op: OpCodes::RLA});
    m.insert(0x34, OpInfo {cycle:4, mode: AddrModes::IMPL, op: OpCodes::NOPD});
    m.insert(0x35, OpInfo {cycle:4, mode: AddrModes::ZPGX, op: OpCodes::AND});
    m.insert(0x36, OpInfo {cycle:6, mode: AddrModes::ZPGX, op: OpCodes::ROL});
    m.insert(0x37, OpInfo {cycle:6, mode: AddrModes::ZPGX, op: OpCodes::RLA});
    m.insert(0x38, OpInfo {cycle:2, mode: AddrModes::IMPL, op: OpCodes::SEC});
    m.insert(0x39, OpInfo {cycle:4, mode: AddrModes::ABSY, op: OpCodes::AND});
    m.insert(0x3A, OpInfo {cycle:2, mode: AddrModes::IMPL, op: OpCodes::NOP});
    m.insert(0x3B, OpInfo {cycle:7, mode: AddrModes::ABSY, op: OpCodes::RLA});
    m.insert(0x3C, OpInfo {cycle:4, mode: AddrModes::IMPL, op: OpCodes::NOPI});
    m.insert(0x3D, OpInfo {cycle:4, mode: AddrModes::ABSX, op: OpCodes::AND});
    m.insert(0x3E, OpInfo {cycle:6, mode: AddrModes::ABSX, op: OpCodes::ROL});
    m.insert(0x3F, OpInfo {cycle:7, mode: AddrModes::ABSX, op: OpCodes::RLA});
    m.insert(0x40, OpInfo {cycle:6, mode: AddrModes::IMPL, op: OpCodes::RTI});
    m.insert(0x41, OpInfo {cycle:6, mode: AddrModes::INDX, op: OpCodes::EOR});
    m.insert(0x42, OpInfo {cycle:2, mode: AddrModes::IMPL, op: OpCodes::NOP});
    m.insert(0x43, OpInfo {cycle:8, mode: AddrModes::INDX, op: OpCodes::SRE});
    m.insert(0x44, OpInfo {cycle:3, mode: AddrModes::IMPL, op: OpCodes::NOPD});
    m.insert(0x45, OpInfo {cycle:3, mode: AddrModes::ZPG, op: OpCodes::EOR});
    m.insert(0x46, OpInfo {cycle:5, mode: AddrModes::ZPG, op: OpCodes::LSR});
    m.insert(0x47, OpInfo {cycle:5, mode: AddrModes::ZPG, op: OpCodes::SRE});
    m.insert(0x48, OpInfo {cycle:3, mode: AddrModes::IMPL, op: OpCodes::PHA});
    m.insert(0x49, OpInfo {cycle:2, mode: AddrModes::IMD, op: OpCodes::EOR});
    m.insert(0x4A, OpInfo {cycle:2, mode: AddrModes::ACM, op: OpCodes::LSR});
    m.insert(0x4C, OpInfo {cycle:3, mode: AddrModes::ABS, op: OpCodes::JMP});
    m.insert(0x4D, OpInfo {cycle:4, mode: AddrModes::ABS, op: OpCodes::EOR});
    m.insert(0x4E, OpInfo {cycle:6, mode: AddrModes::ABS, op: OpCodes::LSR});
    m.insert(0x4F, OpInfo {cycle:6, mode: AddrModes::ABS, op: OpCodes::SRE});
    m.insert(0x50, OpInfo {cycle:2, mode: AddrModes::REL, op: OpCodes::BVC});
    m.insert(0x51, OpInfo {cycle:5, mode: AddrModes::INDY, op: OpCodes::EOR});
    m.insert(0x52, OpInfo {cycle:2, mode: AddrModes::IMPL, op: OpCodes::NOP});
    m.insert(0x53, OpInfo {cycle:8, mode: AddrModes::INDY, op: OpCodes::SRE});
    m.insert(0x54, OpInfo {cycle:4, mode: AddrModes::IMPL, op: OpCodes::NOPD});
    m.insert(0x55, OpInfo {cycle:4, mode: AddrModes::ZPGX, op: OpCodes::EOR});
    m.insert(0x56, OpInfo {cycle:6, mode: AddrModes::ZPGX, op: OpCodes::LSR});
    m.insert(0x57, OpInfo {cycle:6, mode: AddrModes::ZPGX, op: OpCodes::SRE});
    m.insert(0x58, OpInfo {cycle:2, mode: AddrModes::IMPL, op: OpCodes::CLI});
    m.insert(0x59, OpInfo {cycle:4, mode: AddrModes::ABSY, op: OpCodes::EOR});
    m.insert(0x5A, OpInfo {cycle:2, mode: AddrModes::IMPL, op: OpCodes::NOP});
    m.insert(0x5B, OpInfo {cycle:7, mode: AddrModes::ABSY, op: OpCodes::SRE});
    m.insert(0x5C, OpInfo {cycle:4, mode: AddrModes::IMPL, op: OpCodes::NOPI});
    m.insert(0x5D, OpInfo {cycle:4, mode: AddrModes::ABSX, op: OpCodes::EOR});
    m.insert(0x5E, OpInfo {cycle:6, mode: AddrModes::ABSX, op: OpCodes::LSR});
    m.insert(0x5F, OpInfo {cycle:7, mode: AddrModes::ABSX, op: OpCodes::SRE});
    m.insert(0x60, OpInfo {cycle:6, mode: AddrModes::IMPL, op: OpCodes::RTS});
    m.insert(0x61, OpInfo {cycle:6, mode: AddrModes::INDX, op: OpCodes::ADC});
    m.insert(0x62, OpInfo {cycle:2, mode: AddrModes::IMPL, op: OpCodes::NOP});
    m.insert(0x63, OpInfo {cycle:8, mode: AddrModes::INDX, op: OpCodes::RRA});
    m.insert(0x64, OpInfo {cycle:3, mode: AddrModes::IMPL, op: OpCodes::NOPD});
    m.insert(0x65, OpInfo {cycle:3, mode: AddrModes::ZPG, op: OpCodes::ADC});
    m.insert(0x66, OpInfo {cycle:5, mode: AddrModes::ZPG, op: OpCodes::ROR});
    m.insert(0x67, OpInfo {cycle:5, mode: AddrModes::ZPG, op: OpCodes::RRA});
    m.insert(0x68, OpInfo {cycle:4, mode: AddrModes::IMPL, op: OpCodes::PLA});
    m.insert(0x69, OpInfo {cycle:2, mode: AddrModes::IMD, op: OpCodes::ADC});
    m.insert(0x6A, OpInfo {cycle:2, mode: AddrModes::ACM, op: OpCodes::ROR});
    m.insert(0x6C, OpInfo {cycle:5, mode: AddrModes::ABSIND, op: OpCodes::JMP});
    m.insert(0x6D, OpInfo {cycle:4, mode: AddrModes::ABS, op: OpCodes::ADC});
    m.insert(0x6E, OpInfo {cycle:6, mode: AddrModes::ABS, op: OpCodes::ROR});
    m.insert(0x6F, OpInfo {cycle:6, mode: AddrModes::ABS, op: OpCodes::RRA});
    m.insert(0x70, OpInfo {cycle:2, mode: AddrModes::REL, op: OpCodes::BVS});
    m.insert(0x71, OpInfo {cycle:5, mode: AddrModes::INDY, op: OpCodes::ADC});
    m.insert(0x72, OpInfo {cycle:2, mode: AddrModes::IMPL, op: OpCodes::NOP});
    m.insert(0x73, OpInfo {cycle:8, mode: AddrModes::INDY, op: OpCodes::RRA});
    m.insert(0x74, OpInfo {cycle:4, mode: AddrModes::IMPL, op: OpCodes::NOPD});
    m.insert(0x75, OpInfo {cycle:4, mode: AddrModes::ZPGX, op: OpCodes::ADC});
    m.insert(0x76, OpInfo {cycle:6, mode: AddrModes::ZPGX, op: OpCodes::ROR});
    m.insert(0x77, OpInfo {cycle:6, mode: AddrModes::ZPGX, op: OpCodes::RRA});
    m.insert(0x78, OpInfo {cycle:2, mode: AddrModes::IMPL, op: OpCodes::SEI});
    m.insert(0x79, OpInfo {cycle:4, mode: AddrModes::ABSY, op: OpCodes::ADC});
    m.insert(0x7A, OpInfo {cycle:2, mode: AddrModes::IMPL, op: OpCodes::NOP});
    m.insert(0x7B, OpInfo {cycle:7, mode: AddrModes::ABSY, op: OpCodes::RRA});
    m.insert(0x7C, OpInfo {cycle:4, mode: AddrModes::IMPL, op: OpCodes::NOPI});
    m.insert(0x7D, OpInfo {cycle:4, mode: AddrModes::ABSX, op: OpCodes::ADC});
    m.insert(0x7E, OpInfo {cycle:6, mode: AddrModes::ABSX, op: OpCodes::ROR});
    m.insert(0x7F, OpInfo {cycle:7, mode: AddrModes::ABSX, op: OpCodes::RRA});
    m.insert(0x80, OpInfo {cycle:2, mode: AddrModes::IMPL, op: OpCodes::NOPD});
    m.insert(0x81, OpInfo {cycle:6, mode: AddrModes::INDX, op: OpCodes::STA});
    m.insert(0x82, OpInfo {cycle:2, mode: AddrModes::IMPL, op: OpCodes::NOPD});
    m.insert(0x83, OpInfo {cycle:6, mode: AddrModes::INDX, op: OpCodes::SAX});
    m.insert(0x84, OpInfo {cycle:3, mode: AddrModes::ZPG, op: OpCodes::STY});
    m.insert(0x85, OpInfo {cycle:3, mode: AddrModes::ZPG, op: OpCodes::STA});
    m.insert(0x86, OpInfo {cycle:3, mode: AddrModes::ZPG, op: OpCodes::STX});
    m.insert(0x87, OpInfo {cycle:3, mode: AddrModes::ZPG, op: OpCodes::SAX});
    m.insert(0x88, OpInfo {cycle:2, mode: AddrModes::IMPL, op: OpCodes::DEY});
    m.insert(0x89, OpInfo {cycle:2, mode: AddrModes::IMPL, op: OpCodes::NOPD});
    m.insert(0x8A, OpInfo {cycle:2, mode: AddrModes::IMPL, op: OpCodes::TXA});
    m.insert(0x8C, OpInfo {cycle:4, mode: AddrModes::ABS, op: OpCodes::STY});
    m.insert(0x8D, OpInfo {cycle:4, mode: AddrModes::ABS, op: OpCodes::STA});
    m.insert(0x8E, OpInfo {cycle:4, mode: AddrModes::ABS, op: OpCodes::STX});
    m.insert(0x8F, OpInfo {cycle:4, mode: AddrModes::ABS, op: OpCodes::SAX});
    m.insert(0x90, OpInfo {cycle:2, mode: AddrModes::REL, op: OpCodes::BCC});
    m.insert(0x91, OpInfo {cycle:6, mode: AddrModes::INDY, op: OpCodes::STA});
    m.insert(0x92, OpInfo {cycle:2, mode: AddrModes::IMPL, op: OpCodes::NOP});
    m.insert(0x94, OpInfo {cycle:4, mode: AddrModes::ZPGX, op: OpCodes::STY});
    m.insert(0x95, OpInfo {cycle:4, mode: AddrModes::ZPGX, op: OpCodes::STA});
    m.insert(0x96, OpInfo {cycle:4, mode: AddrModes::ZPGY, op: OpCodes::STX});
    m.insert(0x97, OpInfo {cycle:4, mode: AddrModes::ZPGY, op: OpCodes::SAX});
    m.insert(0x98, OpInfo {cycle:2, mode: AddrModes::IMPL, op: OpCodes::TYA});
    m.insert(0x99, OpInfo {cycle:5, mode: AddrModes::ABSY, op: OpCodes::STA});
    m.insert(0x9A, OpInfo {cycle:2, mode: AddrModes::IMPL, op: OpCodes::TXS});
    m.insert(0x9D, OpInfo {cycle:4, mode: AddrModes::ABSX, op: OpCodes::STA});
    m.insert(0xA0, OpInfo {cycle:2, mode: AddrModes::IMD, op: OpCodes::LDY});
    m.insert(0xA1, OpInfo {cycle:6, mode: AddrModes::INDX, op: OpCodes::LDA});
    m.insert(0xA2, OpInfo {cycle:2, mode: AddrModes::IMD, op: OpCodes::LDX});
    m.insert(0xA3, OpInfo {cycle:6, mode: AddrModes::INDX, op: OpCodes::LAX});
    m.insert(0xA4, OpInfo {cycle:3, mode: AddrModes::ZPG, op: OpCodes::LDY});
    m.insert(0xA5, OpInfo {cycle:3, mode: AddrModes::ZPG, op: OpCodes::LDA});
    m.insert(0xA6, OpInfo {cycle:3, mode: AddrModes::ZPG, op: OpCodes::LDX});
    m.insert(0xA7, OpInfo {cycle:3, mode: AddrModes::ZPG, op: OpCodes::LAX});
    m.insert(0xA8, OpInfo {cycle:2, mode: AddrModes::IMPL, op: OpCodes::TAY});
    m.insert(0xA9, OpInfo {cycle:2, mode: AddrModes::IMD, op: OpCodes::LDA});
    m.insert(0xAA, OpInfo {cycle:2, mode: AddrModes::IMPL, op: OpCodes::TAX});
    m.insert(0xAC, OpInfo {cycle:4, mode: AddrModes::ABS, op: OpCodes::LDY});
    m.insert(0xAD, OpInfo {cycle:4, mode: AddrModes::ABS, op: OpCodes::LDA});
    m.insert(0xAE, OpInfo {cycle:4, mode: AddrModes::ABS, op: OpCodes::LDX});
    m.insert(0xAF, OpInfo {cycle:4, mode: AddrModes::ABS, op: OpCodes::LAX});
    m.insert(0xB0, OpInfo {cycle:2, mode: AddrModes::REL, op: OpCodes::BCS});
    m.insert(0xB1, OpInfo {cycle:5, mode: AddrModes::INDY, op: OpCodes::LDA});
    m.insert(0xB2, OpInfo {cycle:2, mode: AddrModes::IMPL, op: OpCodes::NOP});
    m.insert(0xB3, OpInfo {cycle:5, mode: AddrModes::INDY, op: OpCodes::LAX});
    m.insert(0xB4, OpInfo {cycle:4, mode: AddrModes::ZPGX, op: OpCodes::LDY});
    m.insert(0xB5, OpInfo {cycle:4, mode: AddrModes::ZPGX, op: OpCodes::LDA});
    m.insert(0xB6, OpInfo {cycle:4, mode: AddrModes::ZPGY, op: OpCodes::LDX});
    m.insert(0xB7, OpInfo {cycle:4, mode: AddrModes::ZPGY, op: OpCodes::LAX});
    m.insert(0xB8, OpInfo {cycle:2, mode: AddrModes::IMPL, op: OpCodes::CLV});
    m.insert(0xB9, OpInfo {cycle:4, mode: AddrModes::ABSY, op: OpCodes::LDA});
    m.insert(0xBA, OpInfo {cycle:2, mode: AddrModes::IMPL, op: OpCodes::TSX});
    m.insert(0xBC, OpInfo {cycle:4, mode: AddrModes::ABSX, op: OpCodes::LDY});
    m.insert(0xBD, OpInfo {cycle:4, mode: AddrModes::ABSX, op: OpCodes::LDA});
    m.insert(0xBE, OpInfo {cycle:4, mode: AddrModes::ABSY, op: OpCodes::LDX});
    m.insert(0xBF, OpInfo {cycle:4, mode: AddrModes::ABSY, op: OpCodes::LAX});
    m.insert(0xC0, OpInfo {cycle:2, mode: AddrModes::IMD, op: OpCodes::CPY});
    m.insert(0xC1, OpInfo {cycle:6, mode: AddrModes::INDX, op: OpCodes::CMP});
    m.insert(0xC2, OpInfo {cycle:2, mode: AddrModes::IMPL, op: OpCodes::NOPD});
    m.insert(0xC3, OpInfo {cycle:8, mode: AddrModes::INDX, op: OpCodes::DCP});
    m.insert(0xC4, OpInfo {cycle:3, mode: AddrModes::ZPG, op: OpCodes::CPY});
    m.insert(0xC5, OpInfo {cycle:3, mode: AddrModes::ZPG, op: OpCodes::CMP});
    m.insert(0xC6, OpInfo {cycle:5, mode: AddrModes::ZPG, op: OpCodes::DEC});
    m.insert(0xC7, OpInfo {cycle:5, mode: AddrModes::ZPG, op: OpCodes::DCP});
    m.insert(0xC8, OpInfo {cycle:2, mode: AddrModes::IMPL, op: OpCodes::INY});
    m.insert(0xC9, OpInfo {cycle:2, mode: AddrModes::IMD, op: OpCodes::CMP});
    m.insert(0xCA, OpInfo {cycle:2, mode: AddrModes::IMPL, op: OpCodes::DEX});
    m.insert(0xCC, OpInfo {cycle:4, mode: AddrModes::ABS, op: OpCodes::CPY});
    m.insert(0xCD, OpInfo {cycle:4, mode: AddrModes::ABS, op: OpCodes::CMP});
    m.insert(0xCE, OpInfo {cycle:6, mode: AddrModes::ABS, op: OpCodes::DEC});
    m.insert(0xCF, OpInfo {cycle:6, mode: AddrModes::ABS, op: OpCodes::DCP});
    m.insert(0xD0, OpInfo {cycle:2, mode: AddrModes::REL, op: OpCodes::BNE});
    m.insert(0xD1, OpInfo {cycle:5, mode: AddrModes::INDY, op: OpCodes::CMP});
    m.insert(0xD2, OpInfo {cycle:2, mode: AddrModes::IMPL, op: OpCodes::NOP});
    m.insert(0xD3, OpInfo {cycle:8, mode: AddrModes::INDY, op: OpCodes::DCP});
    m.insert(0xD4, OpInfo {cycle:4, mode: AddrModes::IMPL, op: OpCodes::NOPD});
    m.insert(0xD5, OpInfo {cycle:4, mode: AddrModes::ZPGX, op: OpCodes::CMP});
    m.insert(0xD6, OpInfo {cycle:6, mode: AddrModes::ZPGX, op: OpCodes::DEC});
    m.insert(0xD7, OpInfo {cycle:6, mode: AddrModes::ZPGX, op: OpCodes::DCP});
    m.insert(0xD8, OpInfo {cycle:2, mode: AddrModes::IMPL, op: OpCodes::CLD});
    m.insert(0xD9, OpInfo {cycle:4, mode: AddrModes::ABSY, op: OpCodes::CMP});
    m.insert(0xDA, OpInfo {cycle:2, mode: AddrModes::IMPL, op: OpCodes::NOP});
    m.insert(0xDB, OpInfo {cycle:7, mode: AddrModes::ABSY, op: OpCodes::DCP});
    m.insert(0xDC, OpInfo {cycle:4, mode: AddrModes::IMPL, op: OpCodes::NOPI});
    m.insert(0xDD, OpInfo {cycle:4, mode: AddrModes::ABSX, op: OpCodes::CMP});
    m.insert(0xDE, OpInfo {cycle:7, mode: AddrModes::ABSX, op: OpCodes::DEC});
    m.insert(0xDF, OpInfo {cycle:7, mode: AddrModes::ABSX, op: OpCodes::DCP});
    m.insert(0xE0, OpInfo {cycle:2, mode: AddrModes::IMD, op: OpCodes::CPX});
    m.insert(0xE1, OpInfo {cycle:6, mode: AddrModes::INDX, op: OpCodes::SBC});
    m.insert(0xE2, OpInfo {cycle:3, mode: AddrModes::IMPL, op: OpCodes::NOPD});
    m.insert(0xE3, OpInfo {cycle:8, mode: AddrModes::INDX, op: OpCodes::ISB});
    m.insert(0xE4, OpInfo {cycle:3, mode: AddrModes::ZPG, op: OpCodes::CPX});
    m.insert(0xE5, OpInfo {cycle:3, mode: AddrModes::ZPG, op: OpCodes::SBC});
    m.insert(0xE6, OpInfo {cycle:5, mode: AddrModes::ZPG, op: OpCodes::INC});
    m.insert(0xE7, OpInfo {cycle:5, mode: AddrModes::ZPG, op: OpCodes::ISB});
    m.insert(0xE8, OpInfo {cycle:2, mode: AddrModes::IMPL, op: OpCodes::INX});
    m.insert(0xE9, OpInfo {cycle:2, mode: AddrModes::IMD, op: OpCodes::SBC});
    m.insert(0xEA, OpInfo {cycle:2, mode: AddrModes::IMPL, op: OpCodes::NOP});
    m.insert(0xEB, OpInfo {cycle:2, mode: AddrModes::IMD, op: OpCodes::SBC});
    m.insert(0xEC, OpInfo {cycle:4, mode: AddrModes::ABS, op: OpCodes::CPX});
    m.insert(0xED, OpInfo {cycle:4, mode: AddrModes::ABS, op: OpCodes::SBC});
    m.insert(0xEE, OpInfo {cycle:6, mode: AddrModes::ABS, op: OpCodes::INC});
    m.insert(0xEF, OpInfo {cycle:6, mode: AddrModes::ABS, op: OpCodes::ISB});
    m.insert(0xF0, OpInfo {cycle:2, mode: AddrModes::REL, op: OpCodes::BEQ});
    m.insert(0xF1, OpInfo {cycle:5, mode: AddrModes::INDY, op: OpCodes::SBC});
    m.insert(0xF2, OpInfo {cycle:2, mode: AddrModes::IMPL, op: OpCodes::NOP});
    m.insert(0xF3, OpInfo {cycle:8, mode: AddrModes::INDY, op: OpCodes::ISB});
    m.insert(0xF4, OpInfo {cycle:4, mode: AddrModes::IMPL, op: OpCodes::NOPD});
    m.insert(0xF5, OpInfo {cycle:4, mode: AddrModes::ZPGX, op: OpCodes::SBC});
    m.insert(0xF6, OpInfo {cycle:6, mode: AddrModes::ZPGX, op: OpCodes::INC});
    m.insert(0xF7, OpInfo {cycle:6, mode: AddrModes::ZPGX, op: OpCodes::ISB});
    m.insert(0xF8, OpInfo {cycle:2, mode: AddrModes::IMPL, op: OpCodes::SED});
    m.insert(0xF9, OpInfo {cycle:4, mode: AddrModes::ABSY, op: OpCodes::SBC});
    m.insert(0xFA, OpInfo {cycle:2, mode: AddrModes::IMPL, op: OpCodes::NOP});
    m.insert(0xFB, OpInfo {cycle:7, mode: AddrModes::ABSY, op: OpCodes::ISB});
    m.insert(0xFC, OpInfo {cycle:4, mode: AddrModes::IMPL, op: OpCodes::NOPI});
    m.insert(0xFD, OpInfo {cycle:4, mode: AddrModes::ABSX, op: OpCodes::SBC});
    m.insert(0xFE, OpInfo {cycle:7, mode: AddrModes::ABSX, op: OpCodes::INC});
    m.insert(0xFF, OpInfo {cycle:7, mode: AddrModes::ABSX, op: OpCodes::ISB});
    m
});

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
            sp: 0xfd,
            pc: 0xc000,
            p: 0x24,
        }
    }
    pub fn reset(&mut self) {
        self.a = 0;
        self.x = 0;
        self.y = 0;
        self.sp = 0xfd;
        self.pc = 0xc000;
        self.p = 0x24;
    }
}

#[derive(Debug)]
pub struct Cpu {
    cycle: u8,
    reg: Register,
    ram: Ram,
}

impl Cpu {
    pub fn new() -> Cpu {
        Cpu {
            cycle: 0,
            reg: Register::new(),
            ram: 1,
        }
    }
    pub fn reset(&mut self) {
        self.cycle = 0;
        self.reg.reset();
    }
}