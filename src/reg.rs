#[allow(non_snake_case)]
pub mod GPRReg {
    pub const A : usize = 0;
    pub const C : usize = 1;
    pub const D : usize = 2;
    pub const B : usize = 3;
    pub const SP: usize = 4;
    pub const BP: usize = 5;
    pub const SI: usize = 6;
    pub const DI: usize = 7;
}
pub const DISASM_REG16_MAP: &[&'static str] = &[
    "ax",
    "cx",
    "dx",
    "bx",
    "sp",
    "bp",
    "si",
    "di"
];
pub const DISASM_REG32_MAP: &[&'static str] = &[
    "eax",
    "ecx",
    "edx",
    "ebx",
    "esp",
    "ebp",
    "esi",
    "edi"
];
