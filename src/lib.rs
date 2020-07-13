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
