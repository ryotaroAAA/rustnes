#![allow(unused_variables)]

extern crate yaml_rust;

mod cpu;

use std::fs;
use std::fs::File;
use std::io::prelude::*;
use yaml_rust::{YamlLoader, YamlEmitter, Yaml};

pub const PROG_ROM_MAX_SIZE: usize = 0x8000;
pub const CHAR_ROM_MAX_SIZE: usize = 0x2000;
pub const PROG_ROM_UNIT_SIZE: usize = 0x4000;
pub const CHAR_ROM_UNIT_SIZE: usize = 0x2000;
pub const NES_HSIZE: usize = 0x0010;

pub struct Cassette {
    path: String,
    rom_size: u64,
    prog_size: usize,
    char_size: usize,
    prog_rom: Vec<u8>,
    char_rom: Vec<u8>
}

impl Cassette {
    fn new(path: &str) -> Cassette{
        let mut f = File::open(path).expect("file not found");

        let mut buf = Vec::new();
        let size = f.read_to_end(&mut buf);
    
        let prog_size: usize = (buf[4] as usize) * PROG_ROM_UNIT_SIZE;
        let char_size: usize = (buf[5] as usize) * CHAR_ROM_UNIT_SIZE;
        let prog_rom_s: usize = NES_HSIZE;
        let char_rom_s: usize = prog_rom_s + prog_size;
        let prog_rom: Vec<u8> = buf[prog_rom_s..(prog_rom_s + prog_size)].to_vec();
        let char_rom: Vec<u8> = buf[char_rom_s..(char_rom_s + char_size)].to_vec();
    
        let metadata = std::fs::metadata(path);
        let rom_size: u64 = metadata.unwrap().len();

        println!("{:?}", &buf[..10]);
        println!("{:?}, {:?}, {:?}", rom_size, prog_size, char_size);
        
        Cassette {
            path: path.to_string(),
            rom_size: rom_size,
            prog_size: prog_size,
            char_size: char_size,
            prog_rom: prog_rom,
            char_rom: char_rom
        }        
    }
}


// OpInfo {i: 0, cycle: 7, mode: AddrModes::IMPL, op: OpCodes::BRK},

fn main() {
    // let path = "rom/hello.nes";
    // let cassette:Cassette = Cassette::new(path);

    let path = "./opset.yaml";
    let f = fs::read_to_string(path);
    let s = f.unwrap().to_string();
    let doc = &YamlLoader::load_from_str(&s).unwrap()[0];
    let op_set = doc.as_hash().unwrap();

    for (i, ops) in op_set {
        let index = i.as_i64().unwrap();
        let op = ops.as_hash().unwrap();
        let cycle = op.get(&Yaml::from_str("cycle")).unwrap().as_i64().unwrap();
        let mode = op.get(&Yaml::from_str("mode")).unwrap().as_str().unwrap();
        let name = op.get(&Yaml::from_str("op")).unwrap().as_str().unwrap();
        // println!("index:{:X}, cycle:{:?}, mode:{:?}, name:{:?}",
        //     index, cycle, mode, name);
        println!("k:{}, v: OpInfo {{i: {:#04X}, cycle:{:?}, mode: AddrMode::{:?}, op: OpCodes::{:?}}}",
            index, index, cycle, mode, name);
    }

    for a in cpu::OP_TABLE.iter() {
        println!("{} {:?}", a.0, a.1);
    }

    // // Index access for map & array
    // assert_eq!(doc["foo"][0].as_str().unwrap(), "list1");
    // assert_eq!(doc["bar"][1].as_f64().unwrap(), 2.0);

    // // Chained key/array access is checked and won't panic,
    // // return BadValue if they are not exist.
    // assert!(doc["INVALID_KEY"][100].is_badvalue());

    // // Dump the YAML object
    // let mut out_str = String::new();
    // {
    //     let mut emitter = YamlEmitter::new(&mut out_str);
    //     emitter.dump(doc).unwrap(); // dump the YAML object to a String
    // }
    // println!("{}", out_str);
}