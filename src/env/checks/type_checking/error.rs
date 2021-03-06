#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Init content is not expected")]
    NotExpectingInit,
    #[error("Types in init content must implement Render trait. Found {0}")]
    InitContentNotRender(String),
    #[error("Init content is assigned to the last parameter, but it is not `[Render]` typed")]
    LastFieldIsNotRender,
}

pub type Result<T, E = Error> = std::result::Result<T, E>;
