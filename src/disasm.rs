use std::collections::HashMap;

use lazy_static::lazy_static;

use crate::{modrm::Modrm, parse_prefixes, prefix::BitPrefix, reader::Reader, reg::{GPRReg, DISASM_REG16_MAP}};


fn disasm_add(r: &mut Reader, _prefixes: BitPrefix, op: u8) -> Option<()> {
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
fn disasm_mov(r: &mut Reader, _prefixes: BitPrefix, op: u8) -> Option<()> {
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
fn disasm_cmp_rax_imm16(r: &mut Reader, _prefixes: BitPrefix, op: u8) -> Option<()> {
    match op {
        0x3D => {
            eprintln!("cmp {}, {}", DISASM_REG16_MAP[GPRReg::A], r.read_u16()?); 
        }
        _ => unreachable!("Invalid")
    }
    Some(())
}

pub type DisasmFunc = fn (reader: &mut Reader, prefixes: BitPrefix, op: u8) -> Option<()>;
lazy_static! {
    pub static ref disasm_no_prefix: HashMap<u8, DisasmFunc> = {
        let mut m: HashMap<u8, DisasmFunc> = HashMap::new();
        for op in 0xB8..0xB8+8 {
            m.insert(op, disasm_mov);
        }
        m.insert(0x01, disasm_add);
        m.insert(0x3D, disasm_cmp_rax_imm16);
        m
    };
}


pub fn disasm_opcode(reader: &mut Reader, prefixes: BitPrefix) -> Option<()> {
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
pub fn disasm_inst(reader: &mut Reader) -> Option<()> {
    let prefixes = parse_prefixes(reader)?;
    disasm_opcode(reader, prefixes);
    Some(())
}
