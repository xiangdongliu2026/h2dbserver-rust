#[derive(Clone, Debug)]
pub struct ReaderLimits {
    pub max_page_bytes: usize,
    pub max_decompressed_page_bytes: usize,
    pub max_chunk_blocks: u32,
    pub max_key_count: usize,
    pub max_string_code_units: usize,
    pub max_tree_depth: usize,
    pub max_chunk_resolution_depth: usize,
    pub max_meta_entries: usize,
    pub max_text_map_bytes: usize,
}

impl ReaderLimits {
    pub fn safe_defaults() -> Self {
        Self {
            max_page_bytes: 16 << 20,
            max_decompressed_page_bytes: 64 << 20,
            max_chunk_blocks: 1_000_000,
            max_key_count: 1_000_000,
            max_string_code_units: 16 << 20,
            max_tree_depth: 256,
            max_chunk_resolution_depth: 256,
            max_meta_entries: 10_000_000,
            max_text_map_bytes: 1 << 20,
        }
    }
}

impl Default for ReaderLimits {
    fn default() -> Self {
        Self::safe_defaults()
    }
}
