pub struct Mmu {
    mem: [u8; 0x1000],
}

impl Mmu {
    pub fn new() -> Self {
        Self { mem: [0; 0x1000] }
    }

    pub fn get(&self, addr: u16) -> u8 {
        self.mem[addr as usize]
    }

    pub fn set(&mut self, addr: u16, value: u8) {
        self.mem[addr as usize] = value;
    }

    pub fn getref(&mut self, addr: u16) -> Ref<'_> {
        Ref::new(self, addr)
    }

    pub fn get16(&self, addr: u16) -> u16 {
        let h = self.get(addr) as u16;
        let l = self.get(addr.wrapping_add(1)) as u16;
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
}
