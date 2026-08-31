pub fn fletcher32(bytes: &[u8]) -> u32 {
    let mut s1 = 0xffffu32;
    let mut s2 = 0xffffu32;
    let mut words = bytes.chunks_exact(2);
    for word in &mut words {
        s1 = (s1 + u16::from_le_bytes([word[0], word[1]]) as u32) % 0xffff;
        s2 = (s2 + s1) % 0xffff;
    }
    if let [last] = words.remainder() {
        s1 = (s1 + u32::from(*last)) % 0xffff;
        s2 = (s2 + s1) % 0xffff;
    }
    (s2 << 16) | s1
}
pub fn calculate_page_check(chunk_id: u32, offset: u32, page_length: u32) -> u16 {
    (chunk_id as u16)
        ^ ((offset ^ (offset >> 16)) as u16)
        ^ ((page_length ^ (page_length >> 16)) as u16)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn stable() {
        assert_eq!(fletcher32(b"abc"), 0xc52562c4);
        assert_eq!(calculate_page_check(1, 2, 3), 0);
    }
}
