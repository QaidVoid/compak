//! High level archive extraction: detect an archive's format and unpack it.
//!
//! Every format is on by default. Turn `default-features` off and pick the
//! ones you need to keep the dependency tree small:
//!
//! | Feature | Enables |
//! | --- | --- |
//! | `zip` | ZIP archives |
//! | `7z` | 7-Zip archives |
//! | `tar` | TAR archives, plain or with an enabled codec |
//! | `gzip` | `.gz`, and `.tar.gz` with `tar` |
//! | `xz` | `.xz`, and `.tar.xz` with `tar` |
//! | `bzip2` | `.bz2`, and `.tar.bz2` with `tar` |
//! | `zstd` | `.zst`, and `.tar.zst` with `tar` |
//!
//! Detection knows every format either way; extracting one whose feature is
//! off fails with [`ArchiveError::Unsupported`].

pub mod archive;
pub mod error;
pub mod format;

pub use archive::*;
pub use error::*;
pub use format::*;
