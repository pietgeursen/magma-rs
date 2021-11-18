#![no_main]
use bytes::{Buf, BufMut};
use blake2::Blake2b;
use libfuzzer_sys::fuzz_target;
use magma_core::replication::response::dto::{EventPayloadPair, Response as ResponseDto};
use magma_core::replication::response::{Response, UnvalidatedResponse};
use magma_core::*;
use std::convert::TryInto;
use snafu::{Snafu, ensure, ResultExt};


#[derive(arbitrary::Arbitrary, Debug)]
struct ArbEventPayloadPair {
    pub event: Vec<u8>,
    pub payload: Option<Vec<u8>>,
}
#[derive(arbitrary::Arbitrary, Debug)]
enum ArbResponse {
    UnknownEvent,
    Data(Vec<ArbEventPayloadPair>),
}

fuzz_target!(|arb_response: ArbResponse| {
    let response_dto = match arb_response {
        ArbResponse::UnknownEvent => ResponseDto::UnknownEvent,
        ArbResponse::Data(data) => {
            let res = data.into_iter().map(|pair| EventPayloadPair {
                event: pair.event,
                payload: pair.payload,
            }).collect();
            ResponseDto::Data(res)
        }
    };

    let _: Result<MyUnvalidatedResponse, _> = response_dto.try_into();
});
#[derive(Debug)]
struct U32Semigroup(u32);
type MyUnvalidatedResponse = UnvalidatedResponse<Blake2b, U32Semigroup>;

#[derive(Snafu, Debug)]
enum CanonicalEncodingU32Error {
    BufferTooSmall,
}

impl Semigroup for U32Semigroup {
    fn combine(&self, other: &Self) -> Self {
        U32Semigroup(self.0.combine(&other.0))
    }
}

impl CanonicalEncoding for U32Semigroup {
    type Error = CanonicalEncodingU32Error;

    fn encode(&self, mut buffer: &mut [u8]) -> core::result::Result<usize, Self::Error> {
        ensure!(buffer.len() > 4, BufferTooSmall);
        buffer.put_u32(self.0);
        Ok(4)
    }

    fn decode(mut buffer: &[u8]) -> core::result::Result<(Self, &[u8]), Self::Error>
    where
        Self: Sized,
    {
        ensure!(buffer.len() > 4, BufferTooSmall);
        let result = buffer.get_u32();
        Ok((U32Semigroup(result), buffer))
    }

    fn encoding_length(&self) -> usize {
        4
    }
}
