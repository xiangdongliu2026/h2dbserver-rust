use crate::{fletcher32, DecodeError, ReaderLimits};
use std::collections::BTreeMap;
#[derive(Clone, Debug)]
pub struct H2TextMap {
    entries: BTreeMap<String, String>,
}
impl H2TextMap {
    pub fn parse_checksummed(bytes: &[u8], limits: &ReaderLimits) -> Result<Self, DecodeError> {
        if bytes.len() > limits.max_text_map_bytes {
            return Err(DecodeError::LimitExceeded {
                resource: "text map bytes",
                requested: bytes.len(),
                maximum: limits.max_text_map_bytes,
            });
        }
        let text: String = bytes.iter().map(|&b| char::from(b)).collect();
        let marker = text
            .rfind(",fletcher:")
            .ok_or_else(|| DecodeError::Invalid {
                kind: "text map",
                offset: 0,
                message: "missing fletcher field".into(),
            })?;
        let checksum_text = text[marker + 10..].trim_matches(['\0', '\r', '\n', ' ']);
        let expected =
            u32::from_str_radix(checksum_text, 16).map_err(|_| DecodeError::Invalid {
                kind: "checksum",
                offset: marker,
                message: "invalid hexadecimal fletcher value".into(),
            })?;
        let actual = fletcher32(&bytes[..marker]);
        if expected != actual {
            return Err(DecodeError::ChecksumMismatch { expected, actual });
        }
        Self::parse_body(&text[..marker])
    }
    fn parse_body(text: &str) -> Result<Self, DecodeError> {
        let mut entries = BTreeMap::new();
        let mut start = 0;
        let mut quoted = false;
        let chars: Vec<char> = text.chars().collect();
        let mut parts = Vec::new();
        for (i, &c) in chars.iter().enumerate() {
            if c == '"' && (i == 0 || chars[i - 1] != '\\') {
                quoted = !quoted;
            } else if c == ',' && !quoted {
                parts.push(chars[start..i].iter().collect::<String>());
                start = i + 1;
            }
        }
        parts.push(chars[start..].iter().collect());
        for part in parts {
            if part.is_empty() {
                continue;
            }
            let (key, value) = part.split_once(':').ok_or_else(|| DecodeError::Invalid {
                kind: "text map entry",
                offset: 0,
                message: "missing colon".into(),
            })?;
            if entries
                .insert(key.into(), value.trim_matches('"').replace("\\\"", "\""))
                .is_some()
            {
                return Err(DecodeError::Invalid {
                    kind: "text map",
                    offset: 0,
                    message: format!("duplicate key {key}"),
                });
            }
        }
        Ok(Self { entries })
    }
    pub fn get(&self, key: &str) -> Option<&str> {
        self.entries.get(key).map(String::as_str)
    }
    pub fn get_hex_u32(&self, key: &str) -> Result<Option<u32>, DecodeError> {
        self.get(key)
            .map(|v| {
                u32::from_str_radix(v, 16).map_err(|_| DecodeError::Invalid {
                    kind: "hex u32",
                    offset: 0,
                    message: key.into(),
                })
            })
            .transpose()
    }
    pub fn get_hex_u64(&self, key: &str) -> Result<Option<u64>, DecodeError> {
        self.get(key)
            .map(|v| {
                u64::from_str_radix(v, 16).map_err(|_| DecodeError::Invalid {
                    kind: "hex u64",
                    offset: 0,
                    message: key.into(),
                })
            })
            .transpose()
    }
    pub fn require_hex_u32(&self, key: &'static str) -> Result<u32, DecodeError> {
        self.get_hex_u32(key)?.ok_or(DecodeError::MissingField(key))
    }
    pub fn require_hex_u64(&self, key: &'static str) -> Result<u64, DecodeError> {
        self.get_hex_u64(key)?.ok_or(DecodeError::MissingField(key))
    }
}
