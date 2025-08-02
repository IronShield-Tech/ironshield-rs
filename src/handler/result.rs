use crate::handler::error::ErrorHandler;

/// Type alias for function signatures.
pub type ResultHandler<T> = Result<T, ErrorHandler>;