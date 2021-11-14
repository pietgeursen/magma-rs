pub mod decode;
pub mod dto;
pub mod encode;

pub use core::num::NonZeroU64;
pub use digest::{generic_array::GenericArray, Digest, Output};
pub use frunk::Semigroup;

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

        predecessor_event_link: Output<D>,
        delta_digest: Output<D>,
        delta_size: u64,

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
    pub fn delta_digest(&self) -> &Output<D> {
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

            let digest = root_event.delta_digest();
            assert_eq!(&buffer[1..digest.len() + 1], digest.as_slice())
        }

        #[test]
        fn last_bytes_of_an_encoded_root_event_contain_size_as_varu64(root_event in root_event_strategy()){
            let digest = root_event.delta_digest();
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
