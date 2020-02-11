extern crate proc_macro;

use ges_codegen::{Addr, Inst};
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
    let defmem = match inst.addr {
        Addr::Acc => quote! {
            let mut mem = RegRef::new(&mut reg.a);
        },
        Addr::Imp => quote! {
            let mem = Imp::new();
        },
        Addr::Imm | Addr::Rel => quote! {
            let mut mem = Imm::new(mmu.get(reg.pc + 1));
        },
        Addr::Zero => quote! {
            let mut mem = {
                let imm = mmu.get(reg.pc + 1);
                mmu.getref(imm as u16)
            };
        },
        Addr::ZeroX => quote! {
            let mut mem = {
                let imm = mmu.get(reg.pc + 1);
                mmu.getref(imm.wrapping_add(reg.x) as u16)
            };
        },
        Addr::ZeroY => quote! {
            let mut mem = {
                let imm = mmu.get(reg.pc + 1);
                mmu.getref(imm.wrapping_add(reg.y) as u16)
            };
        },
        Addr::Abs => quote! {
            let mut mem = {
                let imm = mmu.get16(reg.pc + 1);
                mmu.getref(imm)
            };
        },
        Addr::AbsX => quote! {
            let mut mem = {
                let imm = mmu.get16(reg.pc + 1).wrapping_add(reg.x as u16);
                mmu.getref(imm)
            };
        },
        Addr::AbsY => quote! {
            let mut mem = {
                let imm = mmu.get16(reg.pc + 1).wrapping_add(reg.y as u16);
                mmu.getref(imm)
            };
        },
        Addr::Ind => quote! {
            let mut mem = {
                let imm = mmu.get16(reg.pc + 1);
                let addr = mmu.get16(imm);
                mmu.getref(addr)
            };
        },
        Addr::IndX => quote! {
            let mut mem = {
                let p = mmu.get(reg.pc + 1);
                let h = mmu.get(p.wrapping_add(reg.x) as u16) as u16;
                let l = mmu.get(p.wrapping_add(reg.x + 1) as u16) as u16;
                let addr = mmu.get16(h << 8 | l);
                mmu.getref(addr)
            };
        },
        Addr::IndY => quote! {
            let mut mem = {
                let p = mmu.get(reg.pc + 1);
                let h = mmu.get(p as u16) as u16;
                let l = mmu.get(p.wrapping_add(1) as u16) as u16;
                let p = h << 8 | l;
                let addr = mmu.get16(p.wrapping_add(reg.y as u16));
                mmu.getref(addr)
            };
        },
    };

    let cycles = inst.cycles;

    quote! {
        let mut ctx = Context::new(#cycles);
        #defmem
    }
}

fn debug(inst: &Inst) -> TokenStream {
    let mnem = &inst.mnem;
    let opcode = inst.opcode;

    quote! {
        log::trace!("[{:04x}]: {} (op={:02x}{})", reg.pc, #mnem, #opcode, mem.inspect());
    }
}

#[proc_macro_attribute]
pub fn op(_: TokenStream1, item: TokenStream1) -> TokenStream1 {
    let input = parse_macro_input!(item as ItemFn);

    let mut stream = TokenStream::new();

    let ident = &input.sig.ident;
    let block = &input.block;

    for inst in load_by_name(&ident) {
        let len = inst.len as u16;
        let init = init(&inst);
        let ident = funcname(&inst);
        let debug = debug(&inst);
        let result = quote! {
            fn #ident(reg: &mut Reg, mmu: &mut Mmu) -> usize {
                #init
                #debug
                #block
                if !ctx.jump {
                    reg.pc = reg.pc.wrapping_add(#len);
                }
                ctx.cycles
            }
        };
        stream.extend(result);
    }

    stream.into()
}

#[proc_macro_attribute]
pub fn decode(_: TokenStream1, item: TokenStream1) -> TokenStream1 {
    let input = parse_macro_input!(item as ItemFn);

    let patterns: Vec<_> = load()
        .into_iter()
        .map(|inst| {
            let opcode = inst.opcode;
            let ident = funcname(&inst);
            quote! {
                #opcode => #ident(reg, mmu)
            }
        })
        .collect();

    let ident = &input.sig.ident;

    let result = quote! {
        pub fn #ident(reg: &mut Reg, mmu: &mut Mmu) -> usize {
            let pc = mmu.get(reg.pc);

            match pc {
                #(#patterns,)*
                u => unreachable!("unknown opcode: {:02x}", u),
            }
        }
    };

    result.into()
}
