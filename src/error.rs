#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TeError {
    /// Parse error at the given character offset (1-based when offset == 0).
    ParseError(i32),
    /// Out-of-memory or other internal failure.
    Internal,
}

impl TeError {
    pub fn offset(self) -> Option<i32> {
        match self {
            TeError::ParseError(o) => Some(o),
            TeError::Internal => None,
        }
    }
}

pub type TeResult<T> = Result<T, TeError>;