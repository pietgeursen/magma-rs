#![cfg_attr(not(feature = "std"), no_std)]

pub use core::num::NonZeroU64;
pub use digest::{Digest, Output, generic_array::GenericArray};
pub use frunk::Semigroup;
use snafu::{ensure, OptionExt, Snafu};
pub use varu64::{
    decode as varu64_decode, decode_non_zero_u64, encode as varu64_encode, encode_non_zero_u64,
};

#[derive(Snafu, Debug)]
pub enum Error {
    OutBufferTooSmall,
    DecodeInputIsLengthZero,
    DecodeRootSizeFromVaru64 { source: varu64::DecodeError },
    DecodeSequenceNumberFromVaru64 { source: varu64::DecodeError },
    DecodeDeltaSizeFromVaru64 { source: varu64::DecodeError },
    DecodeSkipDeltaSizeFromVaru64 { source: varu64::DecodeError },
    DecodedSequenceNumberForChildWasNotLargerThanOne,
}

//pub trait Encode {
//    type Error;
//    fn encode(&self, out: &mut [u8]) -> Result<usize, Self::Error>;
//    fn encoding_length(&self) -> usize;
//}
//
//pub enum UnpublishedEvent<'a, 'b, 'c, S: Semigroup, D: Digest> {
//    Root {
//        semigroup_value: &'a S,
//    },
//    Child {
//        semigroup_value: &'a S,
//        predecessor_event: &'b Event<D>,
//        skip_event: &'c Event<D>,
//    },
//}
//
//impl<'a, 'b, 'c, S, D> TryFrom<UnpublishedEvent<'a, 'b, 'c, S, D>> for Event<D>
//where
//    S: Semigroup + Sized + Encode,
//    D: Digest,
//{
//    type Error = Error;
//
//    fn try_from(
//        unpublished_event: UnpublishedEvent<'a, 'b, 'c, S, D>,
//    ) -> Result<Self, <Self as TryFrom<UnpublishedEvent<'a, 'b, 'c, S, D>>>::Error> {
//        match unpublished_event {
//            UnpublishedEvent::Root { semigroup_value } => Event::Root {
//                size: core::mem::size_of_val(semigroup_value) as u64,
//            },
//            UnpublishedEvent::Child {
//                semigroup_value,
//                predecessor_event,
//                skip_event,
//            } => {}
//        }
//    }
//}

#[derive(Clone, Debug)]
pub enum Event<D: Digest>
where
    D: Digest,
{
    Root {
        delta_digest: Output<D>,
        delta_size: u64,
    },
    Child {
        sequence_number: NonZeroU64, // The first **child** sequence_number starts at **2**

        delta_digest: Output<D>,
        delta_size: u64,
        predecessor_event_link: Output<D>,

        skip_event_link: Output<D>, // the skip event, None if this is the first event
        skip_delta_digest: Output<D>, // change compared to the skip event
        skip_delta_size: u64,       // size in bytes of this.skip_delta
    },
}

impl<D> PartialEq for Event<D>
where
    D: Digest,
{
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (
                Self::Root {
                    delta_digest: l_delta_digest,
                    delta_size: l_delta_size,
                },
                Self::Root {
                    delta_digest: r_delta_digest,
                    delta_size: r_delta_size,
                },
            ) => l_delta_digest == r_delta_digest && l_delta_size == r_delta_size,
            (
                Self::Child {
                    sequence_number: l_sequence_number,
                    delta_digest: l_delta_digest,
                    delta_size: l_delta_size,
                    predecessor_event_link: l_predecessor_event_link,
                    skip_event_link: l_skip_event_link,
                    skip_delta_digest: l_skip_delta_digest,
                    skip_delta_size: l_skip_delta_size,
                },
                Self::Child {
                    sequence_number: r_sequence_number,
                    delta_digest: r_delta_digest,
                    delta_size: r_delta_size,
                    predecessor_event_link: r_predecessor_event_link,
                    skip_event_link: r_skip_event_link,
                    skip_delta_digest: r_skip_delta_digest,
                    skip_delta_size: r_skip_delta_size,
                },
            ) => {
                l_sequence_number == r_sequence_number
                    && l_delta_digest == r_delta_digest
                    && l_delta_size == r_delta_size
                    && l_predecessor_event_link == r_predecessor_event_link
                    && l_skip_event_link == r_skip_event_link
                    && l_skip_delta_digest == r_skip_delta_digest
                    && l_skip_delta_size == r_skip_delta_size
            }
            _ => false,
        }
    }
}

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
    pub fn decode(bytes: &[u8]) -> Result<Self, Error> {
        ensure!(bytes.len() > 0, DecodeInputIsLengthZero);
        let digest_size = D::output_size();

        // Is a Root
        if bytes[0] == 0 {
            // The first byte is just whether or not it's a Root.
            let bytes = &bytes[1..];

            let (delta_digest, bytes) = Self::decode_digest(bytes, digest_size)?;

            let (size, _) = varu64_decode(&bytes).map_err(|(varu_error, _)| {
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
    pub fn digest(&self) -> &Output<D> {
        match self {
            Self::Root {
                delta_digest: digest,
                ..
            } => digest,
            Self::Child {
                delta_digest: digest,
                ..
            } => digest,
        }
    }
    pub fn size(&self) -> u64 {
        match self {
            Self::Root {
                delta_size: size, ..
            } => *size,
            Self::Child {
                delta_size: size, ..
            } => *size,
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

    fn decode_digest<'a>(bytes: &'a[u8], digest_size: usize) -> Result<(GenericArray<u8, <D as Digest>::OutputSize>, &'a[u8]), Error> {
        let delta_digest = bytes
            .get(..digest_size)
            .map(Output::<D>::clone_from_slice)
            .context(OutBufferTooSmall)?;
        let bytes = &bytes[digest_size..];
        Ok((delta_digest, bytes))
    }

}


#[cfg(test)]
mod tests {
    use crate::*;
    use blake2::Blake2b;
    use proptest::prelude::*;

    type MyEvent = Event<Blake2b>;

    prop_compose! {
        fn root_event_strategy()(payload in any::<Vec<u8>>()) -> MyEvent{
            let delta_digest = Blake2b::digest(&payload);
            Event::Root{
                delta_digest,
                delta_size: payload.len() as u64
            }
        }
    }
    prop_compose! {
        fn encoded_root_event_strategy()(root_event in root_event_strategy()) -> Vec<u8> {
            let mut buffer = Vec::new();
            buffer.resize(root_event.encoding_length(), 0);

            root_event.encode(&mut buffer).unwrap();

            buffer
        }
    }
    prop_compose! {
        fn digested_root_event_strategy()(root_event in encoded_root_event_strategy()) -> Output<Blake2b> {
            Blake2b::digest(&root_event)
        }
    }

    prop_compose! {
        fn digested_root_event_strategy_one_byte_different()(root_event in digested_root_event_strategy()) -> Output<Blake2b> {
            let mut event = root_event.clone();
            event[0] ^= 1;
            event
        }
    }

    prop_compose! {
        fn valid_sequence_number()(n in any::<u64>())-> NonZeroU64{
            NonZeroU64::new(n).unwrap_or(NonZeroU64::new(2).unwrap())
        }
    }
    prop_compose! {
        fn child_with_skip_same_as_predecessor_event_strategy()(payload in any::<Vec<u8>>(), sequence_number in valid_sequence_number(), digested_root_event in digested_root_event_strategy()) -> MyEvent{
            let delta_digest = Blake2b::digest(&payload);
            Event::Child{
                sequence_number,
                delta_digest,
                delta_size: payload.len() as u64,
                predecessor_event_link: digested_root_event,
                skip_event_link: digested_root_event,
                skip_delta_digest: delta_digest,
                skip_delta_size: payload.len() as u64
            }
        }
    }
    prop_compose! {
        fn child_event_strategy()(payload in any::<Vec<u8>>(), payload_two in any::<Vec<u8>>(), sequence_number in valid_sequence_number(), predecessor_event_link in digested_root_event_strategy(), skip_event_link in digested_root_event_strategy_one_byte_different()) -> MyEvent{

            let delta_digest = Blake2b::digest(&payload);
            let skip_delta_digest = Blake2b::digest(&payload_two);

            Event::Child{
                sequence_number,
                delta_digest,
                delta_size: payload.len() as u64,
                predecessor_event_link,
                skip_event_link,
                skip_delta_digest,
                skip_delta_size: payload_two.len() as u64
            }
        }
    }

    fn random_event_stratedy() -> BoxedStrategy<MyEvent> {
        prop_oneof![
            root_event_strategy(),
            child_with_skip_same_as_predecessor_event_strategy(),
            child_event_strategy()
        ]
        .boxed()
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
        fn encode_decode_event(event in random_event_stratedy()){
            let mut buffer = Vec::new();
            buffer.resize(event.encoding_length(), 0);

            let encoded_size = event.encode(&mut buffer).unwrap();

            let decoded = MyEvent::decode(&buffer[..encoded_size]).unwrap();

            assert_eq!(event, decoded);
        }

        #[test]
        fn encoding_never_panics_from_incorrect_out_buffer_size(event in random_event_stratedy(), mut out in any::<Vec<u8>>()){
            let res = event.encode(&mut out);
            assert!(res.is_ok() || res.is_err());
        }

        #[test]
        fn decoding_never_panics_from_incorrect_out_buffer_size(event in random_event_stratedy(), truncation_amount in 1..1000usize){
            let mut buffer = Vec::new();
            buffer.resize(event.encoding_length(), 0);
            let encoded_size = event.encode(&mut buffer).unwrap();

            let res = MyEvent::decode(&buffer[.. std::cmp::min(encoded_size, truncation_amount)]);

            assert!(res.is_ok() || res.is_err());
        }

    }
}
