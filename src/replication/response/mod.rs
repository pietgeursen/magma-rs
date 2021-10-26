use digest::Digest;
use frunk::Semigroup;

use crate::Event;

pub mod dto;

/// The data clients query for.
pub enum Response<'a, 'b, D: Digest, S: Semigroup> {
    /// The server cannot serve a meaningful response, because it does not know about `old` or
    /// `new`.
    UnknownEvent,
    /// The magma events and semigroup values the client requested.
    Data {
        /// The magma events along the shortest path in the evolution from `new` to `old`.
        events: &'a [Event<D>],
        /// All other magma events between `new` to `old` if the request `mode` is `AllDescending` or `AllDescending`, or the empty set otherwise.
        additional_events: &'a [Event<D>],
        /// Whether the transmission of the first semigroup value starts at byte zero or at the `offset_value` of the request.
        is_complete_payload: bool,
        /// No semigroup values if the request `mode` is `None`, the semigroup values along the shortest path in the evolution between the specified magma events if the `mode` is `Ascending` or `Descending`, or the semigroup values along the longest path in the evolution between the specified magma events if the `mode`  is `AllDescending` or `AllDescending`.
        values: &'b [S],
    },
}
