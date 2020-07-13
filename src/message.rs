use crate::BlobTag;
use core::mem::size_of;
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

values!(pub MessageAttr(u8) {
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
    pub tag: BlobTag,
}

impl MessageHeader {
    pub const EMPTY_BUFFER: [u8; Self::SIZE] = [0xffu8; Self::SIZE];
    pub const SIZE: usize = size_of::<Self>();

    const OBJECT_EVENT: u32 = 1;
    const OBJECT_ACL: u32 = 2;
    const OBJECT_MONITOR: u32 = 3;

    const MONITOR_CLIENT: u8 = 0x00;
    const MONITOR_PEER: u8 = 0x01;
    const MONITOR_SEND: u8 = 0x02;
    const MONITOR_SEQ: u8 = 0x03;
    const MONITOR_TYPE: u8 = 0x04;
    const MONITOR_DATA: u8 = 0x05;

    const STATUS_OK: u8 = 0x00;
    const STATUS_INVALID_COMMAND: u8 = 0x01;
    const STATUS_INVALID_ARGUMENT: u8 = 0x02;
    const STATUS_METHOD_NOT_FOUND: u8 = 0x03;
    const STATUS_NOT_FOUND: u8 = 0x04;
    const STATUS_NO_DATA: u8 = 0x05;
    const STATUS_PERMISSION_DENIED: u8 = 0x06;
    const STATUS_TIMEOUT: u8 = 0x07;
    const STATUS_NOT_SUPPORTED: u8 = 0x08;
    const STATUS_UNKNOWN_ERROR: u8 = 0x09;
    const STATUS_CONNECTION_FAILED: u8 = 0x0a;
}

impl From<[u8; Self::SIZE]> for MessageHeader {
    fn from(buffer: [u8; Self::SIZE]) -> Self {
        unsafe { core::mem::transmute(buffer) }
    }
}
impl Into<[u8; Self::SIZE]> for MessageHeader {
    fn into(self) -> [u8; Self::SIZE] {
        unsafe { core::mem::transmute(self) }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Message {
    pub header: MessageHeader,
    pub data: (),
}
