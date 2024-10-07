
pub struct Reader<'a> {
    data: &'a [u8]
}
impl <'a> Reader <'a> {
    #[inline]
    pub const fn new(data: &'a [u8]) -> Self {
        Self { data }
    }
    pub fn read(&mut self, n: usize) -> Option<&'a [u8]> {
        if self.data.len() < n { return None; }
        let (a, b) = self.data.split_at(n);
        self.data = b;
        Some(a)
    }
    #[inline]
    pub fn read_u8(&mut self) -> Option<u8> {
        Some(self.read(1)?[0])
    }
}
