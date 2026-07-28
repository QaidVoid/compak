use std::{
    fmt::{self, Display},
    fs::File,
    io::Read,
    path::Path,
};

use crate::error::ArchiveError;

/// Enumeration of supported archive formats.
///
/// This enum represents all the archive formats that the library can detect
/// and potentially extract. Some formats may not be fully implemented yet.
///
/// # Examples
///
/// ```rust
/// use compak::ArchiveFormat;
///
/// let format = ArchiveFormat::TarGz;
/// println!("Format: {}", format); // Prints "TAR.GZ"
/// println!("Extension: {}", format.extension()); // Prints "tar.gz"
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArchiveFormat {
    /// ZIP archive format (.zip)
    Zip,
    /// TAR archive compressed with GZIP (.tar.gz, .tgz)
    TarGz,
    /// TAR archive compressed with XZ (.tar.xz, .txz)
    TarXz,
    /// TAR archive compressed with BZIP2 (.tar.bz2, .tbz2)
    TarBz2,
    /// TAR archive compressed with Zstandard (.tar.zst)
    TarZst,
    /// Plain TAR archive (.tar)
    Tar,
    /// 7-Zip archive (.7z)
    SevenZ,
    /// A single GZIP-compressed file (.gz), not a TAR
    Gz,
    /// A single XZ-compressed file (.xz), not a TAR
    Xz,
    /// A single BZIP2-compressed file (.bz2), not a TAR
    Bz2,
    /// A single Zstandard-compressed file (.zst), not a TAR
    Zst,
}

impl Display for ArchiveFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ArchiveFormat::Zip => write!(f, "ZIP"),
            ArchiveFormat::Tar => write!(f, "TAR"),
            ArchiveFormat::TarGz => write!(f, "TAR.GZ"),
            ArchiveFormat::TarBz2 => write!(f, "TAR.BZ2"),
            ArchiveFormat::TarXz => write!(f, "TAR.XZ"),
            ArchiveFormat::TarZst => write!(f, "TAR.ZST"),
            ArchiveFormat::SevenZ => write!(f, "7Z"),
            ArchiveFormat::Gz => write!(f, "GZ"),
            ArchiveFormat::Xz => write!(f, "XZ"),
            ArchiveFormat::Bz2 => write!(f, "BZ2"),
            ArchiveFormat::Zst => write!(f, "ZST"),
        }
    }
}

impl ArchiveFormat {
    /// Returns the standard file extension for this archive format.
    ///
    /// This method returns the most common file extension used for each format.
    /// Note that some formats may have multiple valid extensions (e.g., .tgz for .tar.gz).
    ///
    /// # Returns
    ///
    /// A string slice containing the file extension without the leading dot.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use compak::ArchiveFormat;
    ///
    /// assert_eq!(ArchiveFormat::Zip.extension(), "zip");
    /// assert_eq!(ArchiveFormat::TarGz.extension(), "tar.gz");
    /// assert_eq!(ArchiveFormat::TarXz.extension(), "tar.xz");
    /// ```
    pub fn extension(&self) -> &'static str {
        match self {
            ArchiveFormat::Zip => "zip",
            ArchiveFormat::Tar => "tar",
            ArchiveFormat::TarGz => "tar.gz",
            ArchiveFormat::TarBz2 => "tar.bz2",
            ArchiveFormat::TarXz => "tar.xz",
            ArchiveFormat::TarZst => "tar.zst",
            ArchiveFormat::SevenZ => "7z",
            ArchiveFormat::Gz => "gz",
            ArchiveFormat::Xz => "xz",
            ArchiveFormat::Bz2 => "bz2",
            ArchiveFormat::Zst => "zst",
        }
    }

    /// Returns the MIME type for this archive format.
    ///
    /// This method returns the standard MIME type used for each format.
    ///
    /// # Returns
    ///
    /// A string slice containing the MIME type.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use compak::ArchiveFormat;
    ///
    /// assert_eq!(ArchiveFormat::Zip.mime_type(), "application/zip");
    /// assert_eq!(ArchiveFormat::TarGz.mime_type(), "application/gzip");
    /// ```
    pub fn mime_type(&self) -> &'static str {
        match self {
            ArchiveFormat::Zip => "application/zip",
            ArchiveFormat::TarGz => "application/gzip",
            ArchiveFormat::TarXz => "application/x-xz",
            ArchiveFormat::TarBz2 => "application/x-bzip2",
            ArchiveFormat::TarZst => "application/zstd",
            ArchiveFormat::Tar => "application/x-tar",
            ArchiveFormat::SevenZ => "application/x-7z-compressed",
            ArchiveFormat::Gz => "application/gzip",
            ArchiveFormat::Xz => "application/x-xz",
            ArchiveFormat::Bz2 => "application/x-bzip2",
            ArchiveFormat::Zst => "application/zstd",
        }
    }
}

/// File signature for ZIP files
const ZIP_SIGNATURE: &[u8] = &[0x50, 0x4B, 0x03, 0x04];
/// File signature for GZIP files
const GZIP_SIGNATURE: &[u8] = &[0x1F, 0x8B];
/// File signature for XZ files
const XZ_SIGNATURE: &[u8] = &[0xFD, 0x37, 0x7A, 0x58, 0x5A, 0x00];
/// File signature for BZIP2 files
const BZIP2_SIGNATURE: &[u8] = &[0x42, 0x5A, 0x68];
/// File signature for Zstandard files
const ZSTD_SIGNATURE: &[u8] = &[0x28, 0xB5, 0x2F, 0xFD];
/// File signature for TAR files (located at offset 257)
const TAR_SIGNATURE: &[u8] = &[0x75, 0x73, 0x74, 0x61, 0x72];
/// File signature for 7-Zip files
const SEVENZIP_SIGNATURE: &[u8] = &[0x37, 0x7A, 0xBC, 0xAF, 0x27, 0x1C];

/// Detects archive format from the raw bytes of a file.
///
/// This function examines the magic numbers (file signatures) at the beginning
/// of the file data to determine the archive format. It checks for known
/// byte patterns that identify different archive formats.
///
/// # Arguments
///
/// * `data` - A byte slice containing the beginning of the file
///
/// # Returns
///
/// * `Some(ArchiveFormat)` - Format was successfully detected
/// * `None` - Format could not be determined from the data
///
/// # Examples
///
/// ```rust
/// // This is an internal function, but shows the concept
/// use compak::format::{detect_from_bytes, ArchiveFormat};
///
/// let zip_data = &[0x50, 0x4B, 0x03, 0x04, /* ... */];
/// let format = detect_from_bytes(zip_data);
/// assert_eq!(format, Some(ArchiveFormat::Zip));
/// ```
pub fn detect_from_bytes(data: &[u8]) -> Option<ArchiveFormat> {
    if data.starts_with(ZIP_SIGNATURE) {
        Some(ArchiveFormat::Zip)
    } else if data.starts_with(GZIP_SIGNATURE) {
        Some(ArchiveFormat::TarGz)
    } else if data.starts_with(XZ_SIGNATURE) {
        Some(ArchiveFormat::TarXz)
    } else if data.starts_with(BZIP2_SIGNATURE) {
        Some(ArchiveFormat::TarBz2)
    } else if data.starts_with(ZSTD_SIGNATURE) {
        Some(ArchiveFormat::TarZst)
    } else if data.starts_with(SEVENZIP_SIGNATURE) {
        Some(ArchiveFormat::SevenZ)
    } else if data.len() >= 265 && &data[257..262] == TAR_SIGNATURE {
        Some(ArchiveFormat::Tar)
    } else {
        None
    }
}

/// Detects archive format from a file path's extension.
///
/// This function examines the file extension to determine the archive format.
/// It handles common variations of extensions (e.g., both .tar.gz and .tgz).
///
/// # Arguments
///
/// * `path` - Path to the file whose extension should be examined
///
/// # Returns
///
/// * `Ok(ArchiveFormat)` - Format was successfully detected from extension
/// * `Err(ArchiveError)` - Extension is not recognized or supported
///
/// # Examples
///
/// ```rust
/// use compak::format::{detect_from_extension, ArchiveFormat};
/// use std::path::Path;
///
/// let format = detect_from_extension(Path::new("archive.tar.gz"));
/// let format = detect_from_extension(Path::new("data.zip"));
/// ```
pub fn detect_from_extension<P: AsRef<Path>>(path: P) -> Result<ArchiveFormat, ArchiveError> {
    let path_str = path.as_ref().to_string_lossy().to_lowercase();

    if path_str.ends_with(".tar.gz") || path_str.ends_with(".tgz") {
        Ok(ArchiveFormat::TarGz)
    } else if path_str.ends_with(".tar.xz") || path_str.ends_with(".txz") {
        Ok(ArchiveFormat::TarXz)
    } else if path_str.ends_with(".tar.bz2") || path_str.ends_with(".tbz2") {
        Ok(ArchiveFormat::TarBz2)
    } else if path_str.ends_with(".tar.zst") {
        Ok(ArchiveFormat::TarZst)
    } else if path_str.ends_with(".tar") {
        Ok(ArchiveFormat::Tar)
    } else if path_str.ends_with(".zip") {
        Ok(ArchiveFormat::Zip)
    } else if path_str.ends_with(".7z") {
        Ok(ArchiveFormat::SevenZ)
    } else if path_str.ends_with(".gz") {
        Ok(ArchiveFormat::Gz)
    } else if path_str.ends_with(".xz") {
        Ok(ArchiveFormat::Xz)
    } else if path_str.ends_with(".bz2") {
        Ok(ArchiveFormat::Bz2)
    } else if path_str.ends_with(".zst") {
        Ok(ArchiveFormat::Zst)
    } else {
        Err(ArchiveError::unsupported_static("format"))
    }
}

/// Detects archive format from a file by reading its contents.
///
/// This function first attempts to detect the format using magic numbers
/// by reading the beginning of the file. If that fails, it falls back to
/// extension-based detection.
///
/// # Arguments
///
/// * `path` - Path to the file to examine
///
/// # Returns
///
/// * `Ok(ArchiveFormat)` - Format was successfully detected
/// * `Err(ArchiveError)` - Format could not be determined or file cannot be read
///
/// # Errors
///
/// This function will return an error if:
/// * The file cannot be opened or read
/// * Neither magic number nor extension detection succeeds
/// * I/O errors occur while reading the file
///
/// # Examples
///
/// ```rust
/// use compak::format::detect_from_file;
///
/// let format = detect_from_file("archive.bin");
/// ```
pub fn detect_from_file<P: AsRef<Path>>(path: P) -> Result<ArchiveFormat, ArchiveError> {
    let mut file = File::open(&path)?;
    let mut buffer = [0u8; 512];
    let n = file.read(&mut buffer)?;

    let format = detect_from_bytes(&buffer[..n])
        .or_else(|| detect_from_extension(path.as_ref()).ok())
        .ok_or(ArchiveError::unsupported_static("format"))?;

    Ok(refine_compressed(path.as_ref(), format))
}

/// Distinguish a compressed TAR from a single compressed file.
///
/// Both share a magic number, so the container is only knowable by
/// decompressing far enough to look for the TAR signature at offset 257. A
/// stream that cannot be read is left as detected, so a corrupt archive still
/// fails in extraction rather than being silently reclassified.
fn refine_compressed(path: &Path, format: ArchiveFormat) -> ArchiveFormat {
    let bare = match format {
        ArchiveFormat::TarGz => ArchiveFormat::Gz,
        ArchiveFormat::TarXz => ArchiveFormat::Xz,
        ArchiveFormat::TarBz2 => ArchiveFormat::Bz2,
        ArchiveFormat::TarZst => ArchiveFormat::Zst,
        other => return other,
    };

    match reads_as_tar(path, format) {
        Some(true) | None => format,
        Some(false) => bare,
    }
}

/// Read the first TAR header's worth of the decompressed stream and report
/// whether it carries the TAR signature. `None` means the stream could not be
/// read far enough to tell.
fn reads_as_tar(path: &Path, format: ArchiveFormat) -> Option<bool> {
    let file = File::open(path).ok()?;
    let mut reader: Box<dyn Read> = match format {
        ArchiveFormat::TarGz => Box::new(flate2::read::GzDecoder::new(file)),
        ArchiveFormat::TarXz => Box::new(liblzma::read::XzDecoder::new(file)),
        ArchiveFormat::TarBz2 => Box::new(bzip2::read::BzDecoder::new(file)),
        ArchiveFormat::TarZst => Box::new(zstd::stream::read::Decoder::new(file).ok()?),
        _ => return None,
    };

    let mut head = [0u8; 512];
    let mut filled = 0;
    while filled < head.len() {
        match reader.read(&mut head[filled..]) {
            Ok(0) => break,
            Ok(n) => filled += n,
            Err(_) => return None,
        }
    }

    // Anything shorter than a TAR header cannot be a TAR, and a short read is
    // conclusive here rather than a failure to tell.
    if filled < 262 {
        return Some(false);
    }
    Some(&head[257..262] == TAR_SIGNATURE)
}
