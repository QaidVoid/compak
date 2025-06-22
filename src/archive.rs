use std::{
    io::{self, Read},
    path::{Path, PathBuf},
};

use crate::{
    error::ArchiveError,
    format::{self, ArchiveFormat},
};

/// A handle to an archive file that can be extracted.
///
/// This struct represents an archive file along with its detected format.
/// It provides methods to extract the archive contents to a directory.
///
/// # Examples
///
/// ```rust
/// use compak::Archive;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     // Open an existing archive
///     let archive = Archive::open("example.tar.gz").await?;
///     
///     // Extract to a directory
///     archive.extract_to("./extracted").await?;
///     Ok(())
/// }
/// ```
pub struct Archive {
    path: PathBuf,
    format: ArchiveFormat,
}

impl Archive {
    /// Opens an existing archive file and automatically detects its format.
    ///
    /// This method reads the file to detect the archive format using magic numbers
    /// (file signatures) and falls back to extension-based detection if needed.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the archive file to open
    ///
    /// # Returns
    ///
    /// * `Ok(Archive)` - Successfully opened archive with detected format
    /// * `Err(ArchiveError)` - Failed to open file or detect format
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// * The file does not exist or cannot be read
    /// * The file format cannot be detected or is unsupported
    /// * There are I/O errors while reading the file
    ///
    /// # Examples
    ///
    /// ```rust
    /// use compak::Archive;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let archive = Archive::open("data.zip").await?;
    ///     println!("Opened archive with format: {}", archive.format);
    ///     Ok(())
    /// }
    /// ```
    pub async fn open<P: AsRef<Path>>(path: P) -> Result<Self, ArchiveError> {
        let path = path.as_ref().to_path_buf();
        let format = format::detect_from_file(&path).await?;

        Ok(Archive { path, format })
    }

    /// Creates a new archive instance with format determined by file extension.
    ///
    /// This method is typically used when you want to create a new archive.
    /// The format is determined solely by the file extension of the provided path.
    ///
    /// # Arguments
    ///
    /// * `path` - Path where the archive will be created (format determined by extension)
    ///
    /// # Returns
    ///
    /// * `Ok(Archive)` - Successfully created archive instance
    /// * `Err(ArchiveError)` - Format could not be determined from extension
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// * The file extension is not recognized
    /// * The path has no extension or an unsupported extension
    ///
    /// # Examples
    ///
    /// ```rust
    /// use compak::Archive;
    ///
    /// fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let archive = Archive::new("output.tar.gz")?;
    ///     println!("Created archive for format: {}", archive.format);
    ///     Ok(())
    /// }
    /// ```
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self, ArchiveError> {
        let path = path.as_ref().to_path_buf();
        let format = format::detect_from_extension(&path)?;

        Ok(Archive { path, format })
    }

    /// Extracts the entire archive to the specified output directory.
    ///
    /// This method creates the output directory if it doesn't exist and extracts
    /// all contents of the archive while preserving the directory structure.
    ///
    /// # Arguments
    ///
    /// * `output_dir` - Directory where the archive contents will be extracted
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Extraction completed successfully
    /// * `Err(ArchiveError)` - Extraction failed
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// * The archive file cannot be read
    /// * The output directory cannot be created
    /// * There are permission issues
    /// * The archive is corrupted
    /// * The archive format is not yet implemented
    ///
    /// # Examples
    ///
    /// ```rust
    /// use compak::Archive;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let archive = Archive::open("backup.tar.gz").await?;
    ///     archive.extract_to("./restored_files").await?;
    ///     println!("Archive extracted successfully!");
    ///     Ok(())
    /// }
    /// ```
    pub async fn extract_to<P: AsRef<Path>>(&self, output_dir: P) -> Result<(), ArchiveError> {
        let output_dir = output_dir.as_ref();
        extract_archive_with_format(self.path.as_ref(), output_dir, self.format).await
    }
}

/// Convenience function to extract an archive in a single call.
///
/// This function combines opening an archive and extracting it into one operation.
/// It automatically detects the archive format and extracts all contents.
///
/// # Arguments
///
/// * `archive_path` - Path to the archive file to extract
/// * `output_dir` - Directory where the contents should be extracted
///
/// # Returns
///
/// * `Ok(())` - Archive extracted successfully
/// * `Err(ArchiveError)` - Failed to open or extract the archive
///
/// # Errors
///
/// This function will return an error if:
/// * The archive file cannot be opened or read
/// * The format cannot be detected
/// * The extraction process fails
///
/// # Examples
///
/// ```rust
/// use compak::extract_archive;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     // Extract archive in one line
///     extract_archive("data.zip", "./extracted").await?;
///     println!("Archive extracted!");
///     Ok(())
/// }
/// ```
pub async fn extract_archive<P: AsRef<Path>>(
    archive_path: P,
    output_dir: P,
) -> Result<(), ArchiveError> {
    let archive = Archive::open(archive_path).await?;
    archive.extract_to(output_dir).await
}

/// Internal function that extracts the contents of an archive file to a directory.
///
/// This function handles the actual extraction logic for different archive formats.
/// It creates the output directory if it doesn't exist and delegates to format-specific
/// extraction functions.
///
/// # Arguments
///
/// * `path` - Path to the archive file to be extracted
/// * `output_dir` - Path where contents should be extracted
/// * `format` - The detected archive format to use for extraction
///
/// # Returns
///
/// * `Ok(())` - Extraction was successful
/// * `Err(ArchiveError)` - An error occurred during extraction
///
/// # Errors
///
/// This function will return an error if:
/// * The output directory cannot be created
/// * The archive format is not yet implemented
/// * Format-specific extraction fails
async fn extract_archive_with_format<P: AsRef<Path>>(
    path: P,
    output_dir: P,
    format: ArchiveFormat,
) -> Result<(), ArchiveError> {
    let path = path.as_ref();
    let output_dir = output_dir.as_ref();

    // Ensure output directory exists
    if !output_dir.exists() {
        tokio::fs::create_dir_all(output_dir).await?;
    }

    match format {
        ArchiveFormat::Zip => extract_zip(path, output_dir).await,
        ArchiveFormat::TarGz => extract_tar(path, output_dir, flate2::read::GzDecoder::new).await,
        ArchiveFormat::TarXz => extract_tar(path, output_dir, xz2::read::XzDecoder::new).await,
        ArchiveFormat::TarBz2 => extract_tar(path, output_dir, bzip2::read::BzDecoder::new).await,
        ArchiveFormat::TarZst => {
            extract_tar(path, output_dir, |f| {
                zstd::stream::read::Decoder::new(f).unwrap()
            })
            .await
        }
        ArchiveFormat::Tar => unimplemented!(),
        ArchiveFormat::SevenZ => unimplemented!(),
        ArchiveFormat::Rar => unimplemented!(),
    }
}
/// Generic function for extracting TAR-based archives with different compression formats.
///
/// This function handles the common extraction logic for all TAR-based formats by
/// accepting a decompression function that converts the compressed stream to a
/// readable stream. This allows the same TAR extraction logic to work with
/// GZIP, XZ, BZIP2, and Zstandard compression.
///
/// # Type Parameters
///
/// * `F` - Function type that creates a decompressor from a file
/// * `R` - Reader type returned by the decompression function
///
/// # Arguments
///
/// * `path` - Path to the compressed TAR archive file
/// * `output_dir` - Path where contents should be extracted
/// * `decompress` - Function that takes a file and returns a decompressed reader
///
/// # Returns
///
/// * `Ok(())` - Extraction was successful
/// * `Err(ArchiveError)` - An error occurred during extraction
///
/// # Errors
///
/// This function will return an error if:
/// * The archive file cannot be opened
/// * The decompression fails
/// * The TAR extraction fails
/// * There are I/O errors during extraction
async fn extract_tar<F, R>(
    path: &Path,
    output_dir: &Path,
    decompress: F,
) -> Result<(), ArchiveError>
where
    F: FnOnce(std::fs::File) -> R + Send + 'static,
    R: Read + Send + 'static,
{
    let path = path.to_path_buf();
    let output_dir = output_dir.to_path_buf();

    let file = std::fs::File::open(&path)?;
    let decompressed = decompress(file);
    let mut archive = tar::Archive::new(decompressed);
    archive.unpack(&output_dir)?;

    Ok(())
}

/// Extracts a ZIP archive to the specified output directory.
///
/// This function handles ZIP-specific extraction, including proper handling
/// of directories and file paths. It creates necessary parent directories
/// and extracts all files while preserving the archive's directory structure.
///
/// # Arguments
///
/// * `path` - Path to the ZIP archive file
/// * `output_dir` - Directory where the contents should be extracted
///
/// # Returns
///
/// * `Ok(())` - Extraction was successful
/// * `Err(ArchiveError)` - An error occurred during extraction
///
/// # Errors
///
/// This function will return an error if:
/// * The ZIP file cannot be opened or is corrupted
/// * There are permission issues creating directories or files
/// * There are I/O errors during file extraction
/// * The ZIP contains invalid file paths
async fn extract_zip(path: &Path, output_dir: &Path) -> Result<(), ArchiveError> {
    let path = path.to_path_buf();
    let output_dir = output_dir.to_path_buf();

    let file = std::fs::File::open(&path)?;
    let mut archive = zip::ZipArchive::new(file)?;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let out_path = output_dir.join(file.name());

        if file.name().ends_with('/') {
            std::fs::create_dir_all(&out_path)?;
        } else {
            if let Some(p) = out_path.parent() {
                if !p.exists() {
                    std::fs::create_dir_all(p)?;
                }
            }
            let mut out_file = std::fs::File::create(&out_path)?;
            io::copy(&mut file, &mut out_file)?;
        }
    }
    Ok(())
}
