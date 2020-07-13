#![no_std]
#![allow(dead_code)]

#[cfg(not(no_std))]
extern crate std;

/// Macro for defining helpful enum-like opaque structs
macro_rules! values {
    (
        $vis:vis $name:ident ( $repr:ty ) -> $other:ty {
            $( $variant:ident = $value:literal -> $othervalue:literal ),* $(,)?
        }
    ) => {
        values!($vis $name($repr) { $( $variant = $value , )* });
        impl $name {
            pub fn other(self) -> Option<$other> {
                match self {
                    $( Self::$variant => Some($othervalue), )*
                    _ => None,
                }
            }
        }
    };
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

macro_rules! invalid_data_panic {
    ($($arg:tt)*) => (if cfg!(debug_assertions) { panic!($($arg)*); })
}

macro_rules! valid_data {
    (($left:expr) >= ($right:expr), $msg:literal) => {{
        if !(($left) >= ($right)) {
            invalid_data_panic!("Invalid Data: {} ({:?} < {:?})", $msg, $left, $right);
            return Err(Error::InvalidData($msg));
        }
    }};
    (($left:expr) == ($right:expr), $msg:literal) => {{
        if !(($left) == ($right)) {
            invalid_data_panic!("Invalid Data: {} ({:?} != {:?})", $msg, $left, $right);
            return Err(Error::InvalidData($msg));
        }
    }};
    ($thing:expr, $msg:literal) => {{
        if !($thing) {
            invalid_data_panic!("Invalid Data: {}", $msg);
            return Err(Error::InvalidData($msg));
        }
    }};
}

mod no {
    #[derive(Debug)]
    pub struct IO(());
}

#[derive(Debug)]
pub enum Error<T = no::IO> {
    IO(T),
    InvalidData(&'static str),
}

impl<T: core::fmt::Debug> core::fmt::Display for Error<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        core::fmt::Debug::fmt(self, f)
    }
}

impl<T> From<core::convert::Infallible> for Error<T> {
    fn from(_: core::convert::Infallible) -> Self {
        unreachable!()
    }
}

impl<T: IOError> From<Error<no::IO>> for Error<T> {
    fn from(e: Error<no::IO>) -> Self {
        use Error::*;
        match e {
            IO(_) => unreachable!(),
            InvalidData(v) => InvalidData(v),
        }
    }
}

pub trait IOError {}

pub trait IO {
    type Error: IOError;
    fn put(&mut self, data: &[u8]) -> Result<(), Error<Self::Error>>;
    fn get(&mut self, data: &mut [u8]) -> Result<(), Error<Self::Error>>;
}

#[cfg(not(no_std))]
mod stdio;

mod blob;
mod blobmsg;
mod connection;
mod message;

pub use blob::*;
pub use blobmsg::*;
pub use connection::*;
pub use message::*;
