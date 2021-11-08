use std::convert::TryFrom;
use snafu::Snafu;

use digest::Digest;
use frunk::{Semigroup, semigroup::combine_all_option};

use crate::Event;
use crate::replication::request::Request;

pub mod dto;

/// The data clients query for.
pub enum ResponseRef<'a, 'b, D: Digest, S: Semigroup> {
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

#[cfg(any(feature = "alloc", feature = "std"))]
pub enum Response<D: Digest, S: Semigroup> {
    /// The server cannot serve a meaningful response, because it does not know about `old` or
    /// `new`.
    UnknownEvent,
    /// The magma events and semigroup values the client requested.
    Data {
        /// The magma events along the shortest path in the evolution from `new` to `old`.
        events: Vec<Event<D>>,
        /// All other magma events between `new` to `old` if the request `mode` is `AllDescending` or `AllDescending`, or the empty set otherwise.
        additional_events: Vec<Event<D>>,
        /// Whether the transmission of the first semigroup value starts at byte zero or at the `offset_value` of the request.
        is_complete_payload: bool,
        /// No semigroup values if the request `mode` is `None`, the semigroup values along the shortest path in the evolution between the specified magma events if the `mode` is `Ascending` or `Descending`, or the semigroup values along the longest path in the evolution between the specified magma events if the `mode`  is `AllDescending` or `AllDescending`.
        values: Vec<S>,
    },
}

pub struct UnvalidatedResponse<D: Digest, S: Semigroup>{
    pub request: Request<D>,
    pub response: Response<D, S>
}

#[derive(Debug, Snafu)]
pub enum ResponseValidationError{}

impl<D: Digest, S: Semigroup> TryFrom<UnvalidatedResponse<D, S>> for ValidResponse<D, S>{
    type Error = ResponseValidationError;

    fn try_from(value: UnvalidatedResponse<D, S>) -> Result<Self, Self::Error> {
        todo!()
    }
}

#[cfg(any(feature = "alloc", feature = "std"))]
pub struct ValidResponse<D: Digest, S: Semigroup> {
    pub events: Vec<Event<D>>,
    pub values: Vec<S>,
}

impl<D: Digest, S: Semigroup + Clone> ValidResponse<D,S>{
    pub fn combine_values(&self) -> Option<S>{
        combine_all_option(&self.values)
    }
}

