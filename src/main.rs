pub mod nes;

fn main() {
    let rom = "rom/hello.nes";
    nes::run(rom);
}