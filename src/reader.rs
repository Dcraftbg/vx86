
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
    pub fn eat(&mut self, n: usize) {
        self.data = &self.data[n.min(self.data.len())..]
    }
    #[inline]
    pub fn read_u8(&mut self) -> Option<u8> {
        Some(self.read(1)?[0])
    }

    #[inline]
    pub fn read_u16(&mut self) -> Option<u16> {
        Some(u16::from_le_bytes(self.read(2)?.try_into().unwrap()))
    }
    #[inline]
    pub const fn peak_u8(&self) -> Option<u8> {
        if self.data.len() < 1 { return None; }
        Some(self.data[0])
    }
    #[inline]
    pub const fn has_left(&self) -> bool {
        self.data.len() != 0
    }

    pub fn offset_from(&self, arr: &'a [u8]) -> Option<usize> {
        if self.data.as_ptr() < arr.as_ptr() { return None; }
        unsafe { 
            Some(self.data.as_ptr().offset_from(arr.as_ptr()) as usize)
        }
    }
}
