use log::*;

pub struct Ppu {}

impl Ppu {
    pub fn new() -> Self {
        Self {}
    }

    pub fn get(&self, sel: usize) -> u8 {
        trace!("Read PPU: {:04x} -> {:02x}", sel, 0);
        0
    }

    pub fn set(&self, sel: usize, value: u8) {
        trace!("Write PPU: {:04x} <- {:02x}", sel, value);
    }
}
