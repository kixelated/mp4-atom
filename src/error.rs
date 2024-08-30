use crate::FourCC;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("out of bounds")]
    OutOfBounds,

    #[error("partial decode: {0}")]
    PartialDecode(FourCC),

    #[error("over decode: {0}")]
    OverDecode(FourCC),

    #[error("atom too large")]
    TooLarge(FourCC),

    #[error("invalid size")]
    InvalidSize,

    #[error("invalid fourcc")]
    InvalidFourCC,

    #[error("unknown version: {0}")]
    UnknownVersion(u8),

    #[error("invalid string: {0}")]
    InvalidString(String),

    #[error("missing box: {0}")]
    MissingBox(FourCC),

    #[error("unexpected box: {0}")]
    UnexpectedBox(FourCC),

    #[error("duplicate box: {0}")]
    DuplicateBox(FourCC),

    #[error("missing descriptor")]
    MissingDescriptor,

    #[error("unexpected eof")]
    UnexpectedEof,

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, Error>;
