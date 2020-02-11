use crate::rom::Rom;

pub struct Mmu {
    ram: [u8; 0x800],
    rom: [u8; 0x8000],
}

impl Mmu {
    pub fn new() -> Self {
        Self {
            ram: [0; 0x800],
            rom: [0; 0x8000],
        }
    }

    pub fn load_rom(&mut self, rom: &Rom) {
        self.rom.copy_from_slice(rom.prg());
    }

    pub fn get(&self, addr: u16) -> u8 {
        let addr = addr as usize;

        match addr {
            0..=0x1fff => self.ram[addr % 0x800],
            0x8000..=0xffff => self.rom[(addr - 0x8000) % 0x8000],
            e => unimplemented!("reading: {:04x}", e),
        }
    }

    pub fn set(&mut self, addr: u16, value: u8) {
        let addr = addr as usize;

        match addr {
            0..=0x1fff => {
                self.ram[addr % 0x800] = value;
            }
            e => unimplemented!("writing: {:04x}", e),
        }
    }

    pub fn getref(&mut self, addr: u16) -> Ref<'_> {
        Ref::new(self, addr)
    }

    pub fn get16(&self, addr: u16) -> u16 {
        let l = self.get(addr) as u16;
        let h = self.get(addr.wrapping_add(1)) as u16;
        h << 8 | l
    }
}

pub struct Ref<'a> {
    addr: u16,
    mmu: &'a mut Mmu,
}

impl<'a> Ref<'a> {
    pub fn new(mmu: &'a mut Mmu, addr: u16) -> Self {
        Self { addr, mmu }
    }

    pub fn addr(&self) -> u16 {
        self.addr
    }

    pub fn get(&self) -> u8 {
        self.mmu.get(self.addr)
    }

    pub fn set(&mut self, value: u8) {
        self.mmu.set(self.addr, value);
    }

    pub fn inspect(&self) -> String {
        format!(", mem={:04x}", self.addr)
    }
}
