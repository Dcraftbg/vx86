use std::{collections::HashMap, env, fs, process::ExitCode};

use lazy_static::lazy_static;
use prefix::{BitPrefix, Prefix};
use reader::Reader;
mod prefix;
mod reader;

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
type DisasmFunc = fn (reader: &mut Reader, prefixes: BitPrefix, op: u8) -> Option<()>;
const DISASM_REG16_MAP: &[&'static str] = &[
    "ax",
    "cx",
    "dx",
    "bx",
    "sp",
    "bp",
    "si",
    "di"
];
const DISASM_REG32_MAP: &[&'static str] = &[
    "eax",
    "ecx",
    "edx",
    "ebx",
    "esp",
    "ebp",
    "esi",
    "edi"
];
#[derive(Debug, Clone, Copy)]
struct Modrm(u8);
impl Modrm {
    #[inline]
    pub const fn modb(self) -> u8 {
        self.0 >> 6
    }
    #[inline]
    pub const fn reg(self) -> u8 {
        (self.0 >> 3) & 0b111
    }

    #[inline]
    pub const fn rm(self) -> u8 {
        self.0 & 0b111
    }
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

fn disasm_add(r: &mut Reader, prefixes: BitPrefix, op: u8) -> Option<()> {
    match op {
        0x01 => {
            let modrm = Modrm(r.read_u8()?);
            match modrm.modb() {
                0b11 => {
                    eprintln!("add {}, {}", DISASM_REG16_MAP[modrm.rm() as usize], DISASM_REG16_MAP[modrm.reg() as usize])
                }
                modb => todo!("Unsupported modb={:2b} for add", modb)
            }
            // eprintln!("add {}, {}", DISASM_REG16_MAP[(op-0x01) as usize])
        }
        _ => todo!("Handle 0x{:02X} in add", op)
    }
    Some(())
}
fn disasm_mov(r: &mut Reader, prefixes: BitPrefix, op: u8) -> Option<()> {
    // TODO: Replace with a hashmap... I'm too lazy rn
    match op {
        op if op >= 0xB8 && op <= 0xB8+7 => {
            let reg = op - 0xB8;
            eprintln!("mov {}, {}", DISASM_REG16_MAP[reg as usize], r.read_u16()?); 
        }
        _ => todo!("Handle 0x{:02X} in mov", op)
    }
    Some(())
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
    static ref disasm_no_prefix: HashMap<u8, DisasmFunc> = {
        let mut m: HashMap<u8, DisasmFunc> = HashMap::new();
        for op in 0xB8..0xB8+8 {
            m.insert(op, disasm_mov);
        }
        m.insert(0x01, disasm_add);
        m
    };
    static ref vm_no_prefix: HashMap<u8, RunFunc> = {
        let mut m: HashMap<u8, RunFunc> = HashMap::new();
        for op in 0xB8..0xB8+8 {
            m.insert(op, run_mov);
        }
        m.insert(0x01, run_add);
        m
    };
}
fn disasm_opcode(reader: &mut Reader, prefixes: BitPrefix) -> Option<()> {
    match reader.read_u8()? {
        0x0F => todo!("Escape sequence decoding is not supported"),
        op => {
            match disasm_no_prefix.get(&op) {
                Some(h) => (h)(reader, prefixes, op),
                None => {
                    todo!("Opcode 0x{:02X} is not supported", op);
                }
            }
        }
    }
}
fn disasm_inst(reader: &mut Reader) -> Option<()> {
    let prefixes = parse_prefixes(reader)?;
    disasm_opcode(reader, prefixes);
    Some(())
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

fn main() -> ExitCode {
    let mut args = env::args();
    let mut input = String::new();
    let _exe = args.next().expect("exe");
    while let Some(arg) = args.next() {
        if input.is_empty() { input = arg; }
        else {
            eprintln!("ERROR: Unknown argument `{}`",arg);
            return ExitCode::FAILURE;
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
    while (vm.rip as usize) < vm.ram.len() {
        run_inst(&mut vm).expect("Invalid Instruction");
    }
    eprintln!("INFO: Register dump:");
    vm.dump_gprs();
    ExitCode::SUCCESS
}
