use crate::FourCC;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("partial decode")]
    ShortRead,

    #[error("partial encode")]
    ShortWrite,

    #[error("out of bounds read")]
    LongRead,

    #[error("out of bounds write")]
    LongWrite,

    #[error("invalid size")]
    InvalidSize,

    #[error("invalid fourcc")]
    InvalidFourCC,

    #[error("unknown version: {0}")]
    UnknownVersion(u8),

    #[error("divide by zero")]
    DivideByZero,

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

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, Error>;
