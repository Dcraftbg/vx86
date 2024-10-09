use crate::{disasm::disasm_inst, reader::Reader, run::run_inst, vm::VM};


pub struct Dbg {
    pub vm: VM
}
impl Dbg {
    #[inline]
    pub const fn new(vm: VM) -> Self {
        Self { vm }
    }
    pub fn disasm(&self) {
        eprint!("{:08X}>", self.vm.rip);
        let mut r = Reader::new(&self.vm.ram[self.vm.rip as usize..]);
        if let None = disasm_inst(&mut r) {
            eprintln!("???");
        }
    }
    pub fn next(&mut self) {
        run_inst(&mut self.vm).expect("Failed to run Instruction");
    }
}
