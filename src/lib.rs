#![no_std]
#![allow(dead_code)]

#[cfg(not(no_std))]
extern crate std;

/// Macro for defining helpful enum-like opaque structs
macro_rules! values {
    (
        $vis:vis $name:ident ( $repr:ty ) {
            $( $variant:ident = $value:literal ),* $(,)?
        }
    ) => {
        #[repr(transparent)]
        #[derive(Copy, Clone, PartialEq, Eq)]
        $vis struct $name($repr);
        impl $name {
            $( pub const $variant: Self = Self($value); )*
            pub fn known(self) -> bool {
                match self {
                    $( Self::$variant => true, )*
                    _ => false,
                }
            }
            pub fn value(self) -> $repr {
                self.0
            }
        }
        impl From<$repr> for $name {
            fn from(other: $repr) -> Self {
                Self(other)
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
        let mut buffer = MessageHeader::EMPTY_BUFFER;
        self.io.get(&mut buffer)?;
        let header = MessageHeader::from(buffer);

        assert_eq!(header.version, MessageVersion::CURRENT);
        assert!(header.tag.is_valid());

        // Truncate slice to length of data
        let data = &mut self.buffer[..header.tag.inner_len()];

        // Receive data into slice
        self.io.get(data)?;

        Ok(Message { header, data })
    }
}

#[cfg(test)]
mod test;

#[cfg(not(no_std))]
mod stdio;

mod blob;
mod blobmsg;
mod message;

pub use blob::*;
pub use blobmsg::*;
pub use message::*;
