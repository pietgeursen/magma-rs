use crate::event::dto::Event;
use core::convert::TryFrom;
use digest::Digest;
use frunk::Semigroup;
use snafu::Snafu;

#[cfg_attr(feature = "alloc", derive(Deserialize, Serialize))]
/// A Data Transfer Object representation of a [super::Response].
pub enum Response<'a> {
    UnknownEvent,
    Data {
        events: Vec<Event<'a>>,
        additional_events: Vec<Event<'a>>,
        is_complete_payload: bool,
        values: Vec<&'a [u8]>,
    },
}

#[derive(Snafu, Debug)]
pub enum Error {}

impl<'a, D, S> TryFrom<Response<'a>> for super::Response<'a, 'a, D, S>
where
    D: Digest,
    S: Semigroup,
{
    type Error = Error;

    fn try_from(value: Response<'a>) -> Result<Self, Self::Error> {
        todo!()
    }
}
