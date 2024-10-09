#[derive(Debug, Clone, Copy)]
pub struct Modrm(pub u8);
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
