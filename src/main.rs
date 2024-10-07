use std::{env, fs, process::ExitCode};
type BitPrefix = u32;
#[allow(non_snake_case)]
mod Prefix {
    use crate::BitPrefix;
    const fn bit(n: usize) -> BitPrefix { 1<<n }

    pub const LOCK : BitPrefix = bit(0);
    pub const REPNE: BitPrefix = bit(1);
    pub const REP  : BitPrefix = bit(2);

    pub const CS_OV: BitPrefix = bit(3); 
    pub const SS_OV: BitPrefix = bit(4); 
    pub const DS_OV: BitPrefix = bit(5);
    pub const ES_OV: BitPrefix = bit(6);
    pub const FS_OV: BitPrefix = bit(7);
    pub const GS_OV: BitPrefix = bit(8);

    pub const BRANCH_NOT_TAKEN: BitPrefix = bit(9);
    pub const BRANCH_TAKEN    : BitPrefix = bit(10);
    
    pub const OP_SIZE  : BitPrefix = bit(11);
    pub const ADDR_SIZE: BitPrefix = bit(12);
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
