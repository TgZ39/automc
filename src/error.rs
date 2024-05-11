use std::io;
use strum::Display;
use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Error, Display, Debug)]
pub enum Error {
    Api(#[from] reqwest::Error),
    Io(#[from] io::Error),
    Inquire(#[from] inquire::InquireError),
    SerdeJson(#[from] serde_json::Error),
    Encoding(#[from] std::string::FromUtf8Error),
    Other(String),
}
