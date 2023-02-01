use thiserror::Error;

#[derive(Error, Debug)]
pub enum InkmlError {
    #[error(transparent)]
    XmlWriterError(#[from] xml::writer::Error),
    #[error(transparent)]
    XmlReaderError(#[from] xml::reader::Error),

    #[error("Invalid InkML file")]
    InvalidInkml,
}

pub type InkmlResult<T> = Result<T, InkmlError>;
