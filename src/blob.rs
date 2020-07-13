use core::mem::size_of;
use storage_endian::BEu32;

values!(pub BlobId(u8) {
    UNSPEC  = 0x00,
    NESTED  = 0x01,
    BINARY  = 0x02,
    STRING  = 0x03,
    INT8    = 0x04,
    INT16   = 0x05,
    INT32   = 0x06,
    INT64   = 0x07,
    DOUBLE  = 0x08,
});

#[repr(transparent)]
#[derive(Copy, Clone)]
pub struct BlobTag(BEu32);
impl BlobTag {
    const ID_MASK: u32 = 0x7f_00_00_00;
    const ID_SHIFT: u32 = 24;
    const LEN_MASK: u32 = 0x00_ff_ff_ff;
    const EXTENDED_BIT: u32 = 1 << 31;
    pub fn id(&self) -> BlobId {
        BlobId(u32::from((self.0 >> Self::ID_SHIFT) & Self::ID_MASK) as u8)
    }
    fn len(&self) -> usize {
        u32::from(self.0 & Self::LEN_MASK) as usize
    }
    pub fn data_len(&self) -> usize {
        self.len().saturating_sub(size_of::<Self>())
    }
    pub fn extended(&self) -> bool {
        (self.0 & Self::EXTENDED_BIT) != 0
    }
    pub fn valid(&self) -> bool {
        (self.len() >= size_of::<Self>()) && self.id().known()
    }
}
impl core::fmt::Debug for BlobTag {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        let (id, data_len) = (self.id(), self.data_len());
        let extended = if self.extended() { ", extended" } else { "" };
        write!(f, "BlobTag(id={:?}, data_len={}{})", id, data_len, extended)
    }
}
