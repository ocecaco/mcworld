#![warn(clippy::all)]
mod pos;
pub mod raw;
mod table;
mod world;

mod error {
    pub use failure::Error;
    pub type Result<T> = ::std::result::Result<T, Error>;
}

pub use crate::world::*;
pub use crate::pos::*;
pub use crate::table::BlockId;
