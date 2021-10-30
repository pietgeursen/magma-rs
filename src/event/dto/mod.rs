use core::convert::TryFrom;
use core::num::NonZeroU64;
use digest::{generic_array::GenericArray, Digest};
use serde::{Deserialize, Serialize};
use snafu::{ensure, Snafu};

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
pub enum Error {
    #[snafu(display(
        "Encoded Event had an an invalid sequence number. Child events must have an event >= 2"
    ))]
    InvalidSequenceNumber,

    #[snafu(display(
        "Encoded Event had an an invalid length digest. Expected length: {}, actual length: {}",
        expected_length,
        actual_length
    ))]
    InvalidDigestLength {
        expected_length: usize,
        actual_length: usize,
    },
}

impl<'a, D: Digest> TryFrom<Event<'a>> for ValidEvent<D> {
    type Error = Error;

    fn try_from(value: Event<'a>) -> Result<Self, Self::Error> {
        match value {
            Event::Root {
                delta_digest,
                delta_size,
            } => {
                let digest = try_convert_slice_to_digest(delta_digest)?;

                let evt = ValidEvent::Root {
                    delta_digest: digest,
                    delta_size,
                };
                Ok(evt)
            }
            Event::Child {
                sequence_number,
                predecessor_event_link,
                delta_digest,
                delta_size,
                skip_event_link,
                skip_delta_digest,
                skip_delta_size,
            } => {
                ensure!(sequence_number.get() >= 2u64, InvalidSequenceNumber);
            }
        }
    }
}

fn try_convert_slice_to_digest<D: Digest>(
    delta_digest: &[u8],
) -> Result<GenericArray<u8, <D as Digest>::OutputSize>, Error> {
    let actual_length = delta_digest.len();
    let expected_length = D::output_size();
    ensure!(
        actual_length == expected_length,
        InvalidDigestLength {
            expected_length,
            actual_length
        }
    );
    let digest = GenericArray::clone_from_slice(delta_digest);
    Ok(digest)
}
