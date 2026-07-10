//! Error types shared across all ADB operations.

use std::io;

/// Result type for ADB operations.
pub type AdbResult<T> = Result<T, AdbError>;

/// Custom error type for ADB operations.
#[derive(Debug)]
pub enum AdbError {
    ConnectionError(String),
    DeviceNotFound,
    CommandFailed(String),
    IoError(io::Error),
    ParseError(String),
    NoDeviceSelected,
}

impl std::fmt::Display for AdbError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AdbError::ConnectionError(msg) => write!(f, "Connection error: {}", msg),
            AdbError::DeviceNotFound => write!(f, "Device not found"),
            AdbError::CommandFailed(msg) => write!(f, "Command failed: {}", msg),
            AdbError::IoError(e) => write!(f, "IO error: {}", e),
            AdbError::ParseError(msg) => write!(f, "Parse error: {}", msg),
            AdbError::NoDeviceSelected => write!(f, "No device selected"),
        }
    }
}

impl std::error::Error for AdbError {}

impl From<io::Error> for AdbError {
    fn from(e: io::Error) -> Self {
        AdbError::IoError(e)
    }
}

impl From<adb_client::RustADBError> for AdbError {
    fn from(e: adb_client::RustADBError) -> Self {
        AdbError::CommandFailed(e.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_display_device_not_found() {
        assert_eq!(AdbError::DeviceNotFound.to_string(), "Device not found");
    }

    #[test]
    fn error_display_command_failed() {
        assert_eq!(
            AdbError::CommandFailed("boom".into()).to_string(),
            "Command failed: boom"
        );
    }
}
