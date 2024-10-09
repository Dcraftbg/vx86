use std::collections::HashMap;

use lazy_static::lazy_static;

use crate::{modrm::Modrm, parse::parse_prefixes, prefix::BitPrefix, reader::Reader, vm::VM};


type RunFunc = fn (vm: &mut VM, prefixes: BitPrefix, op: u8) -> Option<()>;

fn run_add(vm: &mut VM, _prefixes: BitPrefix, op: u8) -> Option<()> {
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
fn run_mov(vm: &mut VM, _prefixes: BitPrefix, op: u8) -> Option<()> {
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

pub fn run_opcode(vm: &mut VM, prefixes: BitPrefix) -> Option<()> {
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
pub fn run_inst(vm: &mut VM) -> Option<()> {
    let mut r = Reader::new(&vm.ram[vm.rip as usize..]);
    let prefixes = parse_prefixes(&mut r)?;
    vm.rip = r.offset_from(&vm.ram).unwrap() as u32;
    run_opcode(vm, prefixes);
    Some(())
}
