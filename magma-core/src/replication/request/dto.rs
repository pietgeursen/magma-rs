use super::Mode;
use core::convert::TryFrom;
use digest::Digest;
use serde::{Deserialize, Serialize};
use snafu::Snafu;

#[derive(Deserialize, Serialize)]
/// A Data Transfer Object representation of a [super::Request].
pub struct RequestRef<'a> {
    pub new: &'a [u8],
    pub old: &'a [u8],
    pub mode: Mode,
    pub offset_event: u8,
    pub offset_value: Option<u8>,
}

#[derive(Deserialize, Serialize)]
pub struct Request {
    pub new: Vec<u8>,
    pub old: Vec<u8>,
    pub mode: Mode,
    pub offset_event: u8,
    pub offset_value: Option<u8>,
}
#[derive(Snafu, Debug, Deserialize, Serialize)]
pub enum Error {}

impl<'a, D> TryFrom<Request> for super::Request<D>
where
    D: Digest,
{
    type Error = Error;

    fn try_from(value: Request) -> Result<Self, Self::Error> {
        todo!()
    }
}
