use digest::{Digest, Output};
use serde::{Deserialize, Serialize};

pub mod dto;

#[derive(Deserialize, Serialize)]
/// Specifies which semigroup values are of interest to the client, and in which order they should be transmitted.
pub enum Mode {
    /// no semigroup values at all
    None,
    /// the labels along the shortest path from new to old, in order of ascending depth
    Ascending,
    /// the labels along the shortest path from new to old, in order of descending depth
    Descending,
    /// the labels along the longest path from new to old, in order of ascending depth
    AllAscending,
    /// the labels along the longest path from new to old, in order of descending depth
    AllDescending,
}

/// Describes which data the client wants from the server.
pub struct Request<D: Digest> {
    /// The hash of the magma event for which the client wants to obtain the accumulated value.
    pub new: Output<D>,
    /// The hash of the latest magma event for which the client already knows the accumulated value if available.
    pub old: Option<Output<D>>,
    /// Specifies which semigroup values are of interest to the client, and in which order they should be transmitted.
    pub mode: Mode,
    /// Instructs the server to omit transmitting the first `offset_value` magma events that would have been transmitted otherwise. This enables to efficiently resume responses that were interrupted by a network or endpoint failure.
    pub offset_event: u8,
    /// If this is not `None`, the specified number `n` and digest `d` allow resuming a response in the middle of transmitting a semigroup value. If the first `n` bytes of the first semigroup value of the response hash to `d`, they are omitted from the transmission. If they do not, this is indicated in the response, and the value is transmitted in full.
    pub offset_value: Option<u8>,
}
