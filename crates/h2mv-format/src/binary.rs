use crate::DecodeError;

pub struct ByteCursor<'a> {
    bytes: &'a [u8],
    position: usize,
}

impl<'a> ByteCursor<'a> {
    pub fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, position: 0 }
    }
    pub fn position(&self) -> usize {
        self.position
    }
    pub fn remaining(&self) -> usize {
        self.bytes.len() - self.position
    }
    pub fn is_empty(&self) -> bool {
        self.remaining() == 0
    }
    pub fn read_exact(&mut self, length: usize) -> Result<&'a [u8], DecodeError> {
        let end = self
            .position
            .checked_add(length)
            .ok_or(DecodeError::Overflow {
                context: "cursor position",
                offset: self.position,
            })?;
        let result = self
            .bytes
            .get(self.position..end)
            .ok_or(DecodeError::UnexpectedEof {
                offset: self.position,
                wanted: length,
                remaining: self.remaining(),
            })?;
        self.position = end;
        Ok(result)
    }
    pub fn read_u8(&mut self) -> Result<u8, DecodeError> {
        Ok(self.read_exact(1)?[0])
    }
    pub fn read_u16_be(&mut self) -> Result<u16, DecodeError> {
        Ok(u16::from_be_bytes(self.read_exact(2)?.try_into().unwrap()))
    }
    pub fn read_i16_be(&mut self) -> Result<i16, DecodeError> {
        Ok(self.read_u16_be()? as i16)
    }
    pub fn read_u32_be(&mut self) -> Result<u32, DecodeError> {
        Ok(u32::from_be_bytes(self.read_exact(4)?.try_into().unwrap()))
    }
    pub fn read_i32_be(&mut self) -> Result<i32, DecodeError> {
        Ok(self.read_u32_be()? as i32)
    }
    pub fn read_u64_be(&mut self) -> Result<u64, DecodeError> {
        Ok(u64::from_be_bytes(self.read_exact(8)?.try_into().unwrap()))
    }
    pub fn read_i64_be(&mut self) -> Result<i64, DecodeError> {
        Ok(self.read_u64_be()? as i64)
    }
    pub fn read_var_u32(&mut self) -> Result<u32, DecodeError> {
        let start = self.position;
        let mut value = 0u32;
        for shift in (0..35).step_by(7) {
            let byte = self.read_u8()?;
            if shift == 28 && byte & 0xf0 != 0 {
                break;
            }
            value |= u32::from(byte & 0x7f) << shift;
            if byte & 0x80 == 0 {
                return Ok(value);
            }
        }
        Err(DecodeError::Invalid {
            kind: "varint",
            offset: start,
            message: "more than five bytes or overflow".into(),
        })
    }
    pub fn read_var_u64(&mut self) -> Result<u64, DecodeError> {
        let start = self.position;
        let mut value = 0u64;
        for shift in (0..70).step_by(7) {
            let byte = self.read_u8()?;
            if shift == 63 && byte & 0xfe != 0 {
                break;
            }
            value |= u64::from(byte & 0x7f) << shift;
            if byte & 0x80 == 0 {
                return Ok(value);
            }
        }
        Err(DecodeError::Invalid {
            kind: "varlong",
            offset: start,
            message: "more than ten bytes or overflow".into(),
        })
    }
}

pub fn checked_add_u64(left: u64, right: u64, context: &'static str) -> Result<u64, DecodeError> {
    left.checked_add(right)
        .ok_or(DecodeError::Overflow { context, offset: 0 })
}
pub fn checked_mul_u64(left: u64, right: u64, context: &'static str) -> Result<u64, DecodeError> {
    left.checked_mul(right)
        .ok_or(DecodeError::Overflow { context, offset: 0 })
}
pub fn u64_to_usize(value: u64, context: &'static str) -> Result<usize, DecodeError> {
    usize::try_from(value).map_err(|_| DecodeError::Overflow { context, offset: 0 })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn varints_and_truncation() {
        assert_eq!(ByteCursor::new(&[0xac, 2]).read_var_u32(), Ok(300));
        assert!(matches!(
            ByteCursor::new(&[1]).read_u32_be(),
            Err(DecodeError::UnexpectedEof { .. })
        ));
    }
}
