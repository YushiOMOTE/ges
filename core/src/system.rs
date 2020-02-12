use crate::cpu::{execute, Reg};
use crate::mmu::Mmu;

pub fn run(reg: &mut Reg, mmu: &mut Mmu) {
    reg.reset(mmu);

    loop {
        let cycles = execute(reg, mmu);

        mmu.step(cycles);
    }
}
