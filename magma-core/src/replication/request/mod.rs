use digest::{Digest, Output};
use serde::{Deserialize, Serialize};

pub mod dto;

#[derive(Deserialize, Serialize)]
pub enum Ordering {
    /// the items in order of ascending depth
    Ascending,
    /// the items in order of descending depth
    Descending,
}

#[derive(Deserialize, Serialize)]
/// Specifies which items are of interest to the client
pub enum PathLength {
    /// the items along the shortest path
    ShortestPath,
    /// the items along the longest path
    LongestPath,
}

/// Describes which data the client wants from the server.
pub struct Request<D: Digest> {
    /// The hash of the magma event for which the client wants to obtain the accumulated value.
    pub new: Output<D>,
    /// The hash of the latest magma event for which the client already knows the accumulated value if available.
    pub old: Option<Output<D>>,
    /// Specifies in which order events and values should be transmitted.
    pub ordering: Ordering,
    /// Specifies whether to only return the shortest path or the longest path between `new` and
    /// `old`
    pub path_length: PathLength,
    /// Should the response include the values
    pub include_values: bool,
}
