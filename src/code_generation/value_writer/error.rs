#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("No content is written")]
    EmptyContent,
    #[error("{0}")]
    IoError(#[from] std::io::Error),
}
