use serde::Serialize;

#[derive(Debug, thiserror::Error)]
pub enum KeelError {
    #[error("{0}")]
    Validation(String),
    #[error("{0} was not found")]
    NotFound(String),
    #[error("{0}")]
    Policy(String),
    #[error("Could not read or save Keel data: {0}")]
    Storage(#[from] std::io::Error),
    #[error("Keel data is not valid: {0}")]
    Data(#[from] serde_json::Error),
    #[error("The Engine could not complete the request: {0}")]
    Engine(String),
    #[error("The system credential store returned an error: {0}")]
    Credential(String),
}

impl Serialize for KeelError {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

pub type Result<T> = std::result::Result<T, KeelError>;
