use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
pub enum Addr {
    Acc,
    Imp,
    Imm,
    Zero,
    ZeroX,
    ZeroY,
    Rel,
    Abs,
    AbsX,
    AbsY,
    Ind,
    IndX,
    IndY,
}

impl FromStr for Addr {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "accumulator" => Addr::Acc,
            "implied" => Addr::Imp,
            "immidiate" => Addr::Imm,
            "zeropage" => Addr::Zero,
            "zeropage,X" => Addr::ZeroX,
            "zeropage,Y" => Addr::ZeroY,
            "relative" => Addr::Rel,
            "absolute" => Addr::Abs,
            "absolute,X" => Addr::AbsX,
            "absolute,Y" => Addr::AbsY,
            "indirect" => Addr::Ind,
            "(indirect,X)" => Addr::IndX,
            "(indirect),Y" => Addr::IndY,
            e => return Err(format!("Unknown addressing mode: {}", e)),
        })
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Inst {
    pub name: String,
    pub desc: String,
    pub mnem: String,
    pub opcode: u8,
    pub addr: Addr,
    pub len: usize,
    pub cycles: usize,
    pub state: Vec<String>,
}
