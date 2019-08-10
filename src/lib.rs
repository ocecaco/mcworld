#![warn(clippy::all)]
pub mod pos;
pub mod raw;
mod table;
pub mod world;

pub mod error {
    pub use failure::Error;
    pub type Result<T> = ::std::result::Result<T, Error>;
}
