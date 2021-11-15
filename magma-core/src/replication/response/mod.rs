use core::convert::TryFrom;
use snafu::{ensure, AsErrorSource, OptionExt, ResultExt, Snafu};

use digest::Digest;
use frunk::{semigroup::combine_all_option, Semigroup};

use crate::replication::request::Request;
use crate::{CanonicalEncoding, Event};

#[cfg(feature = "alloc")]
use alloc::vec::Vec;

pub mod dto;

#[derive(Debug)]
pub struct EventPayloadPair<D: Digest, S: Semigroup> {
    pub event: Event<D>,
    pub payload: Option<S>,
}

#[derive(Debug)]
pub enum Response<D: Digest, S: Semigroup + CanonicalEncoding> {
    UnknownEvent,
    Data(Vec<EventPayloadPair<D, S>>),
}

#[derive(Debug)]
pub enum UnvalidatedResponse<D: Digest, S: Semigroup + CanonicalEncoding> {
    UnknownEvent,
    Data(Vec<EventPayloadPair<D, S>>),
}

impl<D: Digest, S: Semigroup + CanonicalEncoding> UnvalidatedResponse<D, S> {
    pub fn try_into_valid_response(
        self,
        request: Request<D>,
    ) -> Result<ValidResponse<D, S>, ResponseValidationError> {
        match self {
            Self::UnknownEvent => Err(ResponseValidationError::UnknownEvent),
            Self::Data(pairs) => {
                // TODO !!!! ALL THE LOGIC
                //
                // Regardless of the specifics of the communication protocol, a server sending a `Data` response first transmits the `Events` in order of descending depth. The client hashes the first received magma event and verifies that the resulting digest matches the one it requested. For all further magma events, the client verifies that the depth has the correct value (the correct position in the shortest path in the evolution).
                //
                // Next, the server transmits the `additional_events`, again in order of descending depth. The client again verifies that the depth of each magma event is correct.
                //
                // Furthermore, whenever the client receives a magma event, it verifies that all incoming and outgoing links are consistent. The client computes the hash of the received event and verifies that it matches with the `predecessor_event_link`  or `skip_event_link` value of all known (to the client) magma events that correspond to in-neighbors in the evolution graph. The client does the same for the out-neighbors as well.
                //
                // The server next communicates the value of `is_complete_payload`.
                //
                // Then, the semigroup `values` of the response are transmitted sorted by the order of the depths of the associated magma events. Whether they are sorted in ascending or descending order is specified by the `Mode` of the request. When the client receives a semigroup value, it verifies that its hash and length exactly match the ones given in the corresponding magma event.

                let (events, values) = pairs
                    .into_iter()
                    .map(|pair| (pair.event, pair.payload))
                    .unzip();

                Ok(ValidResponse { events, values })
            }
        }
    }
}

#[cfg(any(feature = "alloc", feature = "std"))]
#[readonly::make]
#[derive(Debug)]
/// A Valid response created by calling [Response.try_into_valid_request]
pub struct ValidResponse<D: Digest, S: Semigroup> {
    pub events: Vec<Event<D>>,
    pub values: Vec<Option<S>>,
}

#[derive(Debug, Snafu)]
pub enum ResponseValidationError {
    UnknownEvent,
    ExpectedAtLeastOneEventInEvents,
    FirstEventHashDidNotMatchHashOfRequestNew,
}
