pub mod nes;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();

    println!("{:?}", args);
    match args.len() {
        1 => nes::run("rom/nestest.nes"),
        2 => nes::run(&args[1]),
        _ => panic!("invalid args {:?}", args),
    } 

}