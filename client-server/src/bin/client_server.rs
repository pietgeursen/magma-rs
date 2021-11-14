use std::convert::TryInto;

use blake2::Blake2b;
use jsonrpc_core::futures::{self, future, TryFutureExt};
use jsonrpc_core::{BoxFuture, Error, IoHandler, Result};
use jsonrpc_core_client::transports::local;
use jsonrpc_derive::rpc;
use magma_core::replication::request::dto::{Error as DtoConversionError, Request as DtoRequest};
use magma_core::replication::request::Request;
use magma_core::replication::response::dto::Response as DtoResponse;
use magma_core::replication::response::Response;
use magma_core::*;
use snafu::Snafu;

/// Rpc trait
#[rpc]
pub trait Rpc {
    /// Returns a protocol version
    #[rpc(name = "protocolVersion")]
    fn protocol_version(&self) -> Result<String>;

    /// Adds two numbers and returns a result
    #[rpc(name = "add", alias("callAsyncMetaAlias"))]
    fn add(&self, a: u64, b: u64) -> Result<u64>;

    /// Adds two numbers and returns a result
    #[rpc(name = "request")]
    fn request(&self, request: DtoRequest) -> Result<DtoResponse>;

    /// Performs asynchronous operation
    #[rpc(name = "callAsync")]
    fn call(&self, a: u64) -> BoxFuture<Result<String>>;
}

type MyEvent = Event<Blake2b>;
type MyRequest = Request<Blake2b>;

struct U32Semigroup(u32);
type MyResponse = Response<Blake2b, U32Semigroup>;

#[derive(Snafu, Debug)]
enum CanonicalEncodingU32Error {}

impl Semigroup for U32Semigroup {
    fn combine(&self, other: &Self) -> Self {
        U32Semigroup(self.0.combine(&other.0))
    }
}

impl CanonicalEncoding for U32Semigroup {
    type Error = CanonicalEncodingU32Error;

    fn encode(buffer: &mut [u8]) -> core::result::Result<usize, Self::Error> {
        todo!()
    }

    fn decode(buffer: &[u8]) -> core::result::Result<(usize, Self), Self::Error>
    where
        Self: Sized,
    {
        todo!()
    }
}

struct RpcImpl;

impl Rpc for RpcImpl {
    fn protocol_version(&self) -> Result<String> {
        Ok("version1".into())
    }

    fn add(&self, a: u64, b: u64) -> Result<u64> {
        Ok(a + b)
    }

    fn call(&self, _: u64) -> BoxFuture<Result<String>> {
        Box::pin(future::ready(Ok("OK".to_owned())))
    }

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
    let fut = client.add(5, 6).map_ok(|res| println!("5 + 6 = {}", res));
    futures::executor::block_on(async move { futures::join!(fut, server) })
        .0
        .unwrap();
}
