use core::convert::TryInto;
use core::mem::{align_of, size_of, transmute};
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
    const ID_MASK: u32 = 0x7f;
    const ID_SHIFT: u32 = 24;
    const LEN_MASK: u32 = 0xff_ff_ff;
    const EXTENDED_BIT: u32 = 1 << 31;
    const SIZE: usize = size_of::<Self>();
    const ALIGNMENT: usize = align_of::<Self>();
    /// Create BlobTag from a byte array
    pub fn from_bytes(bytes: [u8; Self::SIZE]) -> Self {
        unsafe { transmute(bytes) }
    }
    /// ID code of this blob
    pub fn id(&self) -> u32 {
        u32::from((self.0 >> Self::ID_SHIFT) & Self::ID_MASK)
    }
    /// Total number of bytes this blob contains (header + data)
    fn len(&self) -> usize {
        u32::from(self.0 & Self::LEN_MASK) as usize
    }
    /// Number of padding bytes between this blob and the next blob
    fn padding(&self) -> usize {
        Self::ALIGNMENT.wrapping_sub(self.data_len()) & (Self::ALIGNMENT - 1)
    }
    /// Number of bytes to the next tag
    fn next_tag(&self) -> usize {
        self.len() + self.padding()
    }
    /// Total number of bytes in the payload
    pub fn data_len(&self) -> usize {
        self.len().saturating_sub(Self::SIZE)
    }
    /// Is this an "extended" blob
    pub fn is_extended(&self) -> bool {
        (self.0 & Self::EXTENDED_BIT) != 0
    }
    /// Does this blob look valid
    pub fn is_valid(&self) -> bool {
        self.len() >= Self::SIZE
    }
}
impl core::fmt::Debug for BlobTag {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        let (id, data_len) = (self.id(), self.data_len());
        let extended = if self.is_extended() { ", extended" } else { "" };
        write!(f, "BlobTag(id={:?}, data_len={}{})", id, data_len, extended)
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Blob<'a> {
    pub tag: BlobTag,
    pub data: &'a [u8],
}

pub struct BlobIter<'a> {
    data: &'a [u8],
}
impl<'a> BlobIter<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self { data }
    }
}
impl<'a> Iterator for BlobIter<'a> {
    type Item = Blob<'a>;
    fn next(&mut self) -> Option<Blob<'a>> {
        if self.data.len() < 4 {
            return None;
        }

        // Read the blob's tag
        let bytes = self.data[..BlobTag::SIZE].try_into().unwrap();
        let tag = BlobTag::from_bytes(bytes);

        // Get data slice
        let data = &self.data[BlobTag::SIZE..BlobTag::SIZE + tag.data_len()];

        // Advance the internal pointer
        self.data = &self.data[tag.next_tag()..];

        Some(Blob { tag, data })
    }
}
