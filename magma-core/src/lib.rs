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
    type Error: AsErrorSource;
    fn encode(buffer: &mut [u8]) -> Result<usize, Self::Error>;
    fn decode(buffer: &[u8]) -> Result<(usize, Self), Self::Error>
    where
        Self: Sized;
}

#[cfg(test)]
mod tests {
    use crate::*;
    use blake2::Blake2b;

    type MyEvent = Event<Blake2b>;

    // todo
    // - verify
    //  - this is kinda specified by replication oddly enough
    // - api for client
    //  - client finds about a hash of an event. They want to get up to date. They might have an
    //  existing hash or none
    //      - verify a collection of events
    //      - all they care about is getting a collection of semigroup values they can combine to
    //      make the final value
    //  - client wants to publish a new Value relative to a given event.
    //      - who publishes the event? The client or server? Actually the client & server
    //      lingo is more about replication not about publishing.
    // - api for server
    //
}
