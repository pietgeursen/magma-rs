use core::convert::TryFrom;
use serde::{Serialize, Deserialize};
use digest::Digest;
use frunk::Semigroup;
use snafu::Snafu;

/// A Data Transfer Object representation of a [super::Response].
#[cfg_attr(any(feature = "alloc", feature = "std"), derive(Deserialize, Serialize))]
pub enum Response{
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
    S: Semigroup,
{
    type Error = Error;

    fn try_from(value: Response) -> Result<Self, Self::Error> {
        todo!()
    }
}

impl<'a, D, S> From<super::Response<D,S>> for Response
where
    D: Digest,
    S: Semigroup,
{
    fn from(_: super::Response<D,S>) -> Self {
        todo!()
    }
}
