#![warn(clippy::all)]
mod table;
pub mod world;
pub mod pos;
pub mod raw;

pub mod error {
    pub use failure::Error;
    pub type Result<T> = ::std::result::Result<T, Error>;
}
