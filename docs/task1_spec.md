# Task 1 Specification: Read-Only H2 MVStore `.mv.db` File Reader in Rust

**Suggested filename:** `h2mv-task-1-read-only-mvstore-reader.md`  
**Project:** Rust Read-Only H2 Query Server  
**Task:** Physical MVStore file reader  
**Reference implementation:** H2 `version-2.4.240`  
**Initial format target:** MVStore format 3  
**Status:** Draft implementation specification

---

## 1. Version Assumption

This specification assumes the target version is **H2 2.4.240**.

The implementation must pin an exact H2 source tag and commit. All test fixtures and Java-oracle outputs must be generated with the matching H2 JAR.

Recommended source reference:

```text
Repository: h2database/h2database
Tag:        version-2.4.240
```

The Rust implementation must reject unsupported MVStore formats or H2 producer versions instead of attempting best-effort decoding.

---

## 2. Scope Clarification

H2’s `SingleFileStore` is only the lowest physical file-access layer.

The relevant Java hierarchy is approximately:

```text
MVStore
  └── FileStore<SFChunk>
        └── RandomAccessStore
              └── SingleFileStore
```

`SingleFileStore` itself is responsible for:

- Opening a file.
- Optionally wrapping it with H2 encryption.
- Obtaining a shared file lock in read-only mode.
- Recording the file size.
- Performing positioned reads.
- Closing the file and releasing the lock.

It does not independently decode:

- Store headers.
- Chunk metadata.
- Pages.
- MVStore maps.
- H2 table rows.
- H2 indexes.
- SQL values.

Those responsibilities are implemented by surrounding H2 classes such as:

```text
RandomAccessStore
FileStore
Chunk
Page
MVMap
Cursor
DataUtils
StringDataType
```

For the Rust project, Task 1 should therefore be broader than a literal `SingleFileStore` port.

---

## 3. Task Definition

### Task 1: Read-Only Physical MVStore Reader

Build a Rust library capable of opening one clean, immutable H2 2.4.240 `.mv.db` file and performing the following operations:

1. Open the file safely in read-only mode.
2. Validate both MVStore file-header copies.
3. Select the current committed store version.
4. Decode and validate chunk metadata.
5. Resolve physical page positions.
6. Read, validate, and decompress MVStore pages.
7. Traverse MVStore B-tree maps.
8. Open the internal layout map.
9. Open the internal meta map.
10. Enumerate all MVStore maps.
11. Perform point reads on internal `String → String` maps.
12. Perform bounded range reads on internal `String → String` maps.

Task 1 will not yet decode H2 SQL tables, rows, indexes, or transaction values.

---

## 4. Input Contract

### 4.1 Initially supported

The first implementation should support only:

- H2 `2.4.240`.
- MVStore format `3`.
- Physical block size `4096`.
- Local regular files.
- Read-only file access.
- Files that remain immutable while open.
- Files cleanly closed by H2.
- Unencrypted files.
- Uncompressed pages.
- LZF-compressed pages.
- Deflate-compressed pages.
- Little or no unresolved transaction state.
- Linux x86-64 as the initial deployment platform.

### 4.2 Initially rejected

The reader must reject:

- Unknown H2 producer versions.
- Unknown MVStore formats.
- Unknown block sizes.
- Uncleanly closed databases.
- Files requiring recovery.
- Encrypted files.
- Files actively modified by another process.
- Files copied while H2 was writing them.
- Truncated files.
- Corrupted files.
- Legacy PageStore files.
- Unsupported compression modes.
- Write operations.
- Compaction.
- Repair.
- Commit processing.
- Garbage collection.

Recovery support must be a separate future task. It must not be silently introduced into the initial file reader.

---

## 5. Recommended Rust Workspace

```text
h2mv-rs/
├── Cargo.toml
├── crates/
│   ├── h2mv-format/
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── error.rs
│   │       ├── limits.rs
│   │       │
│   │       ├── binary/
│   │       │   ├── mod.rs
│   │       │   ├── cursor.rs
│   │       │   ├── varint.rs
│   │       │   └── checked.rs
│   │       │
│   │       ├── checksum/
│   │       │   ├── mod.rs
│   │       │   ├── fletcher32.rs
│   │       │   └── page_check.rs
│   │       │
│   │       ├── text_map/
│   │       │   ├── mod.rs
│   │       │   └── parser.rs
│   │       │
│   │       ├── string/
│   │       │   ├── mod.rs
│   │       │   ├── h2_string.rs
│   │       │   └── codec.rs
│   │       │
│   │       ├── store/
│   │       │   ├── mod.rs
│   │       │   ├── header.rs
│   │       │   └── snapshot.rs
│   │       │
│   │       ├── chunk/
│   │       │   ├── mod.rs
│   │       │   ├── metadata.rs
│   │       │   ├── header.rs
│   │       │   └── footer.rs
│   │       │
│   │       ├── page/
│   │       │   ├── mod.rs
│   │       │   ├── position.rs
│   │       │   ├── header.rs
│   │       │   ├── node.rs
│   │       │   ├── leaf.rs
│   │       │   └── decoder.rs
│   │       │
│   │       ├── compression/
│   │       │   ├── mod.rs
│   │       │   ├── lzf.rs
│   │       │   └── deflate.rs
│   │       │
│   │       └── datatype/
│   │           ├── mod.rs
│   │           └── codec.rs
│   │
│   ├── h2mv-reader/
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── options.rs
│   │       │
│   │       ├── io/
│   │       │   ├── mod.rs
│   │       │   ├── read_at.rs
│   │       │   ├── single_file.rs
│   │       │   ├── file_identity.rs
│   │       │   └── file_lock.rs
│   │       │
│   │       ├── store/
│   │       │   ├── mod.rs
│   │       │   ├── bootstrap.rs
│   │       │   └── read_context.rs
│   │       │
│   │       ├── chunk/
│   │       │   ├── mod.rs
│   │       │   ├── registry.rs
│   │       │   └── resolver.rs
│   │       │
│   │       ├── page/
│   │       │   ├── mod.rs
│   │       │   └── loader.rs
│   │       │
│   │       ├── btree/
│   │       │   ├── mod.rs
│   │       │   ├── map.rs
│   │       │   ├── cursor.rs
│   │       │   └── path.rs
│   │       │
│   │       ├── catalog/
│   │       │   ├── mod.rs
│   │       │   ├── layout.rs
│   │       │   ├── meta.rs
│   │       │   └── map_descriptor.rs
│   │       │
│   │       ├── cache/
│   │       │   ├── mod.rs
│   │       │   ├── no_cache.rs
│   │       │   └── bounded.rs
│   │       │
│   │       ├── metrics.rs
│   │       └── reader.rs
│   │
│   ├── h2mv-inspect/
│   │   └── src/
│   │       └── main.rs
│   │
│   └── h2mv-testkit/
│       ├── java-oracle/
│       ├── fixtures/
│       └── src/
│           └── lib.rs
```

### 5.1 Crate responsibilities

#### `h2mv-format`

Contains H2-compatible binary format logic:

- Store-header parser.
- Chunk parser.
- Page-position decoder.
- Page parser.
- Checksums.
- Compression.
- H2 string encoding.
- Data codecs.

This crate may contain algorithms selectively translated from H2.

#### `h2mv-reader`

Contains independently designed Rust runtime components:

- File I/O.
- Snapshot bootstrap.
- Chunk resolution.
- Page loading.
- B-tree traversal.
- Caching.
- Public API.
- Metrics.

#### `h2mv-inspect`

Command-line inspection and validation utility.

#### `h2mv-testkit`

Contains:

- Java H2 oracle.
- Fixture generation.
- Differential tests.
- Corrupted-file fixtures.
- Fuzzing support.

Separating the format and runtime crates simplifies:

- Version pinning.
- H2 format upgrades.
- Licensing.
- Testing.
- Reuse by the future SQL server.

---

## 6. Module and Type Requirements

# 6.1 File I/O Module

## `ReadAt`

The reader requires an abstraction equivalent to the useful read-only portion of Java `FileChannel`.

```rust
pub trait ReadAt: Send + Sync + 'static {
    fn len(&self) -> Result<u64, ReaderError>;

    fn read_exact_at(
        &self,
        offset: u64,
        destination: &mut [u8],
    ) -> Result<(), ReaderError>;
}
```

### Responsibilities

- Perform positioned reads.
- Avoid a shared seek cursor.
- Support concurrent reads from multiple threads.
- Retry interrupted reads.
- Handle short reads.
- Detect unexpected EOF.
- Check `offset + length` overflow.
- Collect read-operation and read-byte metrics.

### Linux implementation

On Unix-like systems, use `FileExt::read_at`.

```rust
use std::fs::File;
use std::os::unix::fs::FileExt;

fn read_exact_at(
    file: &File,
    mut offset: u64,
    mut destination: &mut [u8],
) -> std::io::Result<()> {
    while !destination.is_empty() {
        let bytes_read = file.read_at(destination, offset)?;

        if bytes_read == 0 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::UnexpectedEof,
                "unexpected EOF during positioned read",
            ));
        }

        offset = offset
            .checked_add(bytes_read as u64)
            .ok_or_else(|| {
                std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "file offset overflow",
                )
            })?;

        destination = &mut destination[bytes_read..];
    }

    Ok(())
}
```

---

## `ReadOnlySingleFileStore`

```rust
pub struct ReadOnlySingleFileStore {
    file: std::fs::File,
    path: std::path::PathBuf,
    identity: FileIdentity,
    opened_length: u64,
    lock: Option<FileLockGuard>,
    stats: IoStats,
}
```

### Responsibilities

- Open the file with read-only permissions.
- Verify that the path refers to a regular file.
- Capture the initial file identity.
- Optionally acquire an advisory shared lock.
- Implement `ReadAt`.
- Detect file truncation.
- Close the file and release the lock through `Drop`.

### Explicitly excluded

Do not implement:

- `writeFully`
- `sync`
- `truncate`
- `backup`
- free-space management
- file growth
- encryption during the first milestone

---

## `FileIdentity`

```rust
pub struct FileIdentity {
    pub length: u64,
    pub modified_at: Option<std::time::SystemTime>,
    pub platform_id: PlatformFileId,
}
```

On Linux, `PlatformFileId` should include:

```rust
pub struct PlatformFileId {
    pub device: u64,
    pub inode: u64,
}
```

The open file descriptor remains authoritative. Identity checks provide additional diagnostics and can detect unexpected path replacement.

---

## `FileLockGuard`

```rust
pub struct FileLockGuard {
    file: std::fs::File,
}
```

The initial implementation should attempt a shared advisory lock.

The lock policy should be configurable because:

- Some filesystems do not support advisory locking consistently.
- Files may be atomically published and never modified.
- The production deployment may rely on filesystem-level immutability.

---

# 6.2 Safe Binary Decoder Module

## `ByteCursor`

```rust
pub struct ByteCursor<'a> {
    bytes: &'a [u8],
    position: usize,
}
```

Required operations:

```rust
impl<'a> ByteCursor<'a> {
    pub fn read_u8(&mut self) -> Result<u8, DecodeError>;

    pub fn read_u16_be(&mut self) -> Result<u16, DecodeError>;

    pub fn read_i16_be(&mut self) -> Result<i16, DecodeError>;

    pub fn read_u32_be(&mut self) -> Result<u32, DecodeError>;

    pub fn read_i32_be(&mut self) -> Result<i32, DecodeError>;

    pub fn read_u64_be(&mut self) -> Result<u64, DecodeError>;

    pub fn read_i64_be(&mut self) -> Result<i64, DecodeError>;

    pub fn read_var_u32(&mut self) -> Result<u32, DecodeError>;

    pub fn read_var_u64(&mut self) -> Result<u64, DecodeError>;

    pub fn read_exact(
        &mut self,
        length: usize,
    ) -> Result<&'a [u8], DecodeError>;

    pub fn position(&self) -> usize;

    pub fn remaining(&self) -> usize;

    pub fn is_empty(&self) -> bool;
}
```

### Requirements

- No unchecked indexing.
- No panics for malformed input.
- Checked integer conversion.
- Checked position advancement.
- Configurable maximum lengths.
- Structured errors with offsets.

---

## Checked arithmetic helpers

```rust
pub fn checked_add_u64(
    left: u64,
    right: u64,
    context: &'static str,
) -> Result<u64, DecodeError>;

pub fn checked_mul_u64(
    left: u64,
    right: u64,
    context: &'static str,
) -> Result<u64, DecodeError>;

pub fn u64_to_usize(
    value: u64,
    context: &'static str,
) -> Result<usize, DecodeError>;
```

Every file-controlled offset, count, or length must use checked arithmetic.

---

## `ReaderLimits`

```rust
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
```

Example safe defaults:

```rust
impl ReaderLimits {
    pub fn safe_defaults() -> Self {
        Self {
            max_page_bytes: 16 * 1024 * 1024,
            max_decompressed_page_bytes: 64 * 1024 * 1024,
            max_chunk_blocks: 1_000_000,
            max_key_count: 1_000_000,
            max_string_code_units: 16 * 1024 * 1024,
            max_tree_depth: 256,
            max_chunk_resolution_depth: 256,
            max_meta_entries: 10_000_000,
            max_text_map_bytes: 1024 * 1024,
        }
    }
}
```

These values are security boundaries, not only performance settings.

---

# 6.3 Checksum Module

## Fletcher-32

Implement H2-compatible Fletcher-32 validation.

```rust
pub fn fletcher32(bytes: &[u8]) -> u32;
```

Use cases:

- Store-header text-map validation.
- Chunk metadata validation.
- Other H2 checksummed textual structures.

---

## Page check value

Page validation uses information such as:

- Chunk ID.
- Page offset.
- Page length.

Expose the exact H2-compatible calculation through:

```rust
pub fn calculate_page_check(
    chunk_id: u32,
    offset: u32,
    page_length: u32,
) -> u16;
```

The calculation must be tested against values produced by the pinned Java H2 implementation.

---

# 6.4 H2 Text-Map Parser

H2 store headers and chunk metadata use an ISO-8859-1 textual key-value format.

## `H2TextMap`

```rust
pub struct H2TextMap {
    entries: std::collections::BTreeMap<String, String>,
}
```

Required operations:

```rust
impl H2TextMap {
    pub fn parse_checksummed(
        bytes: &[u8],
        limits: &ReaderLimits,
    ) -> Result<Self, DecodeError>;

    pub fn get(&self, key: &str) -> Option<&str>;

    pub fn get_hex_u32(
        &self,
        key: &str,
    ) -> Result<Option<u32>, DecodeError>;

    pub fn get_hex_u64(
        &self,
        key: &str,
    ) -> Result<Option<u64>, DecodeError>;

    pub fn require_hex_u32(
        &self,
        key: &'static str,
    ) -> Result<u32, DecodeError>;

    pub fn require_hex_u64(
        &self,
        key: &'static str,
    ) -> Result<u64, DecodeError>;
}
```

### Parser responsibilities

- Parse comma-separated `key:value` entries.
- Handle quoted values.
- Handle escaped values.
- Decode ISO-8859-1 input.
- Parse hexadecimal integers.
- Parse hexadecimal byte arrays where required.
- Validate the trailing `fletcher` checksum.
- Reject duplicate mandatory fields where ambiguity would result.
- Return structured errors instead of partial maps.

---

# 6.5 H2 String Representation

Do not immediately represent on-disk MVStore strings as Rust `String`.

H2 serializes strings using Java UTF-16 code units. Java string comparison is based on UTF-16 code-unit ordering, which is not always equivalent to UTF-8 byte ordering.

## `H2String`

```rust
#[derive(Clone, Eq, PartialEq, Hash)]
pub struct H2String {
    utf16: Box<[u16]>,
}
```

Required implementation:

```rust
impl Ord for H2String {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.utf16.cmp(&other.utf16)
    }
}

impl PartialOrd for H2String {
    fn partial_cmp(
        &self,
        other: &Self,
    ) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
```

Required methods:

```rust
impl H2String {
    pub fn decode(
        cursor: &mut ByteCursor<'_>,
        limits: &ReaderLimits,
    ) -> Result<Self, DecodeError>;

    pub fn from_utf16(
        utf16: Vec<u16>,
    ) -> Result<Self, DecodeError>;

    pub fn as_utf16(&self) -> &[u16];

    pub fn to_string_lossy(&self) -> String;

    pub fn try_to_string(
        &self,
    ) -> Result<String, Utf16ConversionError>;
}
```

This representation preserves:

- Java-compatible ordering.
- Supplementary Unicode behavior.
- Unpaired surrogate behavior.
- Correct B-tree lookup semantics.

Using normal Rust `String` comparison may select the wrong child in an MVStore B-tree for some Unicode keys.

---

# 6.6 Store Header Module

## `StoreHeaderCandidate`

Represents one of the two physical store-header copies.

```rust
pub struct StoreHeaderCandidate {
    pub block_index: u8,
    pub valid_checksum: bool,
    pub version: u64,
    pub chunk_id: u32,
    pub chunk_block: u64,
    pub clean: bool,
    pub attributes: H2TextMap,
}
```

## `StoreHeader`

Represents the selected and validated store header.

```rust
pub struct StoreHeader {
    pub format: u32,
    pub format_read: u32,
    pub block_size: u32,
    pub created_at_millis: u64,
    pub version: u64,
    pub last_chunk_id: u32,
    pub last_chunk_block: u64,
    pub clean_shutdown: bool,
}
```

## `StoreSnapshot`

```rust
pub struct StoreSnapshot {
    pub header: StoreHeader,
    pub last_chunk: std::sync::Arc<ChunkMeta>,
    pub version: u64,
    pub layout_root_pos: PagePos,
}
```

## Open algorithm

1. Open the file.
2. Read bytes `0..8192`.
3. Split the data into two 4096-byte header blocks.
4. Parse each header independently.
5. Validate each checksum independently.
6. Reject headers with unsupported format or block size.
7. Select the newest valid header version.
8. Read the referenced latest chunk.
9. Validate its chunk header.
10. Validate its chunk footer.
11. Verify that the chunk metadata matches the store header.
12. Require a clean-shutdown marker.
13. Freeze the selected snapshot for the lifetime of the reader.

If the clean path cannot be validated, Task 1 must return an error rather than entering H2-style recovery.

---

# 6.7 Chunk Module

## `ChunkId`

```rust
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct ChunkId(pub u32);
```

## `ChunkMeta`

```rust
pub struct ChunkMeta {
    pub id: ChunkId,
    pub block: u64,
    pub block_count: u32,
    pub page_count: u32,
    pub max_page_bytes: u64,
    pub layout_root_pos: PagePos,
    pub version: u64,
    pub created_at_delta_millis: u64,
    pub last_map_id: u32,
    pub next_block: Option<u64>,
    pub toc_offset: Option<u32>,
}
```

The parser should also safely recognize optional or write-side fields even when the read-only implementation does not use them.

Examples include:

- Live page count.
- Live page bytes.
- Occupancy information.
- Unused timestamps.
- Pin counts.

## `ChunkHeader`

```rust
pub struct ChunkHeader {
    pub metadata: ChunkMeta,
}
```

## `ChunkFooter`

```rust
pub struct ChunkFooter {
    pub chunk_id: ChunkId,
    pub block_count: u32,
    pub version: u64,
    pub checksum: u32,
}
```

## Validation requirements

- Header chunk ID equals the expected chunk ID.
- Footer chunk ID equals the header chunk ID.
- Footer block count equals the header block count.
- Footer version equals the header version.
- Chunk byte range is inside the file.
- Block-to-byte conversions use checked arithmetic.
- Footer checksum is valid.
- Header metadata length is bounded.
- Chunk block count does not exceed configured limits.

## Read-only fields not required

Do not model write-side state unless it is needed for parsing:

- Garbage-collection priority.
- Chunk relocation state.
- Free-space allocation.
- Current live-page updates.
- Pin-count updates.
- Unused-space timestamps.
- Chunk-writing state.

---

## `ChunkRegistry`

```rust
pub struct ChunkRegistry {
    known: std::sync::RwLock<
        std::collections::HashMap<
            ChunkId,
            std::sync::Arc<ChunkMeta>,
        >,
    >,
}
```

Responsibilities:

- Seed the registry with the latest chunk from the store header.
- Cache chunk metadata loaded from the layout map.
- Avoid holding a write lock during file I/O.
- Permit duplicate benign resolution.
- Detect recursive chunk lookup.
- Enforce a maximum chunk-resolution depth.

---

# 6.8 Page Position Module

## `PagePos`

```rust
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct PagePos(u64);
```

Required methods:

```rust
impl PagePos {
    pub fn from_raw(raw: u64) -> Result<Self, DecodeError>;

    pub fn raw(self) -> u64;

    pub fn chunk_id(self) -> ChunkId;

    pub fn offset(self) -> u32;

    pub fn encoded_max_length(self) -> u8;

    pub fn page_type(self) -> PageType;

    pub fn is_saved(self) -> bool;
}
```

H2 encodes the following into a 64-bit page position:

- Chunk ID.
- Page offset within the chunk.
- Maximum encoded page-length class.
- Node or leaf page type.

Use explicit bit masks and unsigned arithmetic.

Do not rely on Java-style truncating casts.

---

# 6.9 Compression Module

## `PageDecompressor`

```rust
pub trait PageDecompressor: Send + Sync {
    fn expand(
        &self,
        compressed: &[u8],
        expected_length: usize,
        limits: &ReaderLimits,
    ) -> Result<Vec<u8>, DecodeError>;
}
```

Required implementations:

```rust
pub struct LzfPageDecompressor;

pub struct DeflatePageDecompressor;
```

Only decompression is required.

Do not implement page compression during Task 1.

Before allocating:

```rust
if expected_length > limits.max_decompressed_page_bytes {
    return Err(DecodeError::LimitExceeded {
        resource: "decompressed page",
        requested: expected_length,
        maximum: limits.max_decompressed_page_bytes,
    });
}
```

Additional protections:

- Validate compressed input length.
- Validate expected output length.
- Reject trailing or malformed streams where appropriate.
- Never allow decompression to exceed the configured output limit.
- Fuzz both decompressors.

---

# 6.10 Page Module

## `PageType`

```rust
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PageType {
    Leaf,
    Node,
}
```

## `CompressionKind`

```rust
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CompressionKind {
    None,
    Lzf,
    Deflate,
}
```

## `PageHeader`

```rust
pub struct PageHeader {
    pub serialized_length: u32,
    pub check_value: u16,
    pub page_number: u32,
    pub map_id: u32,
    pub key_count: u32,
    pub page_type: PageType,
    pub compression: CompressionKind,
}
```

The page decoder must account for:

1. Serialized page length.
2. Page check value.
3. Page number.
4. Map ID.
5. Key count.
6. Page type and compression flags.
7. Child references for node pages.
8. Compression-length information.
9. Encoded keys.
10. Encoded values for leaf pages.

## `ChildRef`

```rust
pub struct ChildRef {
    pub position: PagePos,
    pub descendant_count: u64,
}
```

## Immutable decoded-page representation

```rust
pub enum DecodedPage<K, V> {
    Leaf(LeafPage<K, V>),
    Node(NodePage<K>),
}
```

```rust
pub struct LeafPage<K, V> {
    pub position: PagePos,
    pub page_number: u32,
    pub keys: Box<[K]>,
    pub values: Box<[V]>,
}
```

```rust
pub struct NodePage<K> {
    pub position: PagePos,
    pub page_number: u32,
    pub keys: Box<[K]>,
    pub children: Box<[ChildRef]>,
}
```

## Page validation

Every page load must validate:

- Page position is valid.
- Encoded chunk ID is valid.
- Offset is inside the chunk.
- Maximum encoded page length is valid.
- Actual page length is positive and bounded.
- Actual page length fits inside the chunk.
- Page check value is correct.
- Page number is valid.
- Map ID equals the expected map ID.
- Encoded node/leaf type matches the expected page position.
- Key count does not exceed configured limits.
- Node child count equals `key_count + 1`.
- Decompressed length is valid.
- Key and value decoders consume valid payload.
- No decoder reads beyond the page boundary.

## Java behavior not to port

Do not port the mutable behavior of H2’s `Page` class:

- Insertion.
- Update.
- Removal.
- Page split.
- Copy-on-write.
- Page serialization for writing.
- Position mutation.
- Atomic position updater.
- Removal marker.
- Dirty-state tracking.
- Memory-estimation bookkeeping.

The Rust page model should be immutable after successful validation.

---

# 6.11 Data Codec Module

## `DataCodec`

```rust
pub trait DataCodec<T>: Send + Sync {
    fn decode_one(
        &self,
        cursor: &mut ByteCursor<'_>,
        limits: &ReaderLimits,
    ) -> Result<T, DecodeError>;

    fn decode_many(
        &self,
        cursor: &mut ByteCursor<'_>,
        count: usize,
        limits: &ReaderLimits,
    ) -> Result<Box<[T]>, DecodeError>;

    fn compare(
        &self,
        left: &T,
        right: &T,
    ) -> std::cmp::Ordering;
}
```

## Task 1 codec

Task 1 initially requires:

```rust
pub struct H2StringCodec;
```

Later tasks will add:

```text
ValueDataTypeCodec
RowDataTypeCodec
VersionedValueCodec
PrimaryIndexKeyCodec
SecondaryIndexKeyCodec
```

The page decoder must remain generic over key and value codecs.

Do not directly hard-code string decoding into the page parser.

---

# 6.12 Page Loader

## `RawPage`

```rust
pub struct RawPage {
    pub position: PagePos,
    pub bytes: bytes::Bytes,
    pub page_type: PageType,
    pub map_id: u32,
}
```

## `PageLoader`

```rust
pub struct PageLoader {
    store: std::sync::Arc<dyn ReadAt>,
    chunks: std::sync::Arc<ChunkRegistry>,
    cache: std::sync::Arc<dyn PageCache>,
    limits: ReaderLimits,
}
```

Primary API:

```rust
impl PageLoader {
    pub fn load<K, V>(
        &self,
        map_id: u32,
        position: PagePos,
        key_codec: &dyn DataCodec<K>,
        value_codec: &dyn DataCodec<V>,
        resolution: &mut ResolutionStack,
    ) -> Result<
        std::sync::Arc<DecodedPage<K, V>>,
        ReaderError,
    >;
}
```

## Page-load sequence

1. Decode `PagePos`.
2. Resolve the containing chunk.
3. Calculate the physical file offset.
4. Determine the maximum number of bytes to read.
5. Perform a positioned read.
6. Parse the actual page length.
7. Validate the page check value.
8. Parse the page header.
9. Parse child references for a node.
10. Decompress the key/value payload when required.
11. Decode keys.
12. Decode leaf values.
13. Verify complete and valid payload consumption.
14. Insert the validated page into the cache.
15. Return an immutable shared page.

---

# 6.13 B-Tree Module

## `ReadOnlyMvMap`

```rust
pub struct ReadOnlyMvMap<K, V> {
    pub map_id: u32,
    pub root_position: PagePos,
    key_codec: std::sync::Arc<dyn DataCodec<K>>,
    value_codec: std::sync::Arc<dyn DataCodec<V>>,
}
```

Required API:

```rust
impl<K, V> ReadOnlyMvMap<K, V> {
    pub fn get(
        &self,
        context: &ReadContext,
        key: &K,
    ) -> Result<Option<V>, ReaderError>;

    pub fn cursor<'a>(
        &'a self,
        context: &'a ReadContext,
        lower: std::ops::Bound<K>,
        upper: std::ops::Bound<K>,
    ) -> Result<MapCursor<'a, K, V>, ReaderError>;
}
```

The actual implementation may return borrowed or reference-counted values to avoid cloning large objects.

## Point lookup algorithm

1. Load the root page.
2. Binary-search the page keys.
3. For a leaf:
   - Return the matching value.
   - Return `None` when no key matches.
4. For a node:
   - Select the correct child.
   - Descend into the child.
5. Enforce maximum tree depth.
6. Validate map ID at every page.

## `MapCursor`

```rust
pub struct MapCursor<'a, K, V> {
    map: &'a ReadOnlyMvMap<K, V>,
    context: &'a ReadContext,
    path: Vec<PathFrame<K>>,
    upper: std::ops::Bound<K>,
}
```

## `PathFrame`

```rust
struct PathFrame<K> {
    page: std::sync::Arc<NodePage<K>>,
    next_child_index: usize,
}
```

Required cursor behavior:

- Seek to a lower bound.
- Binary-search keys within each node.
- Descend to the first matching leaf.
- Iterate entries in the leaf.
- Move to the next leaf using the retained parent path.
- Stop at the upper bound.
- Support inclusive and exclusive bounds.
- Enforce maximum tree depth.
- Detect invalid child references.
- Never retain the complete B-tree.

## Root representation

Use a simple immutable snapshot:

```rust
pub struct RootSnapshot {
    pub map_id: u32,
    pub position: PagePos,
    pub store_version: u64,
}
```

Do not port:

- Java `ConcurrentMap`.
- Root CAS chains.
- `RootReference` lock state.
- Version updates.
- Decision makers.
- Append buffers.
- Mutations.
- Page rewriting.

---

# 6.14 Layout Catalog

The layout map:

- Has map ID `0`.
- Uses H2 string keys.
- Uses H2 string values.
- Contains chunk records.
- Contains map-root records.
- Identifies the meta map.

Representative keys include:

```text
chunk.<id>
root.<mapId>
meta.id
```

## `LayoutCatalog`

```rust
pub struct LayoutCatalog {
    map: ReadOnlyMvMap<H2String, H2String>,
}
```

Required operations:

```rust
impl LayoutCatalog {
    pub fn meta_map_id(
        &self,
        context: &ReadContext,
    ) -> Result<u32, ReaderError>;

    pub fn root_position(
        &self,
        context: &ReadContext,
        map_id: u32,
    ) -> Result<Option<PagePos>, ReaderError>;

    pub fn chunk_metadata(
        &self,
        context: &ReadContext,
        chunk_id: ChunkId,
        resolution: &mut ResolutionStack,
    ) -> Result<Option<ChunkMeta>, ReaderError>;

    pub fn scan_entries(
        &self,
        context: &ReadContext,
    ) -> Result<
        impl Iterator<
            Item = Result<
                (H2String, H2String),
                ReaderError,
            >,
        >,
        ReaderError,
    >;
}
```

---

# 6.15 Meta Catalog

The meta map is opened by:

1. Reading `meta.id` from the layout map.
2. Reading the root position for the meta map ID.
3. Opening that root as another `String → String` MVStore map.

## `MetaCatalog`

```rust
pub struct MetaCatalog {
    map_id: u32,
    map: ReadOnlyMvMap<H2String, H2String>,
}
```

Representative meta keys include:

```text
name.<map-name>
map.<map-id>
root.<map-id>
setting.<name>
```

## `MapDescriptor`

```rust
pub struct MapDescriptor {
    pub id: u32,
    pub name: Option<H2String>,
    pub map_type: Option<H2String>,
    pub creation_version: Option<u64>,
    pub root_position: Option<PagePos>,
    pub raw_metadata: H2String,
}
```

Required operations:

```rust
impl MetaCatalog {
    pub fn list_maps(
        &self,
        context: &ReadContext,
        layout: &LayoutCatalog,
    ) -> Result<Vec<MapDescriptor>, ReaderError>;

    pub fn map_descriptor(
        &self,
        context: &ReadContext,
        layout: &LayoutCatalog,
        map_id: u32,
    ) -> Result<Option<MapDescriptor>, ReaderError>;
}
```

Task 1 does not need to understand application-level semantics of names such as:

```text
table.<id>
index.<id>
undoLog.<id>
lobData
lobMap
```

It only needs to enumerate them correctly.

---

# 6.16 Chunk Resolution

## Circular dependency

The layout map stores chunk metadata, but reading the layout map may require loading a page from an older chunk.

This creates the following dependency:

```text
Load layout page
   |
   v
Resolve page chunk
   |
   v
Read chunk.<id> from layout map
   |
   v
Load another layout page
```

The resolver must avoid infinite recursion and lock deadlocks.

## Required strategy

1. Seed the chunk registry with the latest chunk from the file header.
2. Check the registry before reading the layout map.
3. Resolve missing chunks lazily through `chunk.<id>`.
4. Track the current resolution path.
5. Detect cycles.
6. Enforce maximum resolution depth.
7. Do not hold the registry write lock during recursive page reads.

## `ResolutionStack`

```rust
pub struct ResolutionStack {
    chunk_ids: Vec<ChunkId>,
}
```

Suggested API:

```rust
impl ResolutionStack {
    pub fn enter(
        &mut self,
        id: ChunkId,
        limit: usize,
    ) -> Result<ResolutionGuard<'_>, ReaderError>;
}
```

The guard removes the chunk ID when the nested resolution operation completes.

This component requires dedicated tests using multi-chunk databases where layout-map nodes and leaves reside in different chunks.

---

# 6.17 Cache Module

## `PageCacheKey`

```rust
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct PageCacheKey {
    pub file_generation: u64,
    pub position: PagePos,
}
```

## `PageCache`

```rust
pub trait PageCache: Send + Sync {
    fn get(
        &self,
        key: PageCacheKey,
    ) -> Option<std::sync::Arc<RawPage>>;

    fn insert(
        &self,
        key: PageCacheKey,
        page: std::sync::Arc<RawPage>,
    );

    fn current_bytes(&self) -> usize;

    fn capacity_bytes(&self) -> usize;

    fn clear_generation(&self, generation: u64);
}
```

## Implementation order

1. Implement `NoCache`.
2. Validate reader correctness without caching.
3. Add `BoundedPageCache`.
4. Bound the cache by bytes, not entry count.
5. Shard the internal locks.
6. Avoid holding cache locks during disk I/O.
7. Prefer internal node retention over very large leaf pages.
8. Ensure evicted pages can be released.
9. Avoid parent-child references that retain the entire B-tree.

Do not mechanically port H2’s Java `CacheLongKeyLIRS` during Task 1.

A simpler byte-bounded cache is preferable initially.

---

# 6.18 Metrics Module

## I/O metrics

```text
file_read_operations_total
file_read_bytes_total
file_short_reads_total
file_unexpected_eof_total
```

## Header and chunk metrics

```text
store_header_reads_total
store_header_validation_failures_total
chunk_resolutions_total
chunk_registry_hits_total
chunk_registry_misses_total
chunk_validation_failures_total
```

## Page metrics

```text
page_loads_total
page_read_bytes_total
page_decode_failures_total
page_check_failures_total
page_decompressions_total
page_decompressed_bytes_total
```

## Cache metrics

```text
page_cache_hits_total
page_cache_misses_total
page_cache_insertions_total
page_cache_evictions_total
page_cache_bytes
```

## B-tree metrics

```text
map_point_lookups_total
map_range_scans_total
map_pages_visited_total
map_max_observed_depth
```

Metrics must avoid high-cardinality labels such as raw map keys or file paths.

---

## 7. H2 Java Class-to-Rust Mapping

| H2 Java class | Rust replacement | Behavior to port | Behavior not to port |
|---|---|---|---|
| `SingleFileStore` | `ReadOnlySingleFileStore` | Open, shared lock, size, positioned reads, close | Write, truncate, sync, backup, encryption initially |
| `RandomAccessStore` | `StoreBootstrap` | Two-header selection and clean snapshot validation | Allocation, free-space tracking, compaction, recovery initially |
| `FileStore` | `ReadContext`, `ChunkResolver`, `PageLoader` | Header constants, layout map, chunk lookup, page reads, root lookup | Commit, serialization, housekeeping, background writes |
| `SFChunk` | `ChunkMeta` or `FileChunk` | Physical position information | Write callbacks |
| `Chunk` | `ChunkMeta`, `ChunkHeader`, `ChunkFooter` | Persisted metadata parsing and validation | Live-page bookkeeping and garbage collection |
| `DataUtils` | `binary`, `page_pos`, `text_map`, `checksum` modules | Varints, strings, page positions, map parsing, checksums | Unrelated Java utility methods |
| `Page` | `DecodedPage`, `NodePage`, `LeafPage` | Read and validate persisted pages | Mutation, split, copy, removal, write |
| `MVMap` | `ReadOnlyMvMap` | Point lookup, binary search, root position, cursor | `ConcurrentMap`, mutations, root CAS |
| `Cursor` | `MapCursor` | Tree path and bounded iteration | Mutation-related behavior |
| `CursorPos` | `PathFrame` | Parent path and child index | Java linked-object layout |
| `RootReference` | `RootSnapshot` | Root page position and store version | Lock state, counters, version chain |
| `DataType` | `DataCodec<T>` | Decode and compare | Encoding initially |
| `BasicDataType` | Codec helper functions | Batch decode and binary search helpers | Memory estimation |
| `StringDataType` | `H2StringCodec` | Exact disk decode and Java-compatible comparison | Java object arrays |
| `CompressLZF` | `LzfPageDecompressor` | Expand only | Compression |
| `CompressDeflate` | `DeflatePageDecompressor` | Expand only | Compression |
| `MVStore` | `MvStoreReader` | Open and bootstrap layout/meta maps | Transactions, commits, versions, background writer |
| `CacheLongKeyLIRS` | `BoundedPageCache` | No direct port required | Avoid mechanical Java cache port |

---

## 8. Critical Bootstrap Sequence

```text
Open file
   |
   v
Read two 4 KiB store headers
   |
   v
Validate each header checksum
   |
   v
Select the newest valid clean header
   |
   v
Read the referenced latest chunk header
   |
   v
Read and validate its chunk footer
   |
   v
Obtain layoutRootPos from the latest chunk
   |
   v
Open layout map:
    map ID 0
    H2String -> H2String
   |
   +-- chunk.<id> -> ChunkMeta
   +-- root.<id>  -> PagePos
   +-- meta.id    -> meta map ID
   |
   v
Find the meta-map root
   |
   v
Open meta map:
    H2String -> H2String
   |
   v
Enumerate:
    name.*
    map.*
    root.*
    setting.*
```

This bootstrap path is the central functional goal of Task 1.

---

## 9. Read Context

## `ReadContext`

```rust
pub struct ReadContext {
    pub store: std::sync::Arc<dyn ReadAt>,
    pub snapshot: StoreSnapshot,
    pub chunks: std::sync::Arc<ChunkRegistry>,
    pub pages: PageLoader,
    pub limits: ReaderLimits,
    pub file_generation: u64,
}
```

The context should be immutable except for internally synchronized caches and registries.

It must be safe to share through `Arc<ReadContext>`.

---

## 10. Public API

## `OpenOptions`

```rust
use std::path::Path;
use std::sync::Arc;

#[derive(Clone, Debug)]
pub struct OpenOptions {
    pub require_clean_shutdown: bool,
    pub acquire_shared_lock: bool,
    pub page_cache_bytes: usize,
    pub limits: ReaderLimits,
}
```

Default configuration:

```rust
impl Default for OpenOptions {
    fn default() -> Self {
        Self {
            require_clean_shutdown: true,
            acquire_shared_lock: true,
            page_cache_bytes: 0,
            limits: ReaderLimits::safe_defaults(),
        }
    }
}
```

## `MvStoreReader`

```rust
pub struct MvStoreReader {
    context: Arc<ReadContext>,
    snapshot: StoreSnapshot,
    layout: LayoutCatalog,
    meta: MetaCatalog,
}
```

Required API:

```rust
impl MvStoreReader {
    pub fn open(
        path: impl AsRef<Path>,
        options: OpenOptions,
    ) -> Result<Self, ReaderError>;

    pub fn snapshot(&self) -> &StoreSnapshot;

    pub fn layout(&self) -> &LayoutCatalog;

    pub fn meta(&self) -> &MetaCatalog;

    pub fn list_maps(
        &self,
    ) -> Result<Vec<MapDescriptor>, ReaderError>;

    pub fn map_descriptor(
        &self,
        map_id: u32,
    ) -> Result<Option<MapDescriptor>, ReaderError>;

    pub fn io_stats(&self) -> IoStatsSnapshot;

    pub fn cache_stats(&self) -> CacheStatsSnapshot;
}
```

The initial public API must not expose:

- Internal mutexes.
- Mutable pages.
- Mutable map roots.
- File write handles.
- Unvalidated page bytes.
- H2 implementation-specific object graphs.

---

## 11. Error Model

Use one public error type with structured variants.

```rust
#[derive(Debug, thiserror::Error)]
pub enum ReaderError {
    #[error("I/O failure at byte offset {offset}: {source}")]
    Io {
        offset: u64,

        #[source]
        source: std::io::Error,
    },

    #[error(
        "unexpected EOF at byte offset {offset}: wanted {wanted} bytes"
    )]
    UnexpectedEof {
        offset: u64,
        wanted: usize,
    },

    #[error("path is not a regular file: {path}")]
    NotRegularFile {
        path: std::path::PathBuf,
    },

    #[error("unable to obtain a shared file lock")]
    FileLockUnavailable,

    #[error("invalid MVStore header copy {copy}")]
    InvalidStoreHeader {
        copy: u8,
    },

    #[error("store-header checksum mismatch")]
    StoreHeaderChecksumMismatch,

    #[error(
        "unsupported MVStore format {found}; expected {expected}"
    )]
    UnsupportedFormat {
        found: u32,
        expected: u32,
    },

    #[error(
        "unsupported block size {found}; expected {expected}"
    )]
    UnsupportedBlockSize {
        found: u32,
        expected: u32,
    },

    #[error("database was not cleanly closed")]
    UncleanShutdown,

    #[error("chunk {chunk_id} was not found")]
    ChunkNotFound {
        chunk_id: u32,
    },

    #[error("invalid chunk header for chunk {chunk_id}")]
    InvalidChunkHeader {
        chunk_id: u32,
    },

    #[error(
        "chunk footer does not match its header for chunk {chunk_id}"
    )]
    ChunkFooterMismatch {
        chunk_id: u32,
    },

    #[error("invalid page position 0x{position:016x}")]
    InvalidPagePosition {
        position: u64,
    },

    #[error("invalid page length {length}")]
    InvalidPageLength {
        length: u32,
    },

    #[error(
        "page check mismatch at position 0x{position:016x}"
    )]
    PageCheckMismatch {
        position: u64,
    },

    #[error(
        "page belongs to map {actual}, expected map {expected}"
    )]
    MapIdMismatch {
        expected: u32,
        actual: u32,
    },

    #[error("unsupported page compression type {kind}")]
    UnsupportedCompression {
        kind: u8,
    },

    #[error(
        "{resource} exceeds configured limit: {requested} > {maximum}"
    )]
    LimitExceeded {
        resource: &'static str,
        requested: usize,
        maximum: usize,
    },

    #[error(
        "recursive chunk resolution detected for chunk {chunk_id}"
    )]
    RecursiveChunkResolution {
        chunk_id: u32,
    },

    #[error("maximum B-tree depth exceeded")]
    MaximumTreeDepthExceeded,

    #[error("invalid UTF-16 data in metadata key")]
    InvalidMetadataString,

    #[error("database file changed while it was open")]
    FileChanged,

    #[error("unsupported MVStore feature: {feature}")]
    UnsupportedFeature {
        feature: &'static str,
    },

    #[error("corrupt MVStore data: {message}")]
    CorruptData {
        message: String,
    },
}
```

Malformed input must produce an error, not:

- A panic.
- An abort.
- Infinite recursion.
- Excessive allocation.
- Unbounded decompression.
- Silent partial results.

---

## 12. Explicit Non-Goals for Task 1

Do not include the following H2 database-level classes or capabilities in Task 1:

```text
org.h2.mvstore.db.ValueDataType
org.h2.mvstore.db.RowDataType
org.h2.mvstore.db.MVPrimaryIndex
org.h2.mvstore.db.MVSecondaryIndex
org.h2.mvstore.tx.TransactionStore
org.h2.mvstore.tx.VersionedValue
org.h2.mvstore.db.LobStorageMap
org.h2.command.Parser
org.h2.table.*
org.h2.index.*
```

Task 1 will not provide:

- SQL parsing.
- SQL execution.
- Table-row decoding.
- Primary-index decoding.
- Secondary-index decoding.
- Transaction visibility.
- Undo-log interpretation.
- LOB decoding.
- JDBC.
- TCP serving.
- File recovery.
- File encryption.
- Writes.
- Compaction.
- Repair.

Task 1 will expose:

```text
Store metadata
Current store version
Current chunk metadata
Chunk records
Page metadata
MVStore map IDs
MVStore map names
MVStore map types
MVStore root positions
Layout map contents
Meta map contents
String/String point lookup
String/String bounded range scan
```

Task 2 should add H2 database-level values, rows, transactions, tables, and indexes.

---

## 13. Implementation Order

# PR 1: Source Pinning and Java Oracle

### Deliverables

- Pin H2 `version-2.4.240`.
- Record the exact source commit.
- Resolve the exact H2 JAR through Maven.
- Generate clean one-chunk and multi-chunk databases.
- Create a Java oracle capable of dumping:
  - Store header.
  - Current store version.
  - Latest chunk.
  - Layout map.
  - Meta map.
  - Map IDs.
  - Map names.
  - Root positions.
- Add a source-origin manifest.
- Add initial fixture documentation.

### Acceptance

- The test suite runs against one exact H2 version.
- Every fixture records its producer version.
- Java-oracle output is deterministic.
- Fixture generation is reproducible.

---

# PR 2: File I/O and Binary Primitives

### Implement

- `ReadAt`
- `ReadOnlySingleFileStore`
- Shared file locking
- `FileIdentity`
- `ByteCursor`
- Checked arithmetic
- Varint decoding
- Varlong decoding
- H2 string decoding
- Fletcher-32
- H2 text-map parser
- `ReaderLimits`

### Acceptance

- Both raw store-header blocks can be read.
- Header text can be parsed.
- No chunk or page support is required yet.
- Varint parser is fuzzed.
- Text-map parser is fuzzed.
- Truncation produces typed errors.
- No malformed input causes a panic.

---

# PR 3: Store Header and Latest Chunk Validation

### Implement

- `StoreHeaderCandidate`
- `StoreHeader`
- Header selection
- `ChunkId`
- `ChunkMeta`
- Chunk-header parser
- Chunk-footer parser
- Latest-chunk validation
- Clean-shutdown requirement
- Format and block-size checks

### Acceptance

- Opens clean one-chunk files.
- Opens clean multi-chunk files.
- Selects the same store version as Java H2.
- Rejects damaged header copies.
- Uses the remaining valid header when one copy is damaged.
- Rejects mismatched chunk footers.
- Rejects unsupported format.
- Rejects unclean shutdown.

---

# PR 4: Page Positions, Page Loading, and Compression

### Implement

- `PagePos`
- Page-position bit decoding
- Physical page-range calculation
- Page header parser
- Page check calculation
- Node child-reference parsing
- LZF decompression
- Deflate decompression
- Raw page loader
- Page-size limits

### Acceptance

- Decodes page headers from known fixtures.
- Page number matches Java H2.
- Map ID matches Java H2.
- Node/leaf type matches Java H2.
- Key count matches Java H2.
- Corrupted check values are rejected.
- Oversized decompression is rejected.
- Truncated compressed pages are rejected.

---

# PR 5: Generic String/String Pages

### Implement

- `DataCodec<T>`
- `H2String`
- `H2StringCodec`
- `LeafPage`
- `NodePage`
- Generic page decoder
- Binary search
- Java-compatible H2 string ordering

### Acceptance

- Decodes known layout-map pages.
- Decoded keys match Java output.
- Decoded values match Java output.
- Non-ASCII metadata keys work.
- Supplementary Unicode tests pass.
- Unpaired UTF-16 surrogate behavior is handled deterministically.
- Binary search selects the same positions as Java.

---

# PR 6: Layout Map and Chunk Resolver

### Implement

- `ReadOnlyMvMap`
- Point lookup
- `ChunkRegistry`
- `ChunkResolver`
- `ResolutionStack`
- Resolution-cycle detection
- Layout-map bootstrap
- `chunk.*` lookup
- `root.*` lookup
- `meta.id` lookup

### Acceptance

- Enumerates all chunk metadata.
- Random layout-map lookups match Java H2.
- Multi-chunk recursive resolution works.
- Recursive resolution cycles are detected.
- Registry locks are not held during file I/O.
- Registry locks are not held during recursive resolution.

---

# PR 7: Meta Map and Cursor

### Implement

- `MapCursor`
- Lower-bound seeking
- Inclusive and exclusive bounds
- Forward leaf traversal
- `MetaCatalog`
- `MapDescriptor`
- Map enumeration
- Root-position lookup

### Acceptance

- Full layout-map scan matches Java H2.
- Full meta-map scan matches Java H2.
- Lower-bound behavior matches Java.
- Upper-bound behavior matches Java.
- Empty maps work.
- Single-page maps work.
- Deep maps work.
- Maximum tree depth is enforced.

---

# PR 8: Cache, CLI, and Hardening

### Implement

- `NoCache`
- Byte-bounded page cache
- Cache sharding
- Metrics
- `h2mv-inspect`
- Concurrent reader tests
- Fuzz harnesses
- Soak tests
- File-change diagnostics
- Documentation

### Acceptance

- Cache can be disabled.
- Cache remains within its configured byte budget.
- Evicted pages are releasable.
- Cache locks are not held during file I/O.
- Concurrent point and range reads are safe.
- CLI output matches Java-oracle metadata.
- Long-running tests show no unbounded memory growth.

---

## 14. CLI Deliverable

Task 1 should produce an inspection tool.

Basic usage:

```bash
h2mv-inspect database.mv.db
```

Example output:

```text
File
  path:             database.mv.db
  length:           48234496
  format:           3
  block_size:       4096
  clean_shutdown:   true
  store_version:    000000000000012f

Current chunk
  id:               0000012f
  block:            0000000000002ae1
  blocks:           00000031
  pages:            00000058
  layout_root:      00004bc00003b91c

Maps
  id=0x0   name=<layout>    root=0x00004bc00003b91c
  id=0x1   name=<meta>      root=0x00004bc00005d50c
  id=0x5   name=table.3     root=0x00004b800002a10c
  id=0x6   name=index.4     root=0x00004b800003001d
```

Additional commands:

```bash
h2mv-inspect database.mv.db header
```

```bash
h2mv-inspect database.mv.db chunks
```

```bash
h2mv-inspect database.mv.db maps
```

```bash
h2mv-inspect database.mv.db layout
```

```bash
h2mv-inspect database.mv.db meta
```

```bash
h2mv-inspect database.mv.db map-get 0 root.5
```

```bash
h2mv-inspect database.mv.db \
  map-scan 0 \
  --from chunk.0 \
  --to chunk.ffff
```

Useful global flags:

```text
--no-lock
--no-cache
--cache-bytes <bytes>
--max-page-bytes <bytes>
--json
--verbose
```

---

## 15. Testing Strategy

# 15.1 Java differential oracle

For every supported operation:

1. Generate or load the same `.mv.db` file.
2. Read it using Java H2 2.4.240.
3. Read it using the Rust implementation.
4. Compare normalized outputs.

Compare:

- Selected store version.
- Store-header fields.
- Latest chunk fields.
- Chunk registry.
- Layout-map entries.
- Meta-map entries.
- Map IDs.
- Map names.
- Root positions.
- Point lookup results.
- Range scan results.
- Error classifications for damaged files.

---

# 15.2 Fixture corpus

Generate fixtures containing:

- Empty database.
- One user table.
- Multiple user tables.
- One chunk.
- Many chunks.
- Deep layout-map tree.
- Deep meta-map tree.
- Compressed pages.
- Uncompressed pages.
- LZF pages.
- Deflate pages.
- Many small maps.
- Large map metadata.
- Non-ASCII map names.
- Supplementary Unicode.
- Large file offsets.
- A damaged first header.
- A damaged second header.
- Both headers damaged.
- A damaged chunk footer.
- An invalid page check.
- A truncated page.
- An oversized decompression declaration.
- Recursive or invalid chunk references.

---

# 15.3 Fuzz targets

Fuzz:

- Store-header parser.
- Chunk-header parser.
- Chunk-footer parser.
- Text-map parser.
- Fletcher checksum handling.
- Page-position decoding.
- Varint decoder.
- Varlong decoder.
- H2 string decoder.
- Page-header decoder.
- Node child-reference decoder.
- LZF decompressor.
- Deflate decompressor.
- Generic page decoder.
- Chunk resolver.
- B-tree cursor boundary logic.

Fuzzing properties:

- No panic.
- No abort.
- No memory allocation above configured limits.
- No infinite loop.
- No unbounded recursion.
- No out-of-bounds access.
- Deterministic structured error.

---

# 15.4 Concurrency tests

Test:

- Multiple threads reading the same file.
- Concurrent point lookups.
- Concurrent range scans.
- Cache hits during concurrent reads.
- Cache misses for the same page from multiple threads.
- Chunk resolution from multiple threads.
- Reader shutdown while no operations are active.
- Reader use through `Arc`.
- File truncation diagnostics.
- File-path replacement diagnostics.

---

# 15.5 Soak test

Run at least one 24-hour test with:

- Multiple reader threads.
- Repeated point lookups.
- Repeated bounded scans.
- Cache enabled.
- Cache disabled.
- Multiple `.mv.db` fixture sizes.
- Periodic metrics capture.

Verify:

- No unbounded process-memory growth.
- No file-descriptor leak.
- No cache-size violation.
- No deadlock.
- No increasing lookup latency caused by leaked state.
- Stable page-cache behavior.

---

## 16. Acceptance Criteria

# 16.1 Correctness

Task 1 is complete when the reader:

- Opens clean, unencrypted H2 2.4.240 files.
- Rejects unsupported formats.
- Selects the same store version as Java H2.
- Finds the same current chunk as Java H2.
- Enumerates the same layout entries.
- Enumerates the same meta entries.
- Enumerates the same map names.
- Enumerates the same map IDs.
- Resolves the same root positions.
- Returns matching point-lookup results.
- Returns matching bounded range-scan results.
- Reads uncompressed pages.
- Reads LZF-compressed pages.
- Reads Deflate-compressed pages.

---

# 16.2 Safety

The implementation must provide:

- No panic for malformed input.
- No unchecked integer overflow.
- No unbounded allocation.
- No unbounded decompression.
- Maximum page size.
- Maximum key count.
- Maximum string size.
- Maximum B-tree depth.
- Maximum chunk-resolution depth.
- Recursive chunk-resolution detection.
- Typed errors for truncated files.
- Typed errors for unknown page flags.
- Typed errors for unsupported formats.
- Typed errors for invalid checksums.

---

# 16.3 Memory

The implementation must:

- Never load the complete file into process memory.
- Load pages on demand.
- Support operation with no page cache.
- Enforce a hard cache byte limit.
- Release evicted pages when no reader references remain.
- Avoid parent-child object graphs that retain complete trees.
- Avoid copying page data unnecessarily.
- Avoid allocating Rust strings when H2 UTF-16 data can remain encoded.

---

# 16.4 Concurrency

The implementation must:

- Allow one `MvStoreReader` to be shared through `Arc`.
- Use positioned reads without a global file seek mutex.
- Permit multiple concurrent point lookups.
- Permit multiple concurrent range scans.
- Avoid holding cache locks during file I/O.
- Avoid holding chunk-registry locks during recursive resolution.
- Use immutable page objects.
- Provide safe cancellation boundaries for future query integration.

---

# 16.5 Verification

Completion requires:

- Passing Java/Rust differential tests.
- Passing malformed-input tests.
- Fuzz coverage for headers, chunks, pages, strings, and compression.
- Inspection of at least one multi-gigabyte database without memory growth proportional to file size.
- Completion of a 24-hour concurrent-read soak test.
- Documented benchmark of:
  - Open latency.
  - Point-lookup latency.
  - Range-scan latency.
  - Page-cache hit rate.
  - Memory usage.
  - Incremental memory per open reader.

---

## 17. Final Task Boundary

The final Task 1 dependency graph is:

```text
ReadOnlySingleFileStore
        |
        v
StoreHeaderParser
        |
        v
ChunkHeader/Footer Parser
        |
        v
PageLoader + Decompressors
        |
        v
Generic Page<K, V> Decoder
        |
        v
ReadOnlyMvMap<H2String, H2String>
        |
        +-- LayoutCatalog
        |      |
        |      +-- ChunkResolver
        |      +-- Map root positions
        |      +-- Meta map ID
        |
        +-- MetaCatalog
               |
               +-- Map IDs
               +-- Map names
               +-- Map descriptors
```

Do not put the following into Task 1:

- H2 `Value`.
- H2 `Row`.
- Transaction visibility.
- Undo logs.
- SQL tables.
- Primary indexes.
- Secondary indexes.
- SQL parser.
- Query planner.
- JDBC protocol.
- TCP server.

---

## 18. Effort Estimate

A realistic estimate for one senior Rust and storage-format engineer is:

```text
Core reader implementation:       5–7 engineer-weeks
Java differential tooling:        1–2 engineer-weeks
Fuzzing and corruption testing:   1–2 engineer-weeks
Cache, CLI, and hardening:        1–2 engineer-weeks
-----------------------------------------------------
Total:                            8–12 engineer-weeks
```

The estimate assumes:

- One exact H2 version.
- Cleanly closed files.
- No encryption.
- No recovery.
- Only internal `String → String` MVStore maps.
- No SQL table-row decoding.

The following would be separate substantial additions:

- Crash recovery.
- File encryption.
- H2 SQL value decoding.
- H2 row decoding.
- Transaction visibility.
- Primary and secondary indexes.
- LOB storage.
- Additional H2 versions.

---

## 19. Licensing and Source Provenance

H2 2.4.240 is available under MPL 2.0 or EPL 1.0.

If Rust code is translated or adapted from H2 Java sources:

- Select an applicable H2 license deliberately.
- Preserve copyright notices.
- Mark source-derived Rust files.
- Record the originating H2 Java source file.
- Record the exact H2 source tag and commit.
- Maintain an explicit source-origin manifest.
- Keep source-derived compatibility code in separate files or crates.
- Obtain legal review before distributing binaries externally.

Example provenance manifest:

```text
Rust module                     H2 source reference
----------------------------------------------------------------
store/header.rs                 RandomAccessStore.java
io/single_file.rs               SingleFileStore.java
chunk/metadata.rs               Chunk.java
page/position.rs                DataUtils.java
page/decoder.rs                 Page.java
btree/map.rs                    MVMap.java
btree/cursor.rs                 Cursor.java
string/codec.rs                 StringDataType.java
compression/lzf.rs              CompressLZF.java
compression/deflate.rs          CompressDeflate.java
catalog/layout.rs               FileStore.java
catalog/meta.rs                 MVStore.java
```

This licensing section is an engineering recommendation and should receive formal legal review.

---

## 20. Definition of Done

Task 1 is done when this command works reliably:

```bash
h2mv-inspect production.mv.db maps
```

The command must:

1. Open the file read-only.
2. Validate the current clean snapshot.
3. Find and validate the latest chunk.
4. Open the layout map.
5. Resolve all required chunks.
6. Open the meta map.
7. Enumerate all maps.
8. Produce output matching H2 2.4.240.
9. Operate within configured memory limits.
10. Return structured errors for unsupported or corrupted files.

The resulting Rust crates must provide a stable foundation for Task 2:

```text
H2 value decoding
H2 row decoding
Transaction visibility
Table metadata
Primary-index access
Secondary-index access
```