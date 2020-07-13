use core::convert::TryInto;
use core::marker::PhantomData;
use core::mem::{align_of, size_of, transmute};
use core::str;
use storage_endian::BEu32;

#[repr(transparent)]
#[derive(Copy, Clone)]
pub struct BlobTag(BEu32);
impl BlobTag {
    pub const SIZE: usize = size_of::<Self>();
    const ID_MASK: u32 = 0x7f;
    const ID_SHIFT: u32 = 24;
    const LEN_MASK: u32 = 0xff_ff_ff;
    const EXTENDED_BIT: u32 = 1 << 31;
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
    pub fn size(&self) -> usize {
        u32::from(self.0 & Self::LEN_MASK) as usize
    }
    /// Number of padding bytes between this blob and the next blob
    fn padding(&self) -> usize {
        Self::ALIGNMENT.wrapping_sub(self.size()) & (Self::ALIGNMENT - 1)
    }
    /// Number of bytes to the next tag
    fn next_tag(&self) -> usize {
        self.size() + self.padding()
    }
    /// Total number of bytes following the tag (extended header + data)
    pub fn inner_len(&self) -> usize {
        self.size().saturating_sub(Self::SIZE)
    }
    /// Is this an "extended" blob
    pub fn is_extended(&self) -> bool {
        (self.0 & Self::EXTENDED_BIT) != 0
    }
    /// Does this blob look valid
    pub fn is_valid(&self) -> bool {
        self.size() >= Self::SIZE
    }
}
impl core::fmt::Debug for BlobTag {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        let (id, len) = (self.id(), self.size());
        let extended = if self.is_extended() { ", extended" } else { "" };
        write!(f, "BlobTag(id={:?}, len={}{})", id, len, extended)
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Blob<'a> {
    pub tag: BlobTag,
    pub data: &'a [u8],
    pub name: Option<&'a str>,
}

impl<'a> Blob<'a> {
    pub fn from_bytes(data: &'a [u8]) -> Option<Self> {
        if data.len() < BlobTag::SIZE {
            return None;
        }
        // Read the blob's tag
        let (tag, data) = data.split_at(BlobTag::SIZE);
        let tag = BlobTag::from_bytes(tag.try_into().unwrap());

        Self::from_tag_and_data(tag, data)
    }

    pub fn from_tag_and_data(tag: BlobTag, data: &'a [u8]) -> Option<Self> {
        if !tag.is_valid() || (data.len() < tag.inner_len()) {
            return None; // Actually an error!
        }

        // Restrict data to payload size
        let data = &data[..tag.inner_len()];

        if tag.is_extended() {
            // Extended blobs have a name at the beginning
            // Get the string length
            let (len_bytes, data) = data.split_at(size_of::<u16>());
            let ext_len = u16::from_be_bytes(len_bytes.try_into().unwrap()) as usize;
            // Get the string
            let (ext_bytes, data) = data.split_at(ext_len);
            let name = str::from_utf8(ext_bytes).unwrap();
            // Ensure the rest of the payload is aligned (+1 is for implicit nul terminator)
            let ext_total = size_of::<u16>() + ext_len + 1;
            let padding = BlobTag::ALIGNMENT.wrapping_sub(ext_total) & (BlobTag::ALIGNMENT - 1);
            let data = &data[padding + 1..];
            Some(Blob {
                tag,
                data,
                name: Some(name),
            })
        } else {
            Some(Blob {
                tag,
                data,
                name: None,
            })
        }
    }
}

macro_rules! try_into_be {
    ( $( $ty:ty , )* ) => { $( try_into_be!($ty); )* };
    ( $ty:ty ) => {
        impl TryInto<$ty> for Blob<'_> {
            type Error = Self;
            fn try_into(self) -> Result<$ty, Self::Error> {
                let size = size_of::<$ty>();
                if let Ok(bytes) = self.data[..size].try_into() {
                    Ok(<$ty>::from_be_bytes(bytes))
                } else {
                    Err(self)
                }
            }
        }
    };
}
try_into_be!(u8, i8, u16, i16, u32, i32, u64, i64, f64,);
impl<'a> TryInto<&'a str> for Blob<'a> {
    type Error = core::str::Utf8Error;
    fn try_into(self) -> Result<&'a str, Self::Error> {
        let data = if self.data.last() == Some(&b'\0') {
            &self.data[..self.data.len() - 1]
        } else {
            self.data
        };

        str::from_utf8(data)
    }
}

pub struct BlobIter<'a, T> {
    data: &'a [u8],
    _phantom: PhantomData<T>,
}
impl<'a, T> BlobIter<'a, T> {
    pub fn new(data: &'a [u8]) -> Self {
        Self {
            data,
            _phantom: PhantomData,
        }
    }
}
impl<'a, T: From<Blob<'a>>> Iterator for BlobIter<'a, T> {
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        if let Some(blob) = Blob::from_bytes(self.data) {
            // Advance the internal pointer to the next tag
            self.data = &self.data[blob.tag.next_tag()..];
            Some(blob.into())
        } else {
            None
        }
    }
}
impl<T> core::fmt::Debug for BlobIter<'_, T> {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "BlobIter")
    }
}
