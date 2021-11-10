use digest::Digest;
use snafu::ensure;
use varu64::{encode as varu64_encode, encode_non_zero_u64};

pub mod error;
use crate::Event;
use error::*;

impl<D> Event<D>
where
    D: Digest,
{
    pub fn encode(&self, out: &mut [u8]) -> Result<usize, Error> {
        ensure!(out.len() >= self.encoding_length(), OutBufferTooSmall);

        match self {
            Self::Root {
                delta_digest,
                delta_size,
            } => {
                let mut next_byte_num = 0;

                // If it is a RootEvent the encoding consists of a zero-byte
                out[next_byte_num] = 0;
                next_byte_num += 1;

                // Followed by the delta digest
                let digest_bytes = delta_digest.as_slice();
                out[next_byte_num..digest_bytes.len() + next_byte_num]
                    .copy_from_slice(digest_bytes);
                next_byte_num += digest_bytes.len();

                // Followed by the delta size
                next_byte_num += varu64_encode(*delta_size, &mut out[next_byte_num..]);
                Ok(next_byte_num)
            }
            Self::Child {
                delta_digest,
                delta_size,
                sequence_number,
                predecessor_event_link,
                skip_event_link,
                skip_delta_digest,
                skip_delta_size,
            } => {
                let mut next_byte_num = 0;

                // Sequence number
                next_byte_num += encode_non_zero_u64(*sequence_number, &mut out[next_byte_num..]);

                // Followed by predecessor_event_link
                out[next_byte_num..predecessor_event_link.len() + next_byte_num]
                    .copy_from_slice(&predecessor_event_link);
                next_byte_num += predecessor_event_link.len();

                // Followed by the delta digest
                let digest_bytes = delta_digest.as_slice();
                out[next_byte_num..digest_bytes.len() + next_byte_num]
                    .copy_from_slice(digest_bytes);
                next_byte_num += digest_bytes.len();

                // Followed by the delta size
                next_byte_num += varu64_encode(*delta_size, &mut out[next_byte_num..]);

                if skip_event_link != predecessor_event_link {
                    // Followed by skip_event_link
                    out[next_byte_num..skip_event_link.len() + next_byte_num]
                        .copy_from_slice(&skip_event_link);
                    next_byte_num += skip_event_link.len();

                    // Followed by the skip_delta digest
                    let digest_bytes = skip_delta_digest.as_slice();
                    out[next_byte_num..digest_bytes.len() + next_byte_num]
                        .copy_from_slice(digest_bytes);
                    next_byte_num += digest_bytes.len();

                    // Followed by the skip_delta size
                    next_byte_num += varu64_encode(*skip_delta_size, &mut out[next_byte_num..]);
                }

                Ok(next_byte_num)
            }
        }
    }
    pub fn encoding_length(&self) -> usize {
        match self {
            Self::Root {
                delta_digest: digest,
                delta_size: size,
            } => 1 + digest.len() + varu64::encoding_length(*size),
            Self::Child {
                delta_digest,
                delta_size,
                sequence_number,
                predecessor_event_link,
                skip_event_link,
                skip_delta_size,
                skip_delta_digest,
            } => {
                delta_digest.len()
                    + varu64::encoding_length(*delta_size)
                    + varu64::encoding_length_non_zero_u64(*sequence_number)
                    + predecessor_event_link.len()
                    + skip_event_link.len()
                    + skip_delta_digest.len()
                    + varu64::encoding_length(*skip_delta_size)
            }
        }
    }
}
