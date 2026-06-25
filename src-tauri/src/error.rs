use std::io;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("config error: {0}")]
    Config(String),
    #[error("io error: {0}")]
    Io(String),
    #[error("validation error: {0}")]
    Validation(String),
    #[error("runtime error: {0}")]
    Runtime(String),
    #[error("project store error: {0}")]
    ProjectStore(String),
    #[error("terminal error: {0}")]
    Terminal(String),
    #[error("project is launch-locked and cannot be modified")]
    LaunchLocked,
}

impl AppError {
    pub fn config(message: impl Into<String>) -> Self {
        Self::Config(message.into())
    }

    pub fn validation(message: impl Into<String>) -> Self {
        Self::Validation(message.into())
    }

    pub fn runtime(message: impl Into<String>) -> Self {
        Self::Runtime(message.into())
    }

    pub fn project_store(message: impl Into<String>) -> Self {
        Self::ProjectStore(message.into())
    }

    pub fn terminal(message: impl Into<String>) -> Self {
        Self::Terminal(message.into())
    }
}

impl From<io::Error> for AppError {
    fn from(error: io::Error) -> Self {
        Self::Io(error.to_string())
    }
}

impl From<serde_yaml::Error> for AppError {
    fn from(error: serde_yaml::Error) -> Self {
        Self::Config(error.to_string())
    }
}

impl From<AppError> for String {
    fn from(value: AppError) -> Self {
        value.to_string()
    }
}
