use std::{env, fs, process::ExitCode};

use prefix::Prefix;
use reader::Reader;
mod prefix;
mod reader;
struct Instruction {
    prefix: prefix::BitPrefix
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
fn parse_inst(reader: &mut Reader) -> Option<Instruction> {
    let prefixes = parse_prefixes(reader)?;
    todo!("Parse the rest");
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

    println!("INFO: File size = {}", bytes.len());
    ExitCode::SUCCESS
}
