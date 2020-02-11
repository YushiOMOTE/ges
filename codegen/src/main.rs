use ges_codegen::{Addr, Inst};
use regex::Regex;
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Clone, Debug)]
enum State {
    // Looking for name (e.g. "AND")
    Name,
    // Looking for flag ("N Z C I D V")
    Flg,
    // Looking for flag state ("- - - - - -")
    FlgState,
    // Looking for the bar before operations
    OpsBar,
    // Looking for operations (e.g. "relative BCC oper...")
    Ops,
}

struct Parser {
    state: State,
    insts: Vec<Inst>,
    name: String,
    desc: String,
    flgstate: Vec<String>,
}

impl Parser {
    fn new() -> Self {
        Self {
            state: State::Name,
            insts: vec![],
            name: "".into(),
            desc: "".into(),
            flgstate: vec![],
        }
    }

    fn parse(&mut self, line: &str) {
        let state = self.state.clone();

        match state {
            State::Name => {
                let re = Regex::new(r"^[A-Z][A-Z][A-Z] .*$").unwrap();

                if re.is_match(line) {
                    let title: Vec<_> = line.splitn(2, ' ').collect();
                    let op = title[0].trim();
                    let desc = title[1].trim();

                    println!("{}: {}", op, desc);

                    self.state = State::Flg;
                    self.name = op.to_owned();
                    self.desc = desc.to_owned();
                }
            }
            State::Flg => {
                let re = Regex::new(r"N Z C I D V").unwrap();

                if re.is_match(line) {
                    self.state = State::FlgState;
                }
            }
            State::FlgState => {
                let state: Vec<_> = line.trim().splitn(6, ' ').map(|s| s.to_owned()).collect();
                self.flgstate = state;
                self.state = State::OpsBar;
            }
            State::OpsBar => {
                let re = Regex::new(r"--------------------------------------------").unwrap();

                if re.is_match(line) {
                    self.state = State::Ops;
                }
            }
            State::Ops => {
                let tokens: Vec<_> = line.split_whitespace().collect();

                if tokens.is_empty() {
                    self.state = State::Name;
                    return;
                }

                let addr: Addr = tokens[0].parse().unwrap();
                let re_opc = Regex::new("^[0-9A-Z][0-9A-Z]$").unwrap();
                let re_num = Regex::new("^[0-9]+").unwrap();
                let mut opc = None;
                let mut bytes = None;
                let mut cycles = None;
                let mut mnem = vec![];

                for t in &tokens[1..] {
                    if opc.is_none() && re_opc.is_match(t) {
                        opc = Some(u8::from_str_radix(t, 16).unwrap());
                    } else if re_num.is_match(t) {
                        if bytes.is_none() {
                            bytes = Some(t.parse().unwrap());
                        } else if cycles.is_none() {
                            cycles = Some(t.trim_end_matches('*').parse().unwrap());
                        } else {
                            mnem.push(t.to_owned());
                        }
                    } else {
                        mnem.push(t.to_owned());
                    }
                }

                self.insts.push(Inst {
                    name: self.name.clone(),
                    desc: self.desc.clone(),
                    mnem: mnem.join(" "),
                    opcode: opc.unwrap(),
                    addr: addr,
                    len: bytes.unwrap(),
                    cycles: cycles.unwrap(),
                    state: self.flgstate.clone(),
                });
            }
        }
    }
}

#[derive(StructOpt)]
struct Opt {
    /// Yaml file path to write instruction info.
    #[structopt(name = "output")]
    output: PathBuf,
}

fn main() {
    let opt = Opt::from_args();

    let s = include_str!("inst.txt");

    let mut p = Parser::new();

    for l in s.lines() {
        p.parse(l);
    }

    println!("{:#?}", p.insts);
    println!("{} instructions", p.insts.len());

    serde_yaml::to_writer(std::fs::File::create(opt.output).unwrap(), &p.insts).unwrap();
}
