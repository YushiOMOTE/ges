use log::*;

pub struct Ppu {
    // ctrl register
    nametable: usize,
    vram_inc32: bool,
    sptable: usize,
    bgtable: usize,
    spsize: usize,
    extout: bool,
    nmi: bool,
    scrollx: usize,
    scrolly: usize,
    // mask register
    greyscale: bool,
    show_bgleft: bool,
    show_spleft: bool,
    show_bg: bool,
    show_sp: bool,
    emp_red: bool,
    emp_green: bool,
    emp_blue: bool,
    // status
    sprite_of: bool,
    sprite_hit: bool,
    vblank: bool,
    // bus emulation
    last: u8,
    // oam
    oam: [u8; 64 * 4],
    oamaddr: usize,
    // scroll
    scrollxoff: usize,
    scrollyoff: usize,
    scrollstate: bool,
    // ppu
    ppumem: [u8; 0x4000],
    ppuaddr: usize,
    ppuaddrstate: bool,
}

impl Ppu {
    pub fn new() -> Self {
        Self {
            nametable: 0x2000,
            vram_inc32: false,
            sptable: 0x0000,
            bgtable: 0x0000,
            spsize: 8,
            extout: false,
            nmi: false,
            scrollx: 0,
            scrolly: 0,
            greyscale: false,
            show_bgleft: false,
            show_spleft: false,
            show_bg: false,
            show_sp: false,
            emp_red: false,
            emp_green: false,
            emp_blue: false,
            sprite_of: false,
            sprite_hit: false,
            vblank: false,
            last: 0,
            oam: [0; 64 * 4],
            oamaddr: 0,
            scrollxoff: 0,
            scrollyoff: 0,
            scrollstate: false,
            ppumem: [0; 0x4000],
            ppuaddr: 0,
            ppuaddrstate: false,
        }
    }

    pub fn get(&mut self, addr: usize) -> u8 {
        let value = match addr {
            0x2002 => {
                let mut v = 0;
                v |= self.last & 0x1f;
                v |= if self.sprite_of { 0x20 } else { 0x00 };
                v |= if self.sprite_hit { 0x40 } else { 0x00 };
                v |= if self.vblank { 0x80 } else { 0x00 };
                v
            }
            0x2004 => self.oam[self.oamaddr],
            0x2007 => self.ppumem[self.ppuaddr],
            _ => 0,
        };

        trace!("Read PPU: {:04x} -> {:02x}", addr, value);

        value
    }

    pub fn set(&mut self, addr: usize, value: u8) {
        trace!("Write PPU: {:04x} <- {:02x}", addr, value);

        self.last = value;

        match addr {
            0x2000 => {
                self.nametable = (value & 0x03) as usize * 0x400 + 0x2000;
                self.vram_inc32 = value & 0x04 != 0;
                self.sptable = if value & 0x08 != 0 { 0x1000 } else { 0x0000 };
                self.bgtable = if value & 0x10 != 0 { 0x1000 } else { 0x0000 };
                self.spsize = if value & 0x20 != 0 { 16 } else { 8 };
                self.extout = value & 0x40 != 0;
                self.nmi = value & 0x80 != 0;
                self.scrollx = if value & 0x01 != 0 { 256 } else { 0 };
                self.scrolly = if value & 0x02 != 0 { 240 } else { 0 };
            }
            0x2001 => {
                self.greyscale = value & 0x01 != 0;
                self.show_bgleft = value & 0x02 != 0;
                self.show_spleft = value & 0x04 != 0;
                self.show_bg = value & 0x08 != 0;
                self.show_sp = value & 0x10 != 0;
                self.emp_red = value & 0x20 != 0;
                self.emp_green = value & 0x40 != 0;
                self.emp_blue = value & 0x80 != 0;
            }
            0x2003 => {
                self.oamaddr = value as usize;
            }
            0x2004 => {
                self.oam[self.oamaddr] = value;
            }
            0x2005 => {
                if self.scrollstate {
                    self.scrollyoff = value as usize;
                    self.scrollstate = false;
                } else {
                    self.scrollxoff = value as usize;
                    self.scrollstate = true;
                }
            }
            0x2006 => {
                if self.ppuaddrstate {
                    self.ppuaddr = (self.ppuaddr & 0xff00) | (value as usize);
                    self.ppuaddrstate = false;
                } else {
                    self.ppuaddr = (self.ppuaddr & 0x00ff) | ((value as usize) << 8);
                    self.ppuaddrstate = true;
                }
            }
            0x2007 => {
                self.ppumem[self.ppuaddr] = value;
            }
            _ => {}
        }
    }
}
