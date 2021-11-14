use core::convert::TryFrom;
use digest::Digest;
use frunk::Semigroup;
use serde::{Deserialize, Serialize};
use snafu::Snafu;

use crate::CanonicalEncoding;

#[cfg(feature = "alloc")]
use alloc::vec::Vec;

/// A Data Transfer Object representation of a [super::Response].
#[derive(Deserialize, Serialize)]
pub enum Response {
    UnknownEvent,
    Data {
        events: Vec<Vec<u8>>,
        additional_events: Vec<Vec<u8>>,
        is_complete_payload: bool,
        values: Vec<Vec<u8>>,
    },
}

#[derive(Snafu, Debug)]
pub enum Error {}

impl<D, S> TryFrom<Response> for super::Response<D, S>
where
    D: Digest,
    S: Semigroup + CanonicalEncoding,
{
    type Error = Error;

    fn try_from(value: Response) -> Result<Self, Self::Error> {
        // S needs to be convertable from / bytes. Not sure if this should SerDe is what we want,
        // or encode / decode or fromBytes / toBytes
        // Actually I think S must be constrained
        todo!()
    }
}

impl<'a, D, S> From<super::Response<D, S>> for Response
where
    D: Digest,
    S: Semigroup + CanonicalEncoding,
{
    fn from(_: super::Response<D, S>) -> Self {
        todo!()
    }
}
