use snafu::Snafu;

#[derive(Snafu, Debug)]
#[snafu(visibility = "pub(super)")]
pub enum Error {
    OutBufferTooSmall,
}
