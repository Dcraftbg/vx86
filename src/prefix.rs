pub type BitPrefix = u32;
#[allow(non_snake_case)]
pub mod Prefix {
    use super::BitPrefix;

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
