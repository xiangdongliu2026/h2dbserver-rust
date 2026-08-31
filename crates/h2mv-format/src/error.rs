use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum DecodeError {
    #[error(
        "unexpected end of input at offset {offset}: wanted {wanted} bytes, {remaining} remain"
    )]
    UnexpectedEof {
        offset: usize,
        wanted: usize,
        remaining: usize,
    },
    #[error("integer overflow while decoding {context} at offset {offset}")]
    Overflow {
        context: &'static str,
        offset: usize,
    },
    #[error("invalid {kind} at offset {offset}: {message}")]
    Invalid {
        kind: &'static str,
        offset: usize,
        message: String,
    },
    #[error("{resource} exceeds configured limit: {requested} > {maximum}")]
    LimitExceeded {
        resource: &'static str,
        requested: usize,
        maximum: usize,
    },
    #[error("checksum mismatch: expected 0x{expected:08x}, calculated 0x{actual:08x}")]
    ChecksumMismatch { expected: u32, actual: u32 },
    #[error("missing required field {0}")]
    MissingField(&'static str),
}
