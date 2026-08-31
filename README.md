# h2mv-rs

A safe, read-only Rust reader for clean, unencrypted H2 2.4.240 MVStore format 3 files.

The implementation deliberately excludes recovery, encryption, SQL row decoding, and all writes.
