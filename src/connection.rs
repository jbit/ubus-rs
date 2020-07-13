use crate::{Message, MessageBuilder, MessageType, IO};

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
        Message::from_io(&mut self.io, &mut self.buffer)
    }

    pub fn send(&mut self, message: MessageBuilder) -> Result<(), T::Error> {
        self.io.put(message.into())
    }
}
