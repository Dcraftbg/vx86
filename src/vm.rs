use crate::reg::DISASM_REG32_MAP;

pub struct VM {
    pub gprs: [u32; 8],
    pub rip: u32,
    pub ram: Vec<u8>,
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

