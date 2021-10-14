#![cfg_attr(not(feature = "std"), no_std)]

pub use digest::{Digest, Output};
pub use frunk::Monoid;
use snafu::{ensure, Snafu};
pub use varu64::{decode as varu64_decode, encode as varu64_encode};

#[derive(Snafu, Debug)]
pub enum Error {
    OutBufferTooSmall,
}

#[derive(Clone, Debug)]
pub enum Event<M: Monoid, D: Digest> {
    Root {
        digest: Output<D>,
    },
    Child {
        digest: Output<D>,
        sequence_number: u64, // The first **child** sequence_number starts at **2**

        predecessor_event_link: Output<D>,
        predecessor_delta: M,        // change compared to the predecessor event
        predecessor_delta_size: u64, // size in bytes of this.predecessor_delta

        skip_event: Output<D>, // the skip event, None if this is the first event
        skip_delta: M,         // change compared to the skip event
        skip_delta_size: u64,  // size in bytes of this.skip_delta
    },
}

impl<M, D> Event<M, D>
where
    M: Monoid,
    D: Digest,
{
    pub fn encode(&self, out: &mut [u8]) -> Result<usize, Error> {
        ensure!(out.len() >= self.encoding_length(), OutBufferTooSmall);

        match self {
            Self::Root { digest } => {
                let mut next_byte_num = 0;

                // If it is a RootEvent the encoding consists of a zero-byte
                out[next_byte_num] = 0;
                next_byte_num += 1;

                // Followed by the digest
                let digest_bytes = digest.as_slice();
                out[next_byte_num..digest_bytes.len() + next_byte_num]
                    .copy_from_slice(digest_bytes);
                next_byte_num += digest_bytes.len();

                // Followed by the digest length
                next_byte_num +=
                    varu64_encode(digest_bytes.len() as u64, &mut out[next_byte_num..]);
                Ok(next_byte_num)
            }
            _ => unimplemented!(),
        }
    }
    pub fn digest(&self) -> &Output<D> {
        match self {
            Self::Root { digest } => digest,
            Self::Child { digest, .. } => digest,
        }
    }
    pub fn encoding_length(&self) -> usize {
        match self {
            Self::Root { digest } => {
                1 + digest.len() + varu64::encoding_length(digest.len() as u64)
            }
            _ => unimplemented!(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::*;
    use blake2::Blake2b;
    use proptest::prelude::*;

    prop_compose! {
        fn root_event_strategy()(payload in any::<Vec<u8>>()) -> Event<u64, Blake2b>{
            let digest = Blake2b::digest(&payload);
            Event::Root{
                digest
            }
        }
    }

    proptest! {
        #[test]
        fn first_byte_of_an_encoded_root_event_is_zero(root_event in root_event_strategy() ){
            let mut buffer = Vec::new();
            buffer.resize(root_event.encoding_length(), 0);

            root_event.encode(&mut buffer).unwrap();

            assert_eq!(buffer[0], 0)
        }

        #[test]
        fn next_bytes_of_an_encoded_root_event_contain_digest(root_event in root_event_strategy()){
            let mut buffer = Vec::new();
            buffer.resize(root_event.encoding_length(), 0);
            root_event.encode(&mut buffer).unwrap();

            let digest = root_event.digest();
            assert_eq!(&buffer[1..digest.len() + 1], digest.as_slice())
        }

        #[test]
        fn last_bytes_of_an_encoded_root_event_contain_digest_len_as_varu64(root_event in root_event_strategy()){
            let mut buffer = Vec::new();
            buffer.resize(root_event.encoding_length(), 0);
            root_event.encode(&mut buffer).unwrap();

            let digest = root_event.digest();
            let mut encoded_digest_len = [0;16];
            let n = varu64::encode(digest.len() as u64, &mut encoded_digest_len);
            assert_eq!(&buffer[digest.len() + 1 .. digest.len() + 1 + n], &encoded_digest_len[..n])
        }

        #[test]
        fn encoding_root_event_never_panics_from_incorrect_out_buffer_size(root_event in root_event_strategy(), mut out in any::<Vec<u8>>()){
            let res = root_event.encode(&mut out);
            assert!(res.is_ok() || res.is_err());
        }

    }
}
