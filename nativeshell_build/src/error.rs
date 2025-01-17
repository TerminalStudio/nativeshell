use std::{fmt::Display, io, path::PathBuf, process::ExitStatus};

#[derive(Debug)]
pub enum FileOperation {
    CreateDir,
    Copy,
    Move,
    Remove,
    Read,
    Write,
    Open,
    Create,
    SymLink,
    MetaData,
    CopyDir,
    MkDir,
    ReadDir,
    Canonicalize,
    Command,
    Unarchive,
}
#[derive(Debug)]
pub enum BuildError {
    ToolError {
        command: String,
        status: ExitStatus,
        stderr: String,
        stdout: String,
    },
    FileOperationError {
        operation: FileOperation,
        path: PathBuf,
        source_path: Option<PathBuf>,
        source: io::Error,
    },
    JsonError {
        text: Option<String>,
        source: serde_json::Error,
    },
    YamlError {
        source: yaml_rust::ScanError,
    },
    OtherError(String),
}

pub type BuildResult<T> = Result<T, BuildError>;

impl Display for BuildError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BuildError::ToolError {
                command,
                status,
                stderr,
                stdout,
            } => {
                write!(
                    f,
                    "External Tool Failed!\nStatus: {:?}\nCommand: {:?}\nStderr:\n{}\nStdout:\n{}",
                    status, command, stderr, stdout
                )
            }
            BuildError::FileOperationError {
                operation,
                path,
                source_path,
                source,
            } => match source_path {
                Some(source_path) => {
                    write!(
                        f,
                        "File operation failed: {:?}, target path: {:?}, source path: {:?}, error: {}",
                        operation, path, source_path, source
                    )
                }
                None => {
                    write!(
                        f,
                        "File operation failed: {:?}, path: {:?}, error: {}",
                        operation, path, source
                    )
                }
            },
            BuildError::JsonError { text, source } => {
                write!(f, "JSON operation failed: ${}", source)?;
                if let Some(text) = text {
                    write!(f, "Text:\n{}", text)?;
                }
                Ok(())
            }
            BuildError::YamlError { source } => {
                write!(f, "{}", source)
            }
            BuildError::OtherError(err) => {
                write!(f, "{}", err)
            }
        }
    }
}

impl std::error::Error for BuildError {}

pub(super) trait IOResultExt<T> {
    fn wrap_error<F>(self, operation: FileOperation, path: F) -> BuildResult<T>
    where
        F: FnOnce() -> PathBuf;
    fn wrap_error_with_src<F, G>(
        self,
        operation: FileOperation,
        path: F,
        source_path: G,
    ) -> BuildResult<T>
    where
        F: FnOnce() -> PathBuf,
        G: FnOnce() -> PathBuf;
}

impl<T> IOResultExt<T> for io::Result<T> {
    fn wrap_error<F>(self, operation: FileOperation, path: F) -> BuildResult<T>
    where
        F: FnOnce() -> PathBuf,
    {
        self.map_err(|e| BuildError::FileOperationError {
            operation,
            path: path(),
            source_path: None,
            source: e,
        })
    }

    fn wrap_error_with_src<F, G>(
        self,
        operation: FileOperation,
        path: F,
        source_path: G,
    ) -> BuildResult<T>
    where
        F: FnOnce() -> PathBuf,
        G: FnOnce() -> PathBuf,
    {
        self.map_err(|e| BuildError::FileOperationError {
            operation,
            path: path(),
            source_path: Some(source_path()),
            source: e,
        })
    }
}
