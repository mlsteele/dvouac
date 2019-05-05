pub type Result<T> = std::result::Result<T, failure::Error>;
pub use std::result::Result::Ok;

pub type EResult = Result<()>;
#[allow(dead_code)]
pub const EOK: EResult = Ok(());
