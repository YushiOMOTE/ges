extern crate proc_macro;

use nes_codegen::{Addr, Inst};
use proc_macro::TokenStream as TokenStream1;
use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{parse_macro_input, Ident, ItemFn};

fn load() -> Vec<Inst> {
    serde_yaml::from_str(include_str!("inst.yml")).unwrap()
}

fn load_by_name<T: ToString>(name: T) -> Vec<Inst> {
    let name = name.to_string().to_uppercase();
    load().into_iter().filter(|p| p.name == name).collect()
}

fn funcname(inst: &Inst) -> Ident {
    Ident::new(
        &format!("{}_{:02x}", inst.name.to_lowercase(), inst.opcode),
        Span::call_site(),
    )
}

// See: https://wiki.nesdev.com/w/index.php/CPU_addressing_modes
fn init(inst: &Inst) -> TokenStream {
    match inst.addr {
        Addr::Acc => quote! {
            let m = &mut reg.a;
        },
        Addr::Imp => quote! {},
        Addr::Imm | Addr::Rel => quote! {
            let mut _imm = *mmu.get(reg.pc + 1);
            let m = &mut _imm;
        },
        Addr::Zero => quote! {
            let m = {
                let imm = *mmu.get(reg.pc + 1);
                mmu.get_mut(imm as u16)
            };
        },
        Addr::ZeroX => quote! {
            let m = {
                let imm = *mmu.get(reg.pc + 1);
                mmu.get_mut(imm.wrapping_add(reg.x) as u16)
            };
        },
        Addr::ZeroY => quote! {
            let m = {
                let imm = *mmu.get(reg.pc + 1);
                mmu.get_mut(imm.wrapping_add(reg.y) as u16)
            };
        },
        Addr::Abs => quote! {
            let ma = {
                mmu.get16(reg.pc + 1)
            };
            let m = {
                mmu.get_mut(ma)
            };
        },
        Addr::AbsX => quote! {
            let ma = {
                mmu.get16(reg.pc + 1).wrapping_add(reg.x as u16)
            };
            let m = {
                mmu.get_mut(ma)
            };
        },
        Addr::AbsY => quote! {
            let ma = {
                mmu.get16(reg.pc + 1).wrapping_add(reg.y as u16)
            };
            let m = {
                mmu.get_mut(ma)
            };
        },
        Addr::Ind => quote! {
            let ma = {
                let p = mmu.get16(reg.pc + 1);
                mmu.get16(p)
            };
            let m = {
                mmu.get_mut(ma)
            };
        },
        Addr::IndX => quote! {
            let ma = {
                let p = *mmu.get(reg.pc + 1);
                let h = *mmu.get(p.wrapping_add(reg.x) as u16) as u16;
                let l = *mmu.get(p.wrapping_add(reg.x + 1) as u16) as u16;
                mmu.get16(h << 8 | l)
            };
            let m = {
                mmu.get_mut(ma)
            };
        },
        Addr::IndY => quote! {
            let ma = {
                let p = *mmu.get(reg.pc + 1);
                let h = *mmu.get(p as u16) as u16;
                let l = *mmu.get(p.wrapping_add(1) as u16) as u16;
                let p = h << 8 | l;
                mmu.get16(p.wrapping_add(reg.y as u16))
            };
            let m = {
                mmu.get_mut(ma)
            };
        },
    }
}

fn finalize(inst: &Inst) -> TokenStream {
    let len = inst.len as u16;
    let cycles = inst.cycles;

    quote! {
        reg.pc = reg.pc.wrapping_add(#len);
        #cycles
    }
}

#[proc_macro_attribute]
pub fn op(attr: TokenStream1, item: TokenStream1) -> TokenStream1 {
    let input = parse_macro_input!(item as ItemFn);

    let mut stream = TokenStream::new();

    let ident = &input.sig.ident;
    let block = &input.block;

    for inst in load_by_name(&ident) {
        let pre = init(&inst);
        let post = finalize(&inst);
        let ident = funcname(&inst);
        let result = quote! {
            fn #ident(reg: &mut Reg, mmu: &mut Mmu) -> usize {
                #pre
                #block
                #post
            }
        };
        stream.extend(result);
    }

    stream.into()
}
