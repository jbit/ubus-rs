use crate::{Blob, BlobTag, Message, MessageHeader, MessageType, MessageVersion};
use core::convert::TryInto;

pub trait IO {
    type Error;
    fn put(&mut self, data: &[u8]) -> Result<(), Self::Error>;
    fn get(&mut self, data: &mut [u8]) -> Result<(), Self::Error>;
}

pub struct Connection<T: IO> {
    io: T,
    peer: u32,
    buffer: [u8; 64 * 1024],
}

impl<T: IO> Connection<T> {
    /// Create a new ubus connection from an existing IO
    pub fn new(io: T) -> Result<Self, T::Error> {
        let mut new = Self {
            io,
            peer: 0,
            buffer: [0u8; 64 * 1024],
        };

        // ubus server should say hello on connect
        let message = new.next_message()?;

        // Verify the header is what we expect
        assert_eq!(message.header.message, MessageType::HELLO);

        // Record our peer id
        new.peer = message.header.peer.into();

        Ok(new)
    }

    // Get next message from ubus channel (blocking!)
    pub fn next_message(&mut self) -> Result<Message, T::Error> {
        // Read in the message header and the following blob tag
        let mut buffer = [0xffu8; MessageHeader::SIZE + BlobTag::SIZE];
        self.io.get(&mut buffer)?;

        let (header, tag) = buffer.split_at(MessageHeader::SIZE);

        let header = MessageHeader::from_bytes(header.try_into().unwrap());
        assert_eq!(header.version, MessageVersion::CURRENT);

        let tag = BlobTag::from_bytes(tag.try_into().unwrap());
        assert!(tag.is_valid());

        // Get a slice the size of the blobs data bytes (do we need to worry about padding here?)
        let data = &mut self.buffer[..tag.inner_len()];

        // Receive data into slice
        self.io.get(data)?;

        // Create the blob from our parts
        let blob = Blob::from_tag_and_data(tag, data).unwrap();

        Ok(Message { header, blob })
    }
}
