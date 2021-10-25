use crate::Event;
use core::convert::TryFrom;
use core::num::NonZeroU64;
use digest::{generic_array::GenericArray, Digest, Output};
use frunk::Semigroup;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
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
    /// The hash of the latest magma event for which the client already knows the accumulated value.
    pub old: Output<D>,
    /// The hash of the magma event for which the client wants to obtain the accumulated value.
    pub new: Output<D>,
    /// Specifies which semigroup values are of interest to the client, and in which order they should be transmitted. 
    pub mode: Mode,
    /// Instructs the server to omit transmitting the first `offset_value` magma events that would have been transmitted otherwise. This enables to efficiently resume responses that were interrupted by a network or endpoint failure.
    pub offset_event: u8,
    /// If this is not `None`, the specified number `n` and digest `d` allow resuming a response in the middle of transmitting a semigroup value. If the first `n` bytes of the first semigroup value of the response hash to `d`, they are omitted from the transmission. If they do not, this is indicated in the response, and the value is transmitted in full.
    pub offset_value: Option<u8>,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RequestDTO<'a> {
    pub new: &'a [u8],
    pub old: &'a [u8],
    pub mode: Mode,
    pub offset_event: u8,
    pub offset_value: Option<u8>,
}

/// The data clients query for.
pub enum Response<'a, 'b, D: Digest, S: Semigroup> {
    /// The server cannot serve a meaningful response, because it does not know about `old` or
    /// `new`.
    UnknownEvent,
    /// The magma events and semigroup values the client requested.
    Data{
        /// The magma events along the shortest path in the evolution from `new` to `old`.
        events: &'a [Event<D>],
        /// All other magma events between `new` to `old` if the request `mode` is `AllDescending` or `AllDescending`, or the empty set otherwise.
        additional_events: &'a [Event<D>],
        /// Whether the transmission of the first semigroup value starts at byte zero or at the `offset_value` of the request.
        is_complete_payload: bool,
        /// No semigroup values if the request `mode` is `None`, the semigroup values along the shortest path in the evolution between the specified magma events if the `mode` is `Ascending` or `Descending`, or the semigroup values along the longest path in the evolution between the specified magma events if the `mode`  is `AllDescending` or `AllDescending`.
        values: &'b [S],
    }
}

pub enum RequestDtoError {}
#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum ResponseDTO<> {
    UnknownEvent,
    Data{
        //events: &'a [&[u8]],
        //additional_events: &'a [&[u8]],
        is_complete_payload: bool,
        //values: &'a [&[u8]],
    }
}


impl<'a, D> TryFrom<RequestDTO<'a>> for Request<D>
where
    D: Digest,
{
    type Error = RequestDtoError;

    fn try_from(value: RequestDTO<'a>) -> Result<Self, Self::Error> {
        todo!()
    }
}
