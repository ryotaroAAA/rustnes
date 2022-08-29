// #![feature(test)]
pub mod nes;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();

    println!("{:?}", args);
    match args.len() {
        1 => nes::run("rom/nestest.nes", false),
        2..=4 => {
            let mut rom: &String = &"".to_string();
            let mut is_debug = false;
            for (i, a) in args.iter().enumerate() {
                match a.as_str() {
                    "-r" | "--rom" => {
                        if i < args.len() {
                            rom = &args[i+1];
                        }
                    },
                    "-d" | "--debug" => {
                        is_debug = true;
                    },
                    _ => (),
                }
            }
            nes::run(rom, is_debug);
        }
        _ => panic!("invalid args {:?}", args),
    } 

}