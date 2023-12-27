use std::process::ExitStatus;
use std::string::{FromUtf16Error, FromUtf8Error};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Couldn't find command")]
    CommandNotFound,
    #[error("Command failed with exit status {0}: {1}")]
    CommandFailed(ExitStatus, String),
    #[error("Value expected but is not present")]
    NoValue,
    #[error("Failed to execute `{0}`. Received error code `{1}`")]
    GetIfAddrsError(String, i32),
    #[error("Failed to parse bytes into UTF-8 characters. `{0}`")]
    ParseUtf8Error(FromUtf8Error),
    #[error("Failed to parse bytes into UTF-16 characters. `{0}`")]
    ParseUtf16Error(FromUtf16Error),
}

impl From<FromUtf8Error> for Error {
    fn from(error: FromUtf8Error) -> Self {
        Error::ParseUtf8Error(error)
    }
}

impl From<FromUtf16Error> for Error {
    fn from(error: FromUtf16Error) -> Self {
        Error::ParseUtf16Error(error)
    }
}