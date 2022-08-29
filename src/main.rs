// #![feature(test)]
pub mod nes;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();

    println!("{:?}", args);
    match args.len() {
        1 => nes::run("rom/nestest.nes", false),
        2..=3 => {
            let rom = &args[1];
            let mut is_debug = false;
            if args.len() == 3 {
                is_debug = &args[2] == "debug";
            }
            nes::run(rom, is_debug);
        }
        _ => panic!("invalid args {:?}", args),
    } 

}