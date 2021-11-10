use snafu::Snafu;
use varu64::DecodeError;

#[derive(Snafu, Debug)]
#[snafu(visibility = "pub(super)")]
pub enum Error {
    OutBufferTooSmall,
    DecodeInputIsLengthZero,
    DecodeRootSizeFromVaru64 { source: DecodeError },
    DecodeSequenceNumberFromVaru64 { source: DecodeError },
    DecodeDeltaSizeFromVaru64 { source: DecodeError },
    DecodeSkipDeltaSizeFromVaru64 { source: DecodeError },
    DecodedSequenceNumberForChildWasNotLargerThanOne,
}
