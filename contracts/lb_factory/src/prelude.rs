// Use this crate's custom Error type
pub use crate::error::LbFactoryError as Error;

// Force all Result types to use our Error type
pub type Result<T, E = Error> = core::result::Result<T, E>;
