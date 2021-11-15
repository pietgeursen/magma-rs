use core::convert::TryFrom;
use digest::Digest;
use frunk::Semigroup;
use serde::{Deserialize, Serialize};
use snafu::{AsErrorSource, ResultExt, Snafu};

use crate::{CanonicalEncoding, Event};

#[cfg(feature = "alloc")]
use alloc::vec::Vec;

#[derive(Deserialize, Serialize, Debug)]
pub struct EventPayloadPair {
    pub event: Vec<u8>,
    pub payload: Option<Vec<u8>>,
}

/// A Data Transfer Object representation of a [super::Response].
#[derive(Deserialize, Serialize, Debug)]
pub enum Response {
    UnknownEvent,
    Data(Vec<EventPayloadPair>),
}

#[derive(Snafu, Debug)]
pub enum Error<E: AsErrorSource + core::fmt::Display> {
    DecodeEvent {
        source: crate::event::decode::error::Error,
    },
    DecodePayload {
        source: E,
    },
}

impl<D, S> TryFrom<Response> for super::UnvalidatedResponse<D, S>
where
    D: Digest,
    S: Semigroup + CanonicalEncoding,
    <S as CanonicalEncoding>::Error: AsErrorSource + core::fmt::Display,
{
    type Error = Error<S::Error>;

    // Decode from a dto to an UnvalidatedResponse
    fn try_from(response: Response) -> Result<Self, Self::Error> {
        match response {
            Response::UnknownEvent => Ok(Self::UnknownEvent),
            Response::Data(pairs) => {
                let new_pairs = pairs
                    .iter()
                    .map(|pair| {
                        let event = Event::decode(&pair.event).context(DecodeEvent)?;

                        let payload = pair
                            .payload
                            .as_ref()
                            .map(|payload| {
                                let (res, _) = S::decode(&payload).context(DecodePayload)?;
                                Ok(res)
                            })
                            .transpose()?;

                        Ok(super::EventPayloadPair { event, payload })
                    })
                    .collect::<Result<Vec<_>, Self::Error>>()?;

                Ok(Self::Data(new_pairs))
            }
        }
    }
}

impl<'a, D, S> From<super::Response<D, S>> for Response
where
    D: Digest,
    S: Semigroup + CanonicalEncoding,
{
    fn from(response: super::Response<D, S>) -> Self {
        match response {
            super::Response::UnknownEvent => Self::UnknownEvent,
            super::Response::Data(pairs) => {
                let new_pairs = pairs.iter().map(|pair|{
                    let mut event = Vec::with_capacity(pair.event.encoding_length());

                    pair.event.encode(&mut event).expect("Encoding event failed unexpectedly");

                    let payload = pair.payload.as_ref().map(|payload|{
                        let mut vec = Vec::with_capacity(payload.encoding_length());
                        vec.resize(payload.encoding_length(), 0);

                        // This shouldn't fail unless the payload.encoding_length is buggy
                        payload.encode(&mut vec).expect("Encoding Semigroup value failed unexpectedly. Is payload.encoding_length buggy?");
                        vec
                    });
                    EventPayloadPair{
                        event,
                        payload
                    }
                })
                .collect();

                Self::Data(new_pairs)
            }
        }
    }
}
