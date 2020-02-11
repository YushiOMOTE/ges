use log::*;
use nes_codegen_derive::{decode, op};

use crate::mmu::Mmu;

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
    pub fn new() -> Self {
        Self {
            pc: 0,
            a: 0,
            x: 0,
            y: 0,
            s: 0xfd,
            p: 0x34,
        }
    }

    pub fn push(&mut self, mmu: &mut Mmu, v: u8) {
        mmu.set(self.s as u16 + 0x100, v);
        self.s = self.s.wrapping_sub(1);
    }

    pub fn pop(&mut self, mmu: &mut Mmu) -> u8 {
        self.s = self.s.wrapping_add(1);
        mmu.get(self.s as u16 + 0x100)
    }

    pub fn push16(&mut self, mmu: &mut Mmu, v: u16) {
        self.push(mmu, (v >> 8) as u8);
        self.push(mmu, v as u8);
    }

    pub fn pop16(&mut self, mmu: &mut Mmu) -> u16 {
        let l = self.pop(mmu) as u16;
        let h = self.pop(mmu) as u16;
        h << 8 | l
    }
}

pub struct RegRef<'a> {
    reg: &'a mut u8,
}

impl<'a> RegRef<'a> {
    pub fn new(reg: &'a mut u8) -> Self {
        Self { reg }
    }

    pub fn get(&self) -> u8 {
        *self.reg
    }

    pub fn set(&mut self, value: u8) {
        *self.reg = value;
    }

    pub fn inspect(&self) -> String {
        format!(", acc={:02x}", *self.reg)
    }
}

pub struct Imm {
    imm: u8,
}

impl Imm {
    pub fn new(imm: u8) -> Self {
        Self { imm }
    }

    pub fn get(&self) -> u8 {
        self.imm
    }

    pub fn inspect(&self) -> String {
        format!(", imm={:02x}", self.imm)
    }
}

pub struct Imp;

impl Imp {
    pub fn new() -> Self {
        Self
    }

    pub fn inspect(&self) -> &'static str {
        ""
    }
}

pub struct Context {
    cycles: usize,
    jump: bool,
}

impl Context {
    pub fn new(cycles: usize) -> Self {
        Self {
            cycles,
            jump: false,
        }
    }

    pub fn jump(&mut self) {
        self.jump = true;
    }

    pub fn wait(&mut self, cycles: usize) {
        self.cycles += cycles;
    }
}

macro_rules! c {
    () => {
        0x01
    };
    ($e:ident) => {
        $e.p & c!()
    };
    ($e:ident, set) => {
        $e.p |= c!()
    };
    ($e:ident, unset) => {
        $e.p &= !c!()
    };
    ($e:ident, $v:expr) => {
        $e.p = ($e.p & !c!()) | ($v as u8)
    };
}

macro_rules! z {
    () => {
        0x02
    };
    ($e:ident) => {
        $e.p & z!()
    };
    ($e:ident, set) => {
        $e.p |= z!()
    };
    ($e:ident, unset) => {
        $e.p &= !z!()
    };
    ($e:ident, $v:expr) => {
        if $v == 0 {
            z!($e, set);
        } else {
            z!($e, unset);
        }
    };
}

macro_rules! i {
    () => {
        0x04
    };
    ($e:ident) => {
        $e.p & i!()
    };
    ($e:ident, set) => {
        $e.p |= i!()
    };
    ($e:ident, unset) => {
        $e.p &= !i!()
    };
}

macro_rules! d {
    () => {
        0x08
    };
    ($e:ident) => {
        $e.p & d!()
    };
    ($e:ident, set) => {
        $e.p |= d!()
    };
    ($e:ident, unset) => {
        $e.p &= !d!()
    };
}

macro_rules! b {
    () => {
        0x10
    };
    ($e:ident) => {
        $e.p & b!()
    };
    ($e:ident, set) => {
        $e.p |= b!()
    };
    ($e:ident, unset) => {
        $e.p &= !b!()
    };
}

macro_rules! v {
    () => {
        0x40
    };
    ($e:ident) => {
        $e.p & v!()
    };
    ($e:ident, set) => {
        $e.p |= v!()
    };
    ($e:ident, unset) => {
        $e.p &= !v!()
    };
    ($e:ident, $v:expr) => {
        $e.p = ($e.p & !v!()) | ($v & v!())
    };
}

macro_rules! n {
    () => {
        0x80
    };
    ($e:ident) => {
        $e.p & n!()
    };
    ($e:ident, set) => {
        $e.p |= n!()
    };
    ($e:ident, unset) => {
        $e.p &= !n!()
    };
    ($e:ident, $v:expr) => {
        $e.p = ($e.p & !n!()) | ($v & n!())
    };
}

#[op]
fn adc(reg: &mut Reg, ctx: Context, mem: Ref) {
    let (v, o1) = reg.a.overflowing_add(mem.get());
    let (v, o2) = v.overflowing_add(c!(reg));
    reg.a = v;
    c!(reg, o1 || o2);
    n!(reg, reg.a);
    z!(reg, reg.a);
    // TODO: v!
}

#[op]
fn and(reg: &mut Reg, ctx: Context, mem: Ref) {
    reg.a = reg.a & mem.get();
    n!(reg, reg.a);
    z!(reg, reg.a);
}

#[op]
fn asl(reg: &mut Reg, ctx: Context, mem: Ref) {
    let (a, o) = reg.a.overflowing_shl(1);
    reg.a = a;
    c!(reg, o);
    n!(reg, reg.a);
    z!(reg, reg.a);
}

fn reljump(reg: &mut Reg, ctx: &mut Context, off: u8) {
    let orig = reg.pc;
    let off = (off as i8 as i16) as u16;
    reg.pc = reg.pc + off;

    trace!(
        "Relative jump {:04x} + {} -> {:04x}",
        orig,
        off as i16,
        reg.pc
    );

    ctx.wait(if orig / 0x100 != reg.pc / 0x100 { 2 } else { 1 });
}

#[op]
fn bcc(reg: &mut Reg, mut ctx: Context, mem: Ref) {
    if c!(reg) == 0 {
        reljump(reg, &mut ctx, mem.get());
    }
}

#[op]
fn bcs(reg: &mut Reg, mut ctx: Context, mem: Ref) {
    if c!(reg) != 0 {
        reljump(reg, &mut ctx, mem.get());
    }
}

#[op]
fn beq(reg: &mut Reg, mut ctx: Context, mem: Ref) {
    if z!(reg) != 0 {
        reljump(reg, &mut ctx, mem.get());
    }
}

#[op]
fn bit(reg: &mut Reg, mut ctx: Context, mem: Ref) {
    let t = reg.a & mem.get();
    n!(reg, t);
    v!(reg, t);
    z!(reg, t);
}

#[op]
fn bmi(reg: &mut Reg, mut ctx: Context, mem: Ref) {
    if n!(reg) != 0 {
        reljump(reg, &mut ctx, mem.get());
    }
}

#[op]
fn bne(reg: &mut Reg, mut ctx: Context, mem: Ref) {
    if z!(reg) == 0 {
        reljump(reg, &mut ctx, mem.get());
    }
}

#[op]
fn bpl(reg: &mut Reg, mut ctx: Context, mem: Ref) {
    if n!(reg) == 0 {
        reljump(reg, &mut ctx, mem.get());
    }
}

#[op]
fn brk(reg: &mut Reg, mut ctx: Context, mem: Ref) {
    let pc = reg.pc + 2;
    reg.push16(mmu, pc);
    reg.push(mmu, reg.p);
    b!(reg, set);
}

#[op]
fn bvc(reg: &mut Reg, mut ctx: Context, mem: Ref) {
    if v!(reg) == 0 {
        reljump(reg, &mut ctx, mem.get());
    }
}

#[op]
fn bvs(reg: &mut Reg, mut ctx: Context, mem: Ref) {
    if v!(reg) != 0 {
        reljump(reg, &mut ctx, mem.get());
    }
}

#[op]
fn clc(reg: &mut Reg, mut ctx: Context, mem: Ref) {
    c!(reg, unset);
}

#[op]
fn cld(reg: &mut Reg, mut ctx: Context, mem: Ref) {
    d!(reg, unset);
}

#[op]
fn cli(reg: &mut Reg, mut ctx: Context, mem: Ref) {
    i!(reg, unset);
}

#[op]
fn clv(reg: &mut Reg, mut ctx: Context, mem: Ref) {
    v!(reg, unset);
}

#[op]
fn cmp(reg: &mut Reg, mut ctx: Context, mem: Ref) {
    let (t, o) = reg.a.overflowing_sub(mem.get());
    n!(reg, t);
    z!(reg, t);
    c!(reg, o);
}

#[op]
fn cpx(reg: &mut Reg, mut ctx: Context, mem: Ref) {
    let (t, o) = reg.x.overflowing_sub(mem.get());
    n!(reg, t);
    z!(reg, t);
    c!(reg, o);
}

#[op]
fn cpy(reg: &mut Reg, mut ctx: Context, mem: Ref) {
    let (t, o) = reg.y.overflowing_sub(mem.get());
    n!(reg, t);
    z!(reg, t);
    c!(reg, o);
}

#[op]
fn dec(reg: &mut Reg, mut ctx: Context, mem: Ref) {
    let v = mem.get().wrapping_sub(1);
    mem.set(v);
    n!(reg, v);
    z!(reg, v);
}

#[op]
fn dex(reg: &mut Reg, mut ctx: Context, mem: Ref) {
    reg.x = reg.x.wrapping_sub(1);
    n!(reg, reg.x);
    z!(reg, reg.x);
}

#[op]
fn dey(reg: &mut Reg, mut ctx: Context, mem: Ref) {
    reg.y = reg.y.wrapping_sub(1);
    n!(reg, reg.y);
    z!(reg, reg.y);
}

#[op]
fn eor(reg: &mut Reg, mut ctx: Context, mem: Ref) {
    reg.a = reg.a ^ mem.get();
    n!(reg, reg.a);
    z!(reg, reg.a);
}

#[op]
fn inc(reg: &mut Reg, mut ctx: Context, mem: Ref) {
    let v = mem.get().wrapping_add(1);
    mem.set(v);
    n!(reg, v);
    z!(reg, v);
}

#[op]
fn inx(reg: &mut Reg, mut ctx: Context, mem: Ref) {
    reg.x = reg.x.wrapping_add(1);
    n!(reg, reg.x);
    z!(reg, reg.x);
}

#[op]
fn iny(reg: &mut Reg, mut ctx: Context, mem: Ref) {
    reg.y = reg.y.wrapping_add(1);
    n!(reg, reg.y);
    z!(reg, reg.y);
}

#[op]
fn jmp(reg: &mut Reg, mut ctx: Context, mem: Ref) {
    reg.pc = mem.addr();
    ctx.jump();
}

#[op]
fn jsr(reg: &mut Reg, mut ctx: Context, mem: Ref) {
    let pc = reg.pc + 2;
    reg.pc = mem.addr();
    reg.push16(mmu, pc);
    ctx.jump();
}

#[op]
fn lda(reg: &mut Reg, mut ctx: Context, mem: Ref) {
    reg.a = mem.get();
    n!(reg, reg.a);
    z!(reg, reg.a);
}

#[op]
fn ldx(reg: &mut Reg, mut ctx: Context, mem: Ref) {
    reg.x = mem.get();
    n!(reg, reg.x);
    z!(reg, reg.x);
}

#[op]
fn ldy(reg: &mut Reg, mut ctx: Context, mem: Ref) {
    reg.y = mem.get();
    n!(reg, reg.y);
    z!(reg, reg.y);
}

#[op]
fn lsr(reg: &mut Reg, mut ctx: Context, mem: Ref) {
    let (v, o) = mem.get().overflowing_shr(1);
    mem.set(v);
    n!(reg, unset);
    z!(reg, v);
    c!(reg, o);
}

#[op]
fn nop(reg: &mut Reg, mut ctx: Context, mem: Ref) {}

#[op]
fn ora(reg: &mut Reg, mut ctx: Context, mem: Ref) {
    reg.a = reg.a | mem.get();
    n!(reg, reg.a);
    z!(reg, reg.a);
}

#[op]
fn pha(reg: &mut Reg, mut ctx: Context, mem: Ref) {
    reg.push(mmu, reg.a);
}

#[op]
fn php(reg: &mut Reg, mut ctx: Context, mem: Ref) {
    reg.push(mmu, reg.p);
}

#[op]
fn pla(reg: &mut Reg, mut ctx: Context, mem: Ref) {
    reg.a = reg.pop(mmu);
    n!(reg, reg.a);
    z!(reg, reg.a);
}

#[op]
fn plp(reg: &mut Reg, mut ctx: Context, mem: Ref) {
    reg.p = reg.pop(mmu);
}

#[op]
fn rol(reg: &mut Reg, mut ctx: Context, mem: Ref) {
    let (v, o) = mem.get().overflowing_shl(1);
    let v = v | c!(reg);
    mem.set(v);
    n!(reg, v);
    z!(reg, v);
    c!(reg, o);
}

#[op]
fn ror(reg: &mut Reg, mut ctx: Context, mem: Ref) {
    let (v, o) = mem.get().overflowing_shr(1);
    let v = v | (c!(reg) << 7);
    mem.set(v);
    n!(reg, v);
    z!(reg, v);
    c!(reg, o);
}

#[op]
fn rti(reg: &mut Reg, mut ctx: Context, mem: Ref) {
    reg.p = reg.pop(mmu);
    reg.pc = reg.pop16(mmu);
}

#[op]
fn rts(reg: &mut Reg, mut ctx: Context, mem: Ref) {
    reg.pc = reg.pop16(mmu);
}

#[op]
fn sbc(reg: &mut Reg, mut ctx: Context, mem: Ref) {
    let (v, o1) = reg.a.overflowing_sub(mem.get());
    let (v, o2) = v.overflowing_sub(c!(reg));
    reg.a = v;
    n!(reg, reg.a);
    z!(reg, reg.a);
    c!(reg, o1 || o2);
    // TODO: v!
}

#[op]
fn sec(reg: &mut Reg, mut ctx: Context, mem: Ref) {
    c!(reg, true);
}

#[op]
fn sed(reg: &mut Reg, mut ctx: Context, mem: Ref) {
    d!(reg, set);
}

#[op]
fn sei(reg: &mut Reg, mut ctx: Context, mem: Ref) {
    i!(reg, set);
}

#[op]
fn sta(reg: &mut Reg, mut ctx: Context, mem: Ref) {
    mem.set(reg.a);
}

#[op]
fn stx(reg: &mut Reg, mut ctx: Context, mem: Ref) {
    mem.set(reg.x);
}

#[op]
fn sty(reg: &mut Reg, mut ctx: Context, mem: Ref) {
    mem.set(reg.y);
}

#[op]
fn tax(reg: &mut Reg, mut ctx: Context, mem: Ref) {
    reg.x = reg.a;
    n!(reg, reg.x);
    z!(reg, reg.x);
}

#[op]
fn tay(reg: &mut Reg, mut ctx: Context, mem: Ref) {
    reg.y = reg.a;
    n!(reg, reg.y);
    z!(reg, reg.y);
}

#[op]
fn tsx(reg: &mut Reg, mut ctx: Context, mem: Ref) {
    reg.x = reg.s;
    n!(reg, reg.x);
    z!(reg, reg.x);
}

#[op]
fn txa(reg: &mut Reg, mut ctx: Context, mem: Ref) {
    reg.a = reg.x;
    n!(reg, reg.a);
    z!(reg, reg.a);
}

#[op]
fn txs(reg: &mut Reg, mut ctx: Context, mem: Ref) {
    reg.s = reg.x;
    n!(reg, reg.s);
    z!(reg, reg.s);
}

#[op]
fn tya(reg: &mut Reg, mut ctx: Context, mem: Ref) {
    reg.a = reg.y;
    n!(reg, reg.a);
    z!(reg, reg.a);
}

#[decode]
pub fn execute(reg: &mut Reg, mmu: &mut Mmu) -> usize {}

pub fn run(reg: &mut Reg, mmu: &mut Mmu) {
    // Reset vector
    reg.pc = mmu.get16(0xfffc);

    loop {
        let cycles = execute(reg, mmu);
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn flg_c() {
        let mut r = Reg::new();
        r.p = 0xf0;
        c!(r, set);
        assert_eq!(r.p, 0xf1);
        c!(r, unset);
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
    fn flg_i() {
        let mut r = Reg::new();
        r.p = 0xf0;
        i!(r, set);
        assert_eq!(r.p, 0xf4);
        i!(r, unset);
        assert_eq!(r.p, 0xf0);
    }

    #[test]
    fn flg_d() {
        let mut r = Reg::new();
        r.p = 0xf0;
        d!(r, set);
        assert_eq!(r.p, 0xf8);
        d!(r, unset);
        assert_eq!(r.p, 0xf0);
    }

    #[test]
    fn flg_b() {
        let mut r = Reg::new();
        r.p = 0x0f;
        b!(r, set);
        assert_eq!(r.p, 0x1f);
        b!(r, unset);
        assert_eq!(r.p, 0x0f);
    }

    #[test]
    fn flg_v() {
        let mut r = Reg::new();
        r.p = 0x0f;
        v!(r, set);
        assert_eq!(r.p, 0x4f);
        v!(r, unset);
        assert_eq!(r.p, 0x0f);
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
}
