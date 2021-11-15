use super::{Ordering, PathLength};
use core::convert::TryFrom;
use digest::{Digest, Output};
use serde::{Deserialize, Serialize};
use snafu::{ensure, Snafu};

#[cfg(feature = "alloc")]
use alloc::vec::Vec;

#[derive(Deserialize, Serialize, Debug)]
pub struct Request {
    pub new: Vec<u8>,
    pub old: Option<Vec<u8>>,
    pub ordering: Ordering,
    pub path_length: PathLength,
    pub include_values: bool,
}

impl Request {
    pub fn from_request<D: Digest>(request: &super::Request<D>) -> Self {
        let new = request.new.as_slice().into();
        let old = request.old.as_ref().map(|old| old.as_slice().into());
        Request {
            new,
            old,
            ordering: request.ordering,
            path_length: request.path_length,
            include_values: request.include_values,
        }
    }
}

#[derive(Snafu, Debug, Deserialize, Serialize)]
pub enum Error {
    NewWasIncorrectLength,
    OldWasIncorrectLength,
}

impl<'a, D> TryFrom<Request> for super::Request<D>
where
    D: Digest,
{
    type Error = Error;

    fn try_from(value: Request) -> Result<Self, Self::Error> {
        ensure!(value.new.len() == D::output_size(), NewWasIncorrectLength);
        let new = Output::<D>::clone_from_slice(value.new.as_slice());

        // Refactor this later, yikes
        if let Some(old) = value.old.as_ref() {
            ensure!(old.len() == D::output_size(), OldWasIncorrectLength)
        }
        let old = value.old.map(|old| Output::<D>::clone_from_slice(&old));

        let result = Self {
            new,
            old,
            ordering: value.ordering,
            path_length: value.path_length,
            include_values: value.include_values,
        };
        Ok(result)
    }
}

impl<D> From<super::Request<D>> for Request
where
    D: Digest,
{
    fn from(value: super::Request<D>) -> Self {
        todo!()
    }
}
