use std::fmt;
#[derive(Debug)]
pub enum DomainError {
    InvalidInput(String),
    NotFound(String),
    Conflict(String),
    Storage(String),
}
impl fmt::Display for DomainError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DomainError::InvalidInput(m)
            | DomainError::NotFound(m)
            | DomainError::Conflict(m)
            | DomainError::Storage(m) => write!(f, "{}", m),
        }
    }
}
impl std::error::Error for DomainError {}
