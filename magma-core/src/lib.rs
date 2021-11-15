#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "alloc")]
extern crate alloc;

pub use core::num::NonZeroU64;
pub use digest::{generic_array::GenericArray, Digest, Output};
pub use event::Event;
pub use frunk::Semigroup;
use snafu::AsErrorSource;

pub mod event;
pub mod replication;

pub trait CanonicalEncoding {
    type Error: AsErrorSource + core::fmt::Debug;
    fn encode(&self, buffer: &mut [u8]) -> Result<usize, Self::Error>;
    fn decode(buffer: &[u8]) -> Result<(Self, &[u8]), Self::Error>
    where
        Self: Sized;

    fn encoding_length(&self) -> usize;
}
