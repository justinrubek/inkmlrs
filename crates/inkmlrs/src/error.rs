use thiserror::Error;

#[derive(Error, Debug)]
pub enum InkmlError {
    #[error(transparent)]
    XmlWriterError(#[from] xml::writer::Error),
}

pub type InkmlResult<T> = Result<T, InkmlError>;
