#![allow(unused_variables)]

use core::panic;
use std::collections::HashMap;
use super::Cassette;
use super::Ram;
use super::ppu::*;
use super::Context;
use std::rc::*;
use std::cell::*;

pub const PAD_DELAY: usize = 10;
pub const PAD_INTERVAL: usize = 10;

#[derive(Debug)]
pub struct Video<'a> {
    data: Vec<Vec<u64>>,
    scale: u8
}

impl Video {
    pub fn new() -> Video {
        Video {
            data: vec![vec![0; H_SIZE]; V_SIZE],
            scale: 1
        }
    }
}
