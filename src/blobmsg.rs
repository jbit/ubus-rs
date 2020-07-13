use super::{Blob, BlobIter};
use core::convert::TryInto;
use core::str;

values!(pub BlobMsgType(u32) {
    UNSPEC = 0,
    ARRAY  = 1,
    TABLE  = 2,
    STRING = 3,
    INT64  = 4,
    INT32  = 5,
    INT16  = 6,
    INT8   = 7,
    DOUBLE = 8,
});

#[derive(Debug)]
pub enum BlobMsgData<'a> {
    Array(BlobMsgIter<'a>),
    Table(BlobMsgIter<'a>),
    String(&'a str),
    Int64(i64),
    Int32(i32),
    Int16(i16),
    Int8(i8),
    Double(f64),
    Unknown(BlobMsgType, &'a [u8]),
}

pub struct BlobMsg<'a> {
    pub name: Option<&'a str>,
    pub data: BlobMsgData<'a>,
}

impl<'a> From<Blob<'a>> for BlobMsg<'a> {
    fn from(blob: Blob<'a>) -> Self {
        let data = match blob.tag.id().into() {
            BlobMsgType::ARRAY => BlobMsgData::Array(BlobMsgIter::new(blob.data)),
            BlobMsgType::TABLE => BlobMsgData::Table(BlobMsgIter::new(blob.data)),
            BlobMsgType::STRING => BlobMsgData::String(blob.try_into().unwrap()),
            BlobMsgType::INT64 => BlobMsgData::Int64(blob.try_into().unwrap()),
            BlobMsgType::INT32 => BlobMsgData::Int32(blob.try_into().unwrap()),
            BlobMsgType::INT16 => BlobMsgData::Int16(blob.try_into().unwrap()),
            BlobMsgType::INT8 => BlobMsgData::Int8(blob.try_into().unwrap()),
            id => BlobMsgData::Unknown(id, blob.data),
        };
        BlobMsg {
            name: blob.name,
            data,
        }
    }
}
impl core::fmt::Debug for BlobMsg<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        if let Some(name) = self.name {
            write!(f, "BlobMsg({}:{:?})", name, self.data)
        } else {
            write!(f, "BlobMsg({:?})", self.data)
        }
    }
}

pub struct BlobMsgIter<'a> {
    inner: BlobIter<'a>,
}
impl<'a> BlobMsgIter<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self {
            inner: BlobIter::new(data),
        }
    }
}
impl<'a> Iterator for BlobMsgIter<'a> {
    type Item = BlobMsg<'a>;
    fn next(&mut self) -> Option<BlobMsg<'a>> {
        self.inner.next().map(BlobMsg::from)
    }
}
impl core::fmt::Debug for BlobMsgIter<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "BlobMsgIter")
    }
}
