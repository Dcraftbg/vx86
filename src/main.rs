use std::{env, fs, io::{self, BufRead, Write}, process::ExitCode};

use dbg::Dbg;
use run::run_inst;
use vm::VM;

mod prefix;
mod reader;
mod disasm;
mod reg;
mod modrm;
mod parse;
mod run;
mod vm;
mod dbg;

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
