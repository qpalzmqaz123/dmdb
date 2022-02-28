pub type Result<T> = std::result::Result<T, Error>;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("connection error: `{0}`")]
    Connection(String),
    #[error("prepare error: `{0}`")]
    Prepare(String),
    #[error("statement error: `{0}`")]
    Statement(String),
    #[error("parameter error: `{0}`")]
    Parameter(String),
    #[error("index error: `{0}`")]
    Index(String),
    #[error("internal error: `{0}`")]
    Internal(String),
    #[error("from value error: `{0}`")]
    FromValue(String),
    #[error("query returned no rows")]
    QueryReturnedNoRows,
}
