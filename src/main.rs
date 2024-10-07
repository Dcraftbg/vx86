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
lazy_static! {
    static ref disasm_no_prefix: HashMap<u8, DisasmFunc> = {
        let m = HashMap::new();
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
    disasm_inst(&mut r).expect("Invalid Instruction");
    println!("INFO: File size = {}", bytes.len());
    ExitCode::SUCCESS
}
