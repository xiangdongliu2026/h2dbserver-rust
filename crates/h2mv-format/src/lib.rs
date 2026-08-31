//! Binary format support for read-only H2 2.4.240 MVStore files.

mod binary;
mod checksum;
mod error;
mod limits;
mod string;
mod text_map;

pub use binary::{checked_add_u64, checked_mul_u64, u64_to_usize, ByteCursor};
pub use checksum::{calculate_page_check, fletcher32};
pub use error::DecodeError;
pub use limits::ReaderLimits;
pub use string::{H2String, Utf16ConversionError};
pub use text_map::H2TextMap;
