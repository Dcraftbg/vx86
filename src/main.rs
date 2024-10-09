use std::{collections::HashMap, env, fs, io::{self, BufRead, Write}, process::ExitCode};

use disasm::disasm_inst;
use lazy_static::lazy_static;
use modrm::Modrm;
use prefix::{BitPrefix, Prefix};
use reader::Reader;
use reg::DISASM_REG32_MAP;
mod prefix;
mod reader;
mod disasm;
mod reg;
mod modrm;

#[derive(Debug)]
enum Escape {
    No
}
#[derive(Debug)]
struct Instruction {
    prefix: prefix::BitPrefix,
    escape: Escape,
    op: u8
}
fn parse_prefixes(reader: &mut Reader) -> Option<prefix::BitPrefix> {
    let mut prefix = 0;
    loop {
        match reader.peak_u8()? {
            0xF0 => {
                reader.eat(1);
                prefix |= Prefix::LOCK
            }
            0xF2 => {
                reader.eat(1);
                prefix |= Prefix::REPNE
            }
            0xF3 => {
                reader.eat(1);
                prefix |= Prefix::REP
            }

            0x2E => {
                reader.eat(1);
                prefix |= Prefix::CS_OV
            }
            0x36 => {
                reader.eat(1);
                prefix |= Prefix::SS_OV
            }
            0x3E => {
                reader.eat(1);
                prefix |= Prefix::DS_OV
            }
            0x26 => {
                reader.eat(1);
                prefix |= Prefix::ES_OV
            }
            0x64 => {
                reader.eat(1);
                prefix |= Prefix::FS_OV
            }
            0x65 => {
                reader.eat(1);
                prefix |= Prefix::GS_OV
            }
            _ => break
        }
    }
    Some(prefix)
}
struct VM {
    gprs: [u32; 8],
    rip: u32,
    ram: Vec<u8>,
}
impl VM {
    pub fn dump_gprs(&self) {
        for (i, gpr) in self.gprs.iter().copied().enumerate() {
            if i > 0 {
                if i % 4 == 0 {
                    eprintln!()
                } else {
                    eprint!(" ");
                }
            }
            eprint!("{}={:08X}", DISASM_REG32_MAP[i as usize], gpr)
        }
    }
}


type RunFunc = fn (vm: &mut VM, prefixes: BitPrefix, op: u8) -> Option<()>;

fn run_add(vm: &mut VM, prefixes: BitPrefix, op: u8) -> Option<()> {
    match op {
        0x01 => {
            let mut r = Reader::new(&vm.ram[vm.rip as usize..]);
            let modrm = Modrm(r.read_u8()?);
            match modrm.modb() {
                0b11 => {
                    vm.gprs[modrm.rm() as usize] = (vm.gprs[modrm.rm() as usize] & 0xFFFF0000) | (((vm.gprs[modrm.rm() as usize] as u16) + (vm.gprs[modrm.reg() as usize] as u16)) as u32);
                }
                modb => todo!("Unsupported modb={:2b} for add", modb)
            }
            vm.rip = r.offset_from(&vm.ram).unwrap() as u32;
            // eprintln!("add {}, {}", DISASM_REG16_MAP[(op-0x01) as usize])
        }
        _ => todo!("Handle 0x{:02X} in add", op)
    }
    Some(())
}
fn run_mov(vm: &mut VM, prefixes: BitPrefix, op: u8) -> Option<()> {
    match op {
        op if op >= 0xB8 && op <= 0xB8+7 => {
            let reg = op - 0xB8;
            let mut r = Reader::new(&vm.ram[vm.rip as usize..]);
            vm.gprs[reg as usize] = (r.read_u16()? as u32) | (vm.gprs[reg as usize] & 0xFFFF0000);
            vm.rip = r.offset_from(&vm.ram).unwrap() as u32;
        }
        _ => todo!("Handle 0x{:02X} in mov", op)
    }
    Some(())
}
lazy_static! {
    static ref vm_no_prefix: HashMap<u8, RunFunc> = {
        let mut m: HashMap<u8, RunFunc> = HashMap::new();
        for op in 0xB8..0xB8+8 {
            m.insert(op, run_mov);
        }
        m.insert(0x01, run_add);
        m
    };
}

fn run_opcode(vm: &mut VM, prefixes: BitPrefix) -> Option<()> {
    let mut r = Reader::new(&vm.ram[vm.rip as usize..]);
    match r.read_u8()? {
        0x0F => todo!("Escape sequence decoding is not supported"),
        op => {
            match vm_no_prefix.get(&op) {
                Some(h) => {
                    vm.rip = r.offset_from(&vm.ram).unwrap() as u32;
                    (h)(vm, prefixes, op)
                }
                None => todo!("Opcode 0x{:02X} is not supported", op)
            }
        }
    }
}
fn run_inst(vm: &mut VM) -> Option<()> {
    let mut r = Reader::new(&vm.ram[vm.rip as usize..]);
    let prefixes = parse_prefixes(&mut r)?;
    vm.rip = r.offset_from(&vm.ram).unwrap() as u32;
    run_opcode(vm, prefixes);
    Some(())
}

struct Dbg {
    vm: VM
}
impl Dbg {
    #[inline]
    const fn new(vm: VM) -> Self {
        Self { vm }
    }
    fn disasm(&self) {
        eprint!("{:08X}>", self.vm.rip);
        let mut r = Reader::new(&self.vm.ram[self.vm.rip as usize..]);
        if let None = disasm_inst(&mut r) {
            eprintln!("???");
        }
    }
    fn next(&mut self) {
        run_inst(&mut self.vm).expect("Failed to run Instruction");
    }
}

fn main() -> ExitCode {
    let mut args = env::args();
    let mut input = String::new();
    let _exe = args.next().expect("exe");
    let mut debug = false;
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "-dbg" => {
                debug = true;
            }
            _ => {
                if input.is_empty() { input = arg; }
                else {
                    eprintln!("ERROR: Unknown argument `{}`",arg);
                    return ExitCode::FAILURE;
                }
            }
        }
    }
    if input.is_empty() {
        eprintln!("ERROR: Missing input file");
        return ExitCode::FAILURE;
    }
    let bytes = match fs::read(&input) {
        Err(e) => {
            eprintln!("ERROR: Failed to read `{}`: {}", input, e);
            return ExitCode::FAILURE;
        }
        Ok(v) => v,
    };
    let mut vm = VM { gprs: [0;8], rip: 0, ram: bytes };
    if debug {
        let mut dbg = Dbg::new(vm);
        let stdin = io::stdin();
        let mut lastline = String::new();
        let mut lines = stdin.lock().lines();

        while (dbg.vm.rip as usize) < dbg.vm.ram.len() {
            dbg.disasm();
            eprint!(":");
            io::stderr().flush().unwrap();
            let line = match lines.next() {
                Some(v) => v,
                None => break,
            };
            if let Ok(mut l) = line {
                if l.is_empty() {
                    if lastline.is_empty() { continue; }
                    l = lastline;
                }
                let line = l.as_str().trim();
                match line {
                    "s" => dbg.next(),
                    _ => {
                        eprintln!("Unknown cmd {}", line);
                    }
                }
                lastline = l;
            }
        }
        eprintln!("INFO: Register dump:");
        dbg.vm.dump_gprs();
    } else {
        while (vm.rip as usize) < vm.ram.len() {
            run_inst(&mut vm).expect("Invalid Instruction");
        }
        eprintln!("INFO: Register dump:");
        vm.dump_gprs();
    }
    ExitCode::SUCCESS
}
