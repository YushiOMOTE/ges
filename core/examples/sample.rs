use nes::{run, Mmu, Reg, Rom};

fn main() {
    env_logger::init();

    let mut mmu = Mmu::new();
    let rom = include_bytes!("sample.nes");
    for b in &rom[0..16] {
        print!("{:02x} ", b);
    }
    println!();
    let rom = Rom::new(rom);
    mmu.load_rom(&rom);

    let mut reg = Reg::new();

    run(&mut reg, &mut mmu);
}
