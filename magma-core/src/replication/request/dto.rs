use super::{Ordering, PathLength};
use core::convert::TryFrom;
use digest::Digest;
use serde::{Deserialize, Serialize};
use snafu::Snafu;

#[cfg(feature = "alloc")]
use alloc::vec::Vec;

#[derive(Deserialize, Serialize)]
pub struct Request {
    pub new: Vec<u8>,
    pub old: Vec<u8>,
    pub ordering: Ordering,
    pub path_length: PathLength,
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
