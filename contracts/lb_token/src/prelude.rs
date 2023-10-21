// TODO - decide if using custom Error and Result type or not

// Use this crate's custom Error type
pub use crate::error::LBTokenError as Error;

// Force all Result types to use our Error type
pub type Result<T> = core::result::Result<T, Error>;
