pub mod nes;

use nes::Nes;

fn main() {
    let rom = "rom/hello.nes";
    let nes:Nes = Nes::new(rom);

    // for a in cpu::OP_TABLE.iter() {
    //     println!("{} {:?}", a.0, a.1);
    // }
}