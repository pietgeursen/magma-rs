use core::convert::TryFrom;
use core::num::NonZeroU64;
use serde::{Deserialize, Serialize};
use snafu::{Snafu, ensure};
use digest::Digest;

use crate::Event as ValidEvent;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum Event<'a> {
    Root {
        delta_digest: &'a [u8],
        delta_size: u64,
    },
    Child {
        sequence_number: NonZeroU64, // The first **child** sequence_number starts at **2**

        predecessor_event_link: &'a [u8],
        delta_digest: &'a [u8],
        delta_size: u64,

        skip_event_link: &'a [u8], // the skip event, None if this is the first event
        skip_delta_digest: &'a [u8], // change compared to the skip event
        skip_delta_size: u64,      // size in bytes of this.skip_delta
    },
}

#[derive(Snafu, Debug)]
pub enum DtoConversionError {
    InvalidSequenceNumber

}

impl<'a, D: Digest> TryFrom<Event<'a>> for ValidEvent<D> {
    type Error = DtoConversionError;

    fn try_from(value: Event<'a>) -> Result<Self, Self::Error> {
        match value{
            Event::Root { delta_digest, delta_size } => {

            },
            Event::Child { sequence_number, predecessor_event_link, delta_digest, delta_size, skip_event_link, skip_delta_digest, skip_delta_size } => {
                ensure!(sequence_number.get() >= 2u64, InvalidSequenceNumber);

            },
        }
        todo!()
    }
}
