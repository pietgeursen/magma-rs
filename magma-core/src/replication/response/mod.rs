use core::convert::TryFrom;
use snafu::{ensure, AsErrorSource, OptionExt, ResultExt, Snafu};

use digest::Digest;
use frunk::{semigroup::combine_all_option, Semigroup};

use crate::replication::request::Request;
use crate::{CanonicalEncoding, Event};

#[cfg(feature = "alloc")]
use alloc::vec::Vec;

pub mod dto;

pub struct EventPayloadPair<D: Digest, S: Semigroup> {
    pub event: Event<D>,
    pub payload: Option<S>,
}

pub enum Response<D: Digest, S: Semigroup + CanonicalEncoding> {
    UnknownEvent,
    Data(Vec<EventPayloadPair<D, S>>),
}

///// The data clients query for.
//pub enum ResponseRef<'a, 'b, D: Digest, S: Semigroup> {
//    /// The server cannot serve a meaningful response, because it does not know about `old` or
//    /// `new`.
//    UnknownEvent,
//    /// The magma events and semigroup values the client requested.
//    Data {
//        /// The magma events along the shortest path in the evolution from `new` to `old`.
//        events: &'a [Event<D>],
//        /// All other magma events between `new` to `old` if the request `mode` is `AllDescending` or `AllDescending`, or the empty set otherwise.
//        additional_events: &'a [Event<D>],
//        /// Whether the transmission of the first semigroup value starts at byte zero or at the `offset_value` of the request.
//        is_complete_payload: bool,
//        /// No semigroup values if the request `mode` is `None`, the semigroup values along the shortest path in the evolution between the specified magma events if the `mode` is `Ascending` or `Descending`, or the semigroup values along the longest path in the evolution between the specified magma events if the `mode`  is `AllDescending` or `AllDescending`.
//        values: &'b [S],
//    },
//}
//
//#[cfg(any(feature = "alloc", feature = "std"))]
//pub enum Response<D: Digest, S: Semigroup> {
//    /// The server cannot serve a meaningful response, because it does not know about `old` or
//    /// `new`.
//    UnknownEvent,
//    /// The magma events and semigroup values the client requested.
//    Data {
//        /// The magma events along the shortest path in the evolution from `new` to `old`.
//        events: Vec<Event<D>>,
//        /// All other magma events between `new` to `old` if the request `mode` is `AllDescending` or `AllDescending`, or the empty set otherwise.
//        additional_events: Vec<Event<D>>,
//        /// Whether the transmission of the first semigroup value starts at byte zero or at the `offset_value` of the request.
//        is_complete_payload: bool,
//        /// No semigroup values if the request `mode` is `None`, the semigroup values along the shortest path in the evolution between the specified magma events if the `mode` is `Ascending` or `Descending`, or the semigroup values along the longest path in the evolution between the specified magma events if the `mode`  is `AllDescending` or `AllDescending`.
//        values: Vec<S>,
//    },
//}

pub struct UnvalidatedResponseWithRequest<D: Digest, S: Semigroup + CanonicalEncoding> {
    pub request: Request<D>,
    pub response: Response<D, S>,
}

#[derive(Debug, Snafu)]
pub enum ResponseValidationError {
    UnknownEvent,
    ExpectedAtLeastOneEventInEvents,
    FirstEventHashDidNotMatchHashOfRequestNew,
}

impl<D: Digest, S: Semigroup + CanonicalEncoding> TryFrom<UnvalidatedResponseWithRequest<D, S>>
    for ValidResponse<D, S>
{
    type Error = ResponseValidationError;

    fn try_from(value: UnvalidatedResponseWithRequest<D, S>) -> Result<Self, Self::Error> {
        todo!();
        //match value.response{
        //    Response::UnknownEvent => Err(ResponseValidationError::UnknownEvent),
        //    Response::Data{ events, additional_events, is_complete_payload, values } => {
        //        let first_event = events.get(0).context(ExpectedAtLeastOneEventInEvents)?;

        //        //let first_event_actual_digest = D::digest(first_event.encode())
        //        todo!()
        //    }
        //}
        // Regardless of the specifics of the communication protocol, a server sending a `Data` response first transmits the `Events` in order of descending depth. The client hashes the first received magma event and verifies that the resulting digest matches the one it requested. For all further magma events, the client verifies that the depth has the correct value (the correct position in the shortest path in the evolution).
        //
        // Next, the server transmits the `additional_events`, again in order of descending depth. The client again verifies that the depth of each magma event is correct.
        //
        // Furthermore, whenever the client receives a magma event, it verifies that all incoming and outgoing links are consistent. The client computes the hash of the received event and verifies that it matches with the `predecessor_event_link`  or `skip_event_link` value of all known (to the client) magma events that correspond to in-neighbors in the evolution graph. The client does the same for the out-neighbors as well.
        //
        // The server next communicates the value of `is_complete_payload`.
        //
        // Then, the semigroup `values` of the response are transmitted sorted by the order of the depths of the associated magma events. Whether they are sorted in ascending or descending order is specified by the `Mode` of the request. When the client receives a semigroup value, it verifies that its hash and length exactly match the ones given in the corresponding magma event.

        todo!()
    }
}

#[cfg(any(feature = "alloc", feature = "std"))]
pub struct ValidResponse<D: Digest, S: Semigroup> {
    pub events: Vec<Event<D>>,
    pub values: Vec<S>,
}

#[cfg(any(feature = "alloc", feature = "std"))]
impl<D: Digest, S: Semigroup + Clone> ValidResponse<D, S> {
    pub fn combine_values(&self) -> Option<S> {
        combine_all_option(&self.values)
    }
}
