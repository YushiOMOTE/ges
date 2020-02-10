pub struct Reg {
    pc: u16,
    a: u8,
    x: u8,
    y: u8,
    sr: u8,
    sp: u8,
}

macro_rules! c {
    ($e:ident) => {
        $e.sr & 1
    };
    ($e:ident, $v:expr) => {
        $e.sr |= ($v as u8)
    };
}

macro_rules! n {
    ($e:ident, $v:expr) => {
        $e.sr = ($e.sr & !0x80) | ($v & 0x80)
    };
}

macro_rules! z {
    ($e:ident, $v:expr) => {
        if $v == 0 {
            $e.sr &= !0x40;
        } else {
            $e.sr |= 0x40;
        }
    };
}

// #[opcode]
fn adc(reg: &mut Reg, m: &mut u8) {
    let (v, o1) = reg.a.overflowing_add(*m);
    let (v, o2) = reg.a.overflowing_add(c!(reg));
    reg.a = v;
    c!(reg, o1 || o2);
    n!(reg, reg.a);
}

pub fn run() {}
