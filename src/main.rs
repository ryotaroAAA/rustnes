pub mod nes;

fn main() {
    // let rom = "rom/hello.nes";
    let rom = "rom/nestest.nes";
    nes::run(rom);
}