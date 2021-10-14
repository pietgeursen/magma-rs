#![cfg_attr(not(feature = "std"), no_std)]

pub use digest::{Digest, Output};
pub use frunk::Monoid;
use snafu::{ensure, ResultExt, Snafu};
pub use varu64::{decode as varu64_decode, encode as varu64_encode};

#[derive(Snafu, Debug)]
pub enum Error {
    OutBufferTooSmall,
    DecodeInputIsLengthZero,
    DecodeRootSizeFromVaru64 { varu_error: varu64::DecodeError },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Event<M: Monoid, D: Digest> {
    Root {
        digest: Output<D>,
        size: u64,
    },
    Child {
        digest: Output<D>,
        size: u64,
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
            Self::Root { digest, size } => {
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
                next_byte_num += varu64_encode(*size, &mut out[next_byte_num..]);
                Ok(next_byte_num)
            }
            Self::Child {
                digest,
                size,
                sequence_number,
                predecessor_event_link,
                predecessor_delta,
                predecessor_delta_size,
                skip_event,
                skip_delta,
                skip_delta_size,
            } => {
                let mut next_byte_num = 0;
                unimplemented!()
            }
        }
    }
    pub fn decode(bytes: &[u8]) -> Result<Self, Error> {
        ensure!(bytes.len() > 0, DecodeInputIsLengthZero);

        // Is a Root
        if bytes[0] == 0 {
            let end_of_digest_index = D::output_size() + 1;
            let digest = Output::<D>::clone_from_slice(&bytes[1..end_of_digest_index]);
            let (size, _) = varu64_decode(&bytes[end_of_digest_index..])
                .map_err(|(varu_error, _)| Error::DecodeRootSizeFromVaru64 { varu_error })?;
            Ok(Self::Root { digest, size })
        } else {
            unimplemented!()
        }
    }
    pub fn digest(&self) -> &Output<D> {
        match self {
            Self::Root { digest, .. } => digest,
            Self::Child { digest, .. } => digest,
        }
    }
    pub fn size(&self) -> u64 {
        match self {
            Self::Root { size, .. } => *size,
            Self::Child { size, .. } => *size,
        }
    }

    pub fn encoding_length(&self) -> usize {
        match self {
            Self::Root { digest, size } => 1 + digest.len() + varu64::encoding_length(*size),
            _ => unimplemented!(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::*;
    use blake2::Blake2b;
    use proptest::prelude::*;

    type MyEvent = Event<u64, Blake2b>;

    prop_compose! {
        fn root_event_strategy()(payload in any::<Vec<u8>>()) -> MyEvent{
            let digest = Blake2b::digest(&payload);
            Event::Root{
                digest,
                size: payload.len() as u64
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
        fn last_bytes_of_an_encoded_root_event_contain_size_as_varu64(root_event in root_event_strategy()){
            let digest = root_event.digest();
            let mut buffer = Vec::new();
            buffer.resize(root_event.encoding_length(), 0);
            root_event.encode(&mut buffer).unwrap();

            let mut size_buffer = Vec::new();
            size_buffer.resize(varu64::encoding_length(root_event.size()), 0);
            let (n,_ ) = varu64::decode( &buffer[digest.len() + 1 .. ]).unwrap();

            assert_eq!(n, root_event.size())
        }

        #[test]
        fn encoding_root_event_never_panics_from_incorrect_out_buffer_size(root_event in root_event_strategy(), mut out in any::<Vec<u8>>()){
            let res = root_event.encode(&mut out);
            assert!(res.is_ok() || res.is_err());
        }

        #[test]
        fn encode_decode_root_event(root_event in root_event_strategy()){
            let mut buffer = Vec::new();
            buffer.resize(root_event.encoding_length(), 0);

            root_event.encode(&mut buffer).unwrap();

            let decoded = MyEvent::decode(&buffer).unwrap();

            assert_eq!(decoded.digest(), root_event.digest())
        }
    }
}
