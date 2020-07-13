use super::{Blob, BlobIter, Error};
use core::convert::{TryFrom, TryInto};
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
    Array(BlobIter<'a, BlobMsg<'a>>),
    Table(BlobIter<'a, BlobMsg<'a>>),
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

impl<'a> TryFrom<Blob<'a>> for BlobMsg<'a> {
    type Error = Error;
    fn try_from(blob: Blob<'a>) -> Result<Self, Self::Error> {
        let data = match blob.tag.id().into() {
            BlobMsgType::ARRAY => BlobMsgData::Array(blob.try_into()?),
            BlobMsgType::TABLE => BlobMsgData::Table(blob.try_into()?),
            BlobMsgType::STRING => BlobMsgData::String(blob.try_into()?),
            BlobMsgType::INT64 => BlobMsgData::Int64(blob.try_into()?),
            BlobMsgType::INT32 => BlobMsgData::Int32(blob.try_into()?),
            BlobMsgType::INT16 => BlobMsgData::Int16(blob.try_into()?),
            BlobMsgType::INT8 => BlobMsgData::Int8(blob.try_into()?),
            id => BlobMsgData::Unknown(id, blob.data),
        };
        Ok(BlobMsg {
            name: blob.name,
            data,
        })
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
