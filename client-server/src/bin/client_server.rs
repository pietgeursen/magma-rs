use std::convert::TryInto;

use blake2::Blake2b;
use bytes::{Buf, BufMut};
use jsonrpc_core::futures::{self, future, TryFutureExt};
use jsonrpc_core::{BoxFuture, Error, IoHandler, Result};
use jsonrpc_core_client::transports::local;
use jsonrpc_derive::rpc;
use magma_core::replication::request::dto::{Error as DtoConversionError, Request as DtoRequest};
use magma_core::replication::request::{Ordering, PathLength, Request};
use magma_core::replication::response::dto::Response as DtoResponse;
use magma_core::replication::response::{Response, UnvalidatedResponse};
use magma_core::*;
use snafu::{ensure, Snafu};

/// Rpc trait
#[rpc]
pub trait Rpc {
    /// Adds two numbers and returns a result
    #[rpc(name = "request")]
    fn request(&self, request: DtoRequest) -> Result<DtoResponse>;
}

type MyEvent = Event<Blake2b>;
type MyRequest = Request<Blake2b>;

#[derive(Debug)]
struct U32Semigroup(u32);
type MyResponse = Response<Blake2b, U32Semigroup>;
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

    fn decode(mut buffer: &[u8]) -> core::result::Result<(&[u8], Self), Self::Error>
    where
        Self: Sized,
    {
        ensure!(buffer.len() > 4, BufferTooSmall);
        let result = buffer.get_u32();
        Ok((buffer, U32Semigroup(result)))
    }

    fn encoding_length(&self) -> usize {
        4
    }
}

struct RpcImpl;

impl Rpc for RpcImpl {
    fn request(&self, request_dto: DtoRequest) -> Result<DtoResponse> {
        let request: MyRequest = request_dto
            .try_into()
            .map_err(|err: DtoConversionError| Error::invalid_params(err.to_string()))?;

        // handle request

        // The values in the response could be very large so we need to limit copying and
        // allocating when we don't need to. This is less important for the encoded events
        // themselves, they're not that large.
        // Actually, as long as we just move values that's cheap.

        let response: MyResponse = Response::UnknownEvent;
        Ok(response.into())
    }
}

fn main() {
    let mut io = IoHandler::new();
    io.extend_with(RpcImpl.to_delegate());

    let (client, server) = local::connect::<gen_client::Client, _, _>(io);

    let new = Blake2b::digest(b"123");

    let request = MyRequest {
        ordering: Ordering::Ascending,
        path_length: PathLength::ShortestPath,
        old: None,
        include_values: true,
        new,
    };

    let fut = client
        .request(DtoRequest::from_request(&request))
        .map_ok(|res| {
            // TODO: hide this stuff in internals
            println!("{:?}", res);
            let res: MyUnvalidatedResponse = res.try_into().unwrap();
            println!("{:?}", res);
            let err = res.try_into_valid_response(request);
            println!("{:?}", err);
        });
    futures::executor::block_on(async move { futures::join!(fut, server) })
        .0
        .unwrap();
}
