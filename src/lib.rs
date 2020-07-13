#![no_std]
#![allow(dead_code)]

#[cfg(not(no_std))]
extern crate std;

/// Macro for defining helpful enum-like opaque structs
macro_rules! values {
    (
        $vis:vis $name:ident ( $repr:ident ) {
            $( $variant:ident = $value:literal ),* $(,)?
        }
    ) => {
        #[repr(transparent)]
        #[derive(Copy, Clone, PartialEq, Eq)]
        $vis struct $name($repr);
        impl $name {
            $( pub const $variant: Self = Self($value); )*
            pub fn known(&self) -> bool {
                match *self {
                    $( Self::$variant => true, )*
                    _ => false,
                }
            }
            pub fn value(self) -> $repr {
                self.0
            }
        }
        impl core::fmt::Debug for $name {
            fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
                match *self {
                    $( Self::$variant => write!(f, stringify!($variant)), )*
                    unknown => write!(f, "UNKNOWN({})", unknown.0),
                }
            }
        }
    };
}

pub trait IO {
    type Error;
    fn put(&mut self, data: &[u8]) -> Result<(), Self::Error>;
    fn get(&mut self, data: &mut [u8]) -> Result<(), Self::Error>;
}

pub struct Connection<T: IO> {
    io: T,
    peer: u32,
}

impl<T: IO> Connection<T> {
    /// Create a new ubus connection from an existing IO
    pub fn new(io: T) -> Result<Self, T::Error> {
        let mut new = Self { io, peer: 0 };

        // ubus server should say hello on connect
        let header = new.next_header()?;

        // Verify the header is what we expect
        assert_eq!(header.version, MessageVersion::CURRENT);
        assert_eq!(header.message, MessageType::HELLO);
        assert!(header.tag.valid());
        assert_eq!(header.tag.data_len(), 0);

        // Record our peer id
        new.peer = header.peer.into();

        Ok(new)
    }

    // Get next message from ubus channel (blocking!)
    pub fn next_message(&mut self, buffer: &mut [u8]) -> Result<Message, T::Error> {
        let header = self.next_header()?;

        assert_eq!(header.version, MessageVersion::CURRENT);
        assert!(header.tag.valid());

        // Truncate slice to length of data
        let buffer = &mut buffer[0..header.tag.data_len()];

        // Receive data into slice
        self.io.get(buffer)?;

        Ok(Message { header, data: () })
    }

    fn next_header(&mut self) -> Result<MessageHeader, T::Error> {
        let mut buffer = MessageHeader::EMPTY_BUFFER;
        self.io.get(&mut buffer)?;
        Ok(MessageHeader::from(buffer))
    }
}

#[cfg(test)]
mod test;

#[cfg(not(no_std))]
mod stdio;

mod blob;
mod message;

pub use blob::*;
pub use message::*;