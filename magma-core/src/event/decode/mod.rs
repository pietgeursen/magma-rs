use digest::{generic_array::GenericArray, Digest, Output};
use snafu::{ensure, OptionExt};
use varu64::{decode as varu64_decode, decode_non_zero_u64};

pub mod error;
use crate::Event;
use error::*;

impl<D> Event<D>
where
    D: Digest,
{
    pub fn decode(bytes: &[u8]) -> Result<Self, Error> {
        ensure!(!bytes.is_empty(), DecodeInputIsLengthZero);
        let digest_size = D::output_size();

        // Is a Root
        if bytes[0] == 0 {
            // The first byte is just whether or not it's a Root.
            let bytes = &bytes[1..];

            let (delta_digest, bytes) = Self::decode_digest(bytes, digest_size)?;

            let (size, _) = varu64_decode(bytes).map_err(|(varu_error, _)| {
                Error::DecodeRootSizeFromVaru64 { source: varu_error }
            })?;

            Ok(Self::Root {
                delta_digest,
                delta_size: size,
            })
        } else {
            let (sequence_number, bytes) = decode_non_zero_u64(bytes)
                .map_err(|(err, _)| Error::DecodeSequenceNumberFromVaru64 { source: err })?;

            // Sequence number for a child must be larger >= 2
            ensure!(
                u64::from(sequence_number) >= 2,
                DecodedSequenceNumberForChildWasNotLargerThanOne
            );

            let (predecessor_event_link, bytes) = Self::decode_digest(bytes, digest_size)?;
            let (delta_digest, bytes) = Self::decode_digest(bytes, digest_size)?;

            let (delta_size, bytes) = varu64_decode(bytes)
                .map_err(|(err, _)| Error::DecodeDeltaSizeFromVaru64 { source: err })?;

            // If there are still bytes left then there must be skip link etc.
            // Otherwise we just set skip == delta.
            // TODO I think the Event type should have an option of skips tbh.
            let (skip_event_link, skip_delta_digest, skip_delta_size) = match bytes.len() {
                0 => Ok((
                    predecessor_event_link.clone(),
                    delta_digest.clone(),
                    delta_size,
                )),
                _ => {
                    let (skip_event_link, bytes) = Self::decode_digest(bytes, digest_size)?;
                    let (skip_delta_digest, bytes) = Self::decode_digest(bytes, digest_size)?;
                    let (skip_delta_size, _) = varu64_decode(bytes)
                        .map_err(|(err, _)| Error::DecodeSkipDeltaSizeFromVaru64 { source: err })?;

                    Ok((skip_event_link, skip_delta_digest, skip_delta_size))
                }
            }?;

            Ok(Event::Child {
                sequence_number,
                predecessor_event_link,
                delta_digest,
                delta_size,
                skip_event_link,
                skip_delta_digest,
                skip_delta_size,
            })
        }
    }
    fn decode_digest(
        bytes: &[u8],
        digest_size: usize,
    ) -> Result<(GenericArray<u8, <D as Digest>::OutputSize>, &[u8]), Error> {
        let delta_digest = bytes
            .get(..digest_size)
            .map(Output::<D>::clone_from_slice)
            .context(OutBufferTooSmall)?;
        let bytes = &bytes[digest_size..];
        Ok((delta_digest, bytes))
    }
}
