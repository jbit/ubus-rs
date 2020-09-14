use crate::{Blob, BlobBuilder, BlobIter, BlobMsg, BlobTag, Error, IO};
use core::convert::TryInto;
use core::mem::{size_of, transmute};
use storage_endian::{BEu16, BEu32};

values!(pub MessageVersion(u8) {
    CURRENT = 0x00,
});

values!(pub MessageType(u8) {
    HELLO           = 0x00,
    STATUS          = 0x01,
    DATA            = 0x02,
    PING            = 0x03,
    LOOKUP          = 0x04,
    INVOKE          = 0x05,
    ADD_OBJECT      = 0x06,
    REMOVE_OBJECT   = 0x07,
    SUBSCRIBE       = 0x08,
    UNSUBSCRIBE     = 0x09,
    NOTIFY          = 0x10,
    MONITOR         = 0x11,
});

values!(pub MessageAttrId(u32) {
    UNSPEC      = 0x00,
    STATUS      = 0x01,
    OBJPATH     = 0x02,
    OBJID       = 0x03,
    METHOD      = 0x04,
    OBJTYPE     = 0x05,
    SIGNATURE   = 0x06,
    DATA        = 0x07,
    TARGET      = 0x08,
    ACTIVE      = 0x09,
    NO_REPLY    = 0x0a,
    SUBSCRIBERS = 0x0b,
    USER        = 0x0c,
    GROUP       = 0x0d,
});

#[derive(Copy, Clone, Debug)]
#[repr(C)]
pub struct MessageHeader {
    pub version: MessageVersion,
    pub message: MessageType,
    pub sequence: BEu16,
    pub peer: BEu32,
}

impl MessageHeader {
    pub const SIZE: usize = size_of::<Self>();

    /// Create MessageHeader from a byte array
    pub fn from_bytes(buffer: [u8; Self::SIZE]) -> Self {
        unsafe { transmute(buffer) }
    }
    // Dump out bytes of MessageHeader
    pub fn to_bytes(self) -> [u8; Self::SIZE] {
        unsafe { core::mem::transmute(self) }
    }
}

#[derive(Copy, Clone)]
pub struct Message<'a> {
    pub header: MessageHeader,
    pub blob: Blob<'a>,
}

impl<'a> Message<'a> {
    pub fn from_io<T: IO>(io: &mut T, buffer: &'a mut [u8]) -> Result<Self, Error<T::Error>> {
        let (pre_buffer, buffer) = buffer.split_at_mut(MessageHeader::SIZE + BlobTag::SIZE);

        // Read in the message header and the following blob tag
        io.get(pre_buffer)?;

        let (header, tag) = pre_buffer.split_at(MessageHeader::SIZE);

        let header = MessageHeader::from_bytes(header.try_into().unwrap());
        valid_data!(header.version == MessageVersion::CURRENT, "Wrong version");

        let tag = BlobTag::from_bytes(tag.try_into().unwrap());
        tag.is_valid()?;

        // Get a slice the size of the blob's data bytes (do we need to worry about padding here?)
        let data = &mut buffer[..tag.inner_len()];

        // Receive data into slice
        io.get(data)?;

        // Create the blob from our parts
        let blob = Blob::from_tag_and_data(tag, data).unwrap();

        Ok(Message { header, blob })
    }
}

impl core::fmt::Debug for Message<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(
            f,
            "Message({:?} seq={} peer={:08x}, size={})",
            self.header.message,
            self.header.sequence,
            self.header.peer,
            self.blob.data.len()
        )
    }
}

pub struct MessageBuilder<'a> {
    buffer: &'a mut [u8],
    offset: usize,
}

impl<'a> MessageBuilder<'a> {
    pub fn new(buffer: &'a mut [u8], header: MessageHeader) -> Result<Self, Error> {
        valid_data!(
            buffer.len() >= (MessageHeader::SIZE + BlobTag::SIZE),
            "Builder buffer is too small"
        );

        let header_buf = &mut buffer[..MessageHeader::SIZE];
        let header_buf: &mut [u8; MessageHeader::SIZE] = header_buf.try_into().unwrap();
        *header_buf = header.to_bytes();

        let offset = MessageHeader::SIZE + BlobTag::SIZE;

        Ok(Self { buffer, offset })
    }

    pub fn put(&mut self, attr: MessageAttr) -> Result<(), Error> {
        let mut blob = BlobBuilder::from_bytes(&mut self.buffer[self.offset..]);

        match attr {
            MessageAttr::Status(val) => blob.push_u32(MessageAttrId::STATUS.value(), val as u32)?,
            MessageAttr::ObjPath(val) => blob.push_str(MessageAttrId::OBJPATH.value(), val)?,
            MessageAttr::ObjId(val) => blob.push_u32(MessageAttrId::OBJID.value(), val)?,
            MessageAttr::Method(val) => blob.push_str(MessageAttrId::METHOD.value(), val)?,
            MessageAttr::ObjType(val) => blob.push_u32(MessageAttrId::STATUS.value(), val)?,
            MessageAttr::Signature(_) => unimplemented!(),
            MessageAttr::Data(val) => blob.push_bytes(MessageAttrId::DATA.value(), val)?,
            MessageAttr::Target(val) => blob.push_u32(MessageAttrId::TARGET.value(), val)?,
            MessageAttr::Active(val) => blob.push_bool(MessageAttrId::USER.value(), val)?,
            MessageAttr::NoReply(val) => blob.push_bool(MessageAttrId::USER.value(), val)?,
            MessageAttr::Subscribers(_) => unimplemented!(),
            MessageAttr::User(val) => blob.push_str(MessageAttrId::USER.value(), val)?,
            MessageAttr::Group(val) => blob.push_str(MessageAttrId::GROUP.value(), val)?,
            MessageAttr::Unknown(id, val) => blob.push_bytes(id.value(), val)?,
        };

        self.offset += blob.len();

        Ok(())
    }

    pub fn finish(self) -> &'a [u8] {
        // Update tag with correct size
        let tag = BlobTag::new(0, self.offset - MessageHeader::SIZE).unwrap();
        let tag_buf = &mut self.buffer[MessageHeader::SIZE..MessageHeader::SIZE + BlobTag::SIZE];
        let tag_buf: &mut [u8; BlobTag::SIZE] = tag_buf.try_into().unwrap();
        *tag_buf = tag.to_bytes();

        &self.buffer[..self.offset]
    }
}
impl<'a> Into<&'a [u8]> for MessageBuilder<'a> {
    fn into(self) -> &'a [u8] {
        self.finish()
    }
}

#[derive(Debug)]
pub enum MessageAttr<'a> {
    Status(i32),
    ObjPath(&'a str),
    ObjId(u32),
    Method(&'a str),
    ObjType(u32),
    Signature(BlobIter<'a, BlobMsg<'a>>),
    Data(&'a [u8]),
    Target(u32),
    Active(bool),
    NoReply(bool),
    Subscribers(BlobIter<'a, Blob<'a>>),
    User(&'a str),
    Group(&'a str),
    Unknown(MessageAttrId, &'a [u8]),
}

impl<'a> From<Blob<'a>> for MessageAttr<'a> {
    fn from(blob: Blob<'a>) -> Self {
        match blob.tag.id().into() {
            MessageAttrId::STATUS => MessageAttr::Status(blob.try_into().unwrap()),
            MessageAttrId::OBJPATH => MessageAttr::ObjPath(blob.try_into().unwrap()),
            MessageAttrId::OBJID => MessageAttr::ObjId(blob.try_into().unwrap()),
            MessageAttrId::METHOD => MessageAttr::Method(blob.try_into().unwrap()),
            MessageAttrId::OBJTYPE => MessageAttr::ObjType(blob.try_into().unwrap()),
            MessageAttrId::SIGNATURE => MessageAttr::Signature(blob.try_into().unwrap()),
            MessageAttrId::DATA => MessageAttr::Data(blob.try_into().unwrap()),
            MessageAttrId::TARGET => MessageAttr::Target(blob.try_into().unwrap()),
            MessageAttrId::ACTIVE => MessageAttr::Active(blob.try_into().unwrap()),
            MessageAttrId::NO_REPLY => MessageAttr::NoReply(blob.try_into().unwrap()),
            MessageAttrId::SUBSCRIBERS => MessageAttr::Subscribers(blob.try_into().unwrap()),
            MessageAttrId::USER => MessageAttr::User(blob.try_into().unwrap()),
            MessageAttrId::GROUP => MessageAttr::Group(blob.try_into().unwrap()),
            id => MessageAttr::Unknown(id, blob.data),
        }
    }
}
