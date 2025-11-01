use std::{
    borrow::Cow,
    fmt::{self, Display},
    io,
    sync::Arc,
};

use crate::format::ArchiveFormat;

pub type Result<T> = std::result::Result<T, ArchiveError>;

/// A string type that can be either borrowed or owned, optimized for error messages.
///
/// This type is used internally to efficiently store error messages that may be
/// either static strings or dynamically generated strings.
pub(crate) type ErrorStr = Cow<'static, str>;

/// Represents all possible errors that can occur during archive operations.
///
/// This enum provides comprehensive error handling for various failure modes
/// in archive processing, including I/O errors, format-specific errors,
/// compression issues, and more.
///
/// # Examples
///
/// ```rust
/// use compak::{ArchiveError, ArchiveFormat};
///
/// // Creating a format-specific error
/// let error = ArchiveError::format_static(ArchiveFormat::Zip, "corrupted central directory");
///
/// // Creating an I/O error with context
/// let error = ArchiveError::io_static("reading archive header",
///                                    std::io::ErrorKind::UnexpectedEof,
///                                    "unexpected end of file");
/// ```
#[derive(Debug, Clone)]
pub enum ArchiveError {
    /// An I/O error occurred during archive operations.
    ///
    /// This variant wraps standard I/O errors with additional context
    /// about where the error occurred and what operation was being performed.
    Io {
        context: ErrorStr,
        kind: io::ErrorKind,
        message: ErrorStr,
    },

    /// An error specific to the archive format.
    ///
    /// This represents errors that are related to the structure or
    /// specification of a particular archive format.
    Format {
        format: ArchiveFormat,
        message: ErrorStr,
    },

    /// An error occurred during compression or decompression.
    ///
    /// This variant represents errors in the compression algorithms
    /// used within archives.
    Compression {
        algorithm: ErrorStr,
        message: ErrorStr,
    },

    /// A required file or archive was not found.
    ///
    /// This error occurs when attempting to access a file or archive
    /// that does not exist at the specified path.
    NotFound { path: ErrorStr },

    /// Access to a file or directory was denied.
    ///
    /// This error occurs when the program lacks the necessary permissions
    /// to access a file or directory.
    PermissionDenied { path: ErrorStr },

    /// A file or directory already exists when it shouldn't.
    ///
    /// This error typically occurs during extraction when a file
    /// would overwrite an existing file without explicit permission.
    AlreadyExists { path: ErrorStr },

    /// The archive is invalid or corrupted.
    ///
    /// This error indicates that the archive structure is malformed
    /// or doesn't conform to the expected format specification.
    InvalidArchive {
        format: ArchiveFormat,
        reason: ErrorStr,
    },

    /// A requested feature is not supported.
    ///
    /// This error occurs when attempting to use functionality
    /// that is not implemented or not available in the current context.
    Unsupported { feature: ErrorStr },

    /// An invalid password was provided for an encrypted archive.
    ///
    /// This error occurs when the password provided for decryption
    /// is incorrect or when no password is provided for an encrypted archive.
    InvalidPassword,

    /// A custom error with a specific message.
    ///
    /// This variant allows for application-specific errors that don't
    /// fit into the other categories.
    Custom { message: ErrorStr },

    /// A nested error that wraps another error with additional context.
    ///
    /// This variant is used to chain errors, preserving the original
    /// error while adding contextual information.
    Nested {
        context: ErrorStr,
        source: Arc<dyn std::error::Error + Send + Sync>,
    },
}

impl Display for ArchiveError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ArchiveError::Io {
                context,
                message,
                ..
            } => {
                write!(f, "I/O error during {}: {}", context, message)
            }
            ArchiveError::Format {
                format,
                message,
            } => {
                write!(f, "{} format error: {}", format, message)
            }
            ArchiveError::Compression {
                algorithm,
                message,
            } => {
                write!(f, "{} compression error: {}", algorithm, message)
            }
            ArchiveError::NotFound {
                path,
            } => {
                write!(f, "File or archive not found: {}", path)
            }
            ArchiveError::PermissionDenied {
                path,
            } => {
                write!(f, "Permission denied accessing: {}", path)
            }
            ArchiveError::AlreadyExists {
                path,
            } => {
                write!(f, "File or directory already exists: {}", path)
            }
            ArchiveError::InvalidArchive {
                format,
                reason,
            } => {
                write!(f, "Invalid {} archive: {}", format, reason)
            }
            ArchiveError::Unsupported {
                feature,
            } => {
                write!(f, "Unsupported feature: {}", feature)
            }
            ArchiveError::InvalidPassword => {
                write!(f, "Invalid password provided for encrypted archive")
            }
            ArchiveError::Custom {
                message,
            } => {
                write!(f, "{}", message)
            }
            ArchiveError::Nested {
                context,
                source,
            } => {
                write!(f, "{}: {}", context, source)
            }
        }
    }
}

impl std::error::Error for ArchiveError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ArchiveError::Nested {
                source, ..
            } => Some(source.as_ref()),
            _ => None,
        }
    }
}

impl ArchiveError {
    /// Creates an I/O error with static strings for context and message.
    ///
    /// This is more efficient than the dynamic version when the context
    /// and message are known at compile time.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use std::io::ErrorKind;
    /// use compak::ArchiveError;
    ///
    /// let error = ArchiveError::io_static(
    ///     "reading file header",
    ///     ErrorKind::UnexpectedEof,
    ///     "unexpected end of file"
    /// );
    /// ```
    pub const fn io_static(
        context: &'static str,
        kind: io::ErrorKind,
        message: &'static str,
    ) -> Self {
        Self::Io {
            context: Cow::Borrowed(context),
            kind,
            message: Cow::Borrowed(message),
        }
    }

    /// Creates an I/O error with dynamic strings for context and message.
    ///
    /// Use this when the context or message need to be generated at runtime.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use std::io::ErrorKind;
    /// use compak::ArchiveError;
    ///
    /// let filename = "archive.zip";
    /// let error = ArchiveError::io_dynamic(
    ///     format!("reading file {}", filename),
    ///     ErrorKind::NotFound,
    ///     format!("file {} not found", filename)
    /// );
    /// ```
    pub fn io_dynamic(
        context: impl Into<String>,
        kind: io::ErrorKind,
        message: impl Into<String>,
    ) -> Self {
        Self::Io {
            context: Cow::Owned(context.into()),
            kind,
            message: Cow::Owned(message.into()),
        }
    }

    /// Creates an `ArchiveError` from a standard `io::Error` with additional context.
    ///
    /// This method preserves the original error kind and provides a more
    /// descriptive message based on the error type.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use std::fs::File;
    /// use compak::ArchiveError;
    ///
    /// match File::open("nonexistent.zip") {
    ///     Ok(_) => {},
    ///     Err(io_err) => {
    ///         let archive_err = ArchiveError::io_from_error("opening archive", io_err);
    ///         eprintln!("Archive error: {}", archive_err);
    ///     }
    /// }
    /// ```
    pub fn io_from_error(context: impl Into<ErrorStr>, source: io::Error) -> Self {
        let kind = source.kind();
        let message = source.to_string();

        let msg_cow = match kind {
            io::ErrorKind::NotFound => Cow::Borrowed("file not found"),
            io::ErrorKind::PermissionDenied => Cow::Borrowed("permission denied"),
            io::ErrorKind::AlreadyExists => Cow::Borrowed("file already exists"),
            _ => Cow::Owned(message),
        };

        Self::Io {
            context: context.into(),
            kind,
            message: msg_cow,
        }
    }

    /// Creates a format error with a static message.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use compak::{ArchiveError, ArchiveFormat};
    ///
    /// let error = ArchiveError::format_static(ArchiveFormat::Zip, "invalid central directory");
    /// ```
    pub const fn format_static(format: ArchiveFormat, message: &'static str) -> Self {
        Self::Format {
            format,
            message: Cow::Borrowed(message),
        }
    }

    /// Creates a format error with a dynamic message.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use compak::{ArchiveError, ArchiveFormat};
    ///
    /// let offset = 1024;
    /// let error = ArchiveError::format_dynamic(
    ///     ArchiveFormat::Tar,
    ///     format!("invalid header at offset {}", offset)
    /// );
    /// ```
    pub fn format_dynamic(format: ArchiveFormat, message: impl Into<String>) -> Self {
        Self::Format {
            format,
            message: Cow::Owned(message.into()),
        }
    }

    /// Creates a "not found" error with a static path.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use compak::ArchiveError;
    ///
    /// let error = ArchiveError::not_found_static("archive.zip");
    /// ```
    pub const fn not_found_static(path: &'static str) -> Self {
        Self::NotFound {
            path: Cow::Borrowed(path),
        }
    }

    /// Creates a "not found" error with a dynamic path.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use compak::ArchiveError;
    ///
    /// let filename = String::from("my_archive.tar.gz");
    /// let error = ArchiveError::not_found_dynamic(filename);
    /// ```
    pub fn not_found_dynamic(path: impl Into<String>) -> Self {
        Self::NotFound {
            path: Cow::Owned(path.into()),
        }
    }

    /// Creates a nested error that wraps another error with additional context.
    ///
    /// This is useful for error chaining, where you want to preserve the
    /// original error while adding contextual information.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use compak::ArchiveError;
    /// use std::io::{Error, ErrorKind};
    ///
    /// let io_error = Error::new(ErrorKind::PermissionDenied, "access denied");
    /// let nested_error = ArchiveError::nested("extracting archive", io_error);
    /// ```
    pub fn nested(
        context: impl Into<ErrorStr>,
        source: impl std::error::Error + Send + Sync + 'static,
    ) -> Self {
        Self::Nested {
            context: context.into(),
            source: Arc::new(source),
        }
    }

    /// Creates a ZIP-specific invalid archive error.
    ///
    /// This is a convenience method for creating format errors specific to ZIP archives.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use compak::ArchiveError;
    ///
    /// let error = ArchiveError::zip_invalid("corrupted central directory");
    /// ```
    pub fn zip_invalid(reason: &'static str) -> Self {
        Self::format_static(ArchiveFormat::Zip, reason)
    }

    /// Creates a TAR-specific invalid archive error.
    ///
    /// This is a convenience method for creating format errors specific to TAR archives.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use compak::ArchiveError;
    ///
    /// let error = ArchiveError::tar_invalid("invalid header checksum");
    /// ```
    pub fn tar_invalid(reason: &'static str) -> Self {
        Self::format_static(ArchiveFormat::Tar, reason)
    }

    /// Creates an "unsupported feature" error with a static message.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use compak::ArchiveError;
    ///
    /// let error = ArchiveError::unsupported_static("ZIP64 format");
    /// ```
    pub fn unsupported_static(feature: &'static str) -> Self {
        Self::Unsupported {
            feature: Cow::Borrowed(feature),
        }
    }

    /// Creates a custom error with a static message.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use compak::ArchiveError;
    ///
    /// let error = ArchiveError::custom_static("operation cancelled by user");
    /// ```
    pub fn custom_static(message: &'static str) -> Self {
        Self::Custom {
            message: Cow::Borrowed(message),
        }
    }
}

impl From<zip::result::ZipError> for ArchiveError {
    fn from(err: zip::result::ZipError) -> Self {
        use zip::result::ZipError;

        match err {
            ZipError::Io(io_err) => Self::io_from_error("ZIP I/O operation", io_err),
            ZipError::InvalidArchive(msg) => {
                Self::InvalidArchive {
                    format: ArchiveFormat::Zip,
                    reason: msg,
                }
            }
            ZipError::UnsupportedArchive(msg) => {
                Self::Unsupported {
                    feature: Cow::Owned(format!("ZIP feature: {}", msg)),
                }
            }
            ZipError::FileNotFound => {
                Self::NotFound {
                    path: Cow::Borrowed("file in ZIP archive"),
                }
            }
            ZipError::InvalidPassword => Self::InvalidPassword,
            _ => {
                Self::Custom {
                    message: Cow::Owned(format!("ZIP error: {}", err)),
                }
            }
        }
    }
}

impl From<sevenz_rust2::Error> for ArchiveError {
    fn from(err: sevenz_rust2::Error) -> Self {
        Self::Custom {
            message: Cow::Owned(format!("7-Zip error: {}", err)),
        }
    }
}

impl From<io::Error> for ArchiveError {
    fn from(err: io::Error) -> Self {
        match err.kind() {
            io::ErrorKind::NotFound => {
                Self::NotFound {
                    path: Cow::Borrowed("unknown"),
                }
            }
            io::ErrorKind::PermissionDenied => {
                Self::PermissionDenied {
                    path: Cow::Borrowed("unknown"),
                }
            }
            io::ErrorKind::AlreadyExists => {
                Self::AlreadyExists {
                    path: Cow::Borrowed("unknown"),
                }
            }
            _ => Self::io_from_error("I/O operation", err),
        }
    }
}

/// Extension trait for creating `ErrorStr` from different string types.
///
/// This trait is used internally to provide a consistent interface for
/// creating error strings from both static and dynamic sources.
pub(crate) trait ErrorStrExt {
    fn from_static(s: &'static str) -> Self;
    fn from_string(s: String) -> Self;
}

impl ErrorStrExt for ErrorStr {
    fn from_static(s: &'static str) -> Self {
        Cow::Borrowed(s)
    }

    fn from_string(s: String) -> Self {
        Cow::Owned(s)
    }
}

/// Extension trait for adding context to `Result` types.
///
/// This trait provides convenient methods for adding contextual information
/// to errors, making error messages more descriptive and useful for debugging.
///
/// # Examples
///
/// ```no_run
/// use std::fs::File;
/// use compak::{Result, ErrorContext};
///
/// fn read_archive_file(path: &str) -> Result<File> {
///     File::open(path)
///         .with_static_context("opening archive file")
/// }
/// ```
pub trait ErrorContext<T> {
    /// Adds dynamic context to an error.
    ///
    /// This method wraps the error with additional context information,
    /// making it easier to understand where and why the error occurred.
    fn with_context(self, context: String) -> Result<T>;

    /// Adds static context to an error.
    ///
    /// This is more efficient than `with_context` when the context
    /// is known at compile time.
    fn with_static_context(self, context: &'static str) -> Result<T>;
}

impl<T> ErrorContext<T> for std::result::Result<T, io::Error> {
    fn with_context(self, context: String) -> Result<T> {
        self.map_err(|err| ArchiveError::io_from_error(ErrorStr::from_string(context), err))
    }

    fn with_static_context(self, context: &'static str) -> Result<T> {
        self.map_err(|err| ArchiveError::io_from_error(ErrorStr::from_static(context), err))
    }
}

impl<T> ErrorContext<T> for Result<T> {
    fn with_context(self, context: String) -> Result<T> {
        self.map_err(|err| ArchiveError::nested(ErrorStr::from_string(context), err))
    }

    fn with_static_context(self, context: &'static str) -> Result<T> {
        self.map_err(|err| ArchiveError::nested(ErrorStr::from_static(context), err))
    }
}
