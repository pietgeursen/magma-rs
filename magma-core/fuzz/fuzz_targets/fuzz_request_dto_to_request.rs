#![no_main]

use blake2::Blake2b;
use libfuzzer_sys::fuzz_target;
use magma_core::replication::request::dto::Request as RequestDto;
use magma_core::replication::request::{Ordering, PathLength, Request};
use std::convert::TryInto;

type MyRequest = Request<Blake2b>;

#[derive(arbitrary::Arbitrary, Debug)]
struct ArbRequest {
    pub new: Vec<u8>,
    pub old: Option<Vec<u8>>,
    pub include_values: bool,
}

fuzz_target!(|arb_request: ArbRequest| {
    let request_dto = RequestDto {
        new: arb_request.new,
        old: arb_request.old,
        include_values: arb_request.include_values,
        ordering: Ordering::Ascending,
        path_length: PathLength::ShortestPath,
    };

    let _request: Result<MyRequest, _> = request_dto.try_into();
});
