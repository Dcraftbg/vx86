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
lazy_static! {
    static ref disasm_no_prefix: HashMap<u8, DisasmFunc> = {
        let mut m: HashMap<u8, DisasmFunc> = HashMap::new();
        for op in 0xB8..0xB8+8 {
            m.insert(op, disasm_mov);
        }
        m.insert(0x01, disasm_add);
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
    let mut r = Reader::new(&bytes);
    while r.has_left() {
        disasm_inst(&mut r).expect("Invalid Instruction");
    }
    println!("INFO: File size = {}", bytes.len());
    ExitCode::SUCCESS
}
