use thiserror::Error;

pub type PrinterResult<T> = Result<T, QlDriverError>;

#[derive(Error, Debug)]
pub enum QlDriverError {
    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error("received bad data: {0}")]
    BadData(&'static str),

    #[error("the input data length for a raster data transfer should not exceed 2^8")]
    WrongDataSize,

    #[error("timeout while trying to read printer data")]
    ReadTimeout,
}
