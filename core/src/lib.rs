use nes_codegen_derive::op;
use std::cell::RefCell;

pub struct Reg {
    /// Program counter
    pc: u16,
    /// Accumulator
    a: u8,
    /// X index register
    x: u8,
    /// Y index register
    y: u8,
    /// Stack pointer
    s: u8,
    /// Processor state
    p: u8,
}

impl Reg {
    fn new() -> Self {
        Self {
            pc: 0,
            a: 0,
            x: 0,
            y: 0,
            s: 0xfd,
            p: 0x34,
        }
    }
}

pub struct Mmu {
    mem: [u8; 0x1000],
}

impl Mmu {
    fn new() -> Self {
        Self { mem: [0; 0x1000] }
    }

    fn get(&self, addr: u16) -> &u8 {
        self.mem.get(addr as usize).unwrap()
    }

    fn get_mut(&mut self, addr: u16) -> &mut u8 {
        self.mem.get_mut(addr as usize).unwrap()
    }

    fn get16(&self, addr: u16) -> u16 {
        let h = *self.get(addr) as u16;
        let l = *self.get(addr.wrapping_add(1)) as u16;
        h << 8 | l
    }
}

macro_rules! c {
    ($e:ident) => {
        $e.p & 1
    };
    ($e:ident, $v:expr) => {
        $e.p = ($e.p & !0x01) | ($v as u8)
    };
}

macro_rules! n {
    ($e:ident, $v:expr) => {
        $e.p = ($e.p & !0x80) | ($v & 0x80)
    };
}

macro_rules! z {
    ($e:ident, $v:expr) => {
        if $v == 0 {
            $e.p |= 0x02;
        } else {
            $e.p &= !0x02;
        }
    };
}

#[test]
fn flg_c() {
    let mut r = Reg::new();
    r.p = 0xf0;
    c!(r, true);
    assert_eq!(r.p, 0xf1);
    c!(r, false);
    assert_eq!(r.p, 0xf0);
}

#[test]
fn flg_z() {
    let mut r = Reg::new();
    r.p = 0xf0;
    z!(r, 0);
    assert_eq!(r.p, 0xf2);
    z!(r, 10);
    assert_eq!(r.p, 0xf0);
}

#[test]
fn flg_n() {
    let mut r = Reg::new();
    r.p = 0x0f;
    n!(r, 0x80);
    assert_eq!(r.p, 0x8f);
    n!(r, 0x30);
    assert_eq!(r.p, 0x0f);
}

#[op]
fn adc(reg: &mut Reg, m: &mut u8) {
    let (v, o1) = reg.a.overflowing_add(*m);
    let (v, o2) = reg.a.overflowing_add(c!(reg));
    reg.a = v;
    c!(reg, o1 || o2);
    n!(reg, reg.a);
    z!(reg, reg.a);
}

pub fn run() {}
