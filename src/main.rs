// #![feature(test)]
pub mod nes;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();

    println!("{:?}", args);
    match args.len() {
        1 => nes::run("rom/nestest.nes", false),
        2 => nes::run(&args[1], false),
        _ => panic!("invalid args {:?}", args),
    } 

}