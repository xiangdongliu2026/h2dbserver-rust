use crate::{ByteCursor, DecodeError, ReaderLimits};
#[derive(Clone, Eq, PartialEq, Hash)]
pub struct H2String {
    utf16: Box<[u16]>,
}
impl Ord for H2String {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.utf16.cmp(&other.utf16)
    }
}
impl PartialOrd for H2String {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl H2String {
    pub fn decode(cursor: &mut ByteCursor<'_>, limits: &ReaderLimits) -> Result<Self, DecodeError> {
        let count = cursor.read_var_u32()? as usize;
        if count > limits.max_string_code_units {
            return Err(DecodeError::LimitExceeded {
                resource: "UTF-16 code units",
                requested: count,
                maximum: limits.max_string_code_units,
            });
        }
        let mut units = Vec::with_capacity(count);
        for _ in 0..count {
            units.push(cursor.read_u16_be()?);
        }
        Ok(Self {
            utf16: units.into_boxed_slice(),
        })
    }
    pub fn from_utf16(utf16: Vec<u16>) -> Result<Self, DecodeError> {
        Ok(Self {
            utf16: utf16.into_boxed_slice(),
        })
    }
    pub fn as_utf16(&self) -> &[u16] {
        &self.utf16
    }
    pub fn to_string_lossy(&self) -> String {
        String::from_utf16_lossy(&self.utf16)
    }
    pub fn try_to_string(&self) -> Result<String, Utf16ConversionError> {
        String::from_utf16(&self.utf16).map_err(|_| Utf16ConversionError)
    }
}
#[derive(Debug, thiserror::Error)]
#[error("invalid UTF-16 string")]
pub struct Utf16ConversionError;
