#![forbid(unsafe_code)]

mod batch;
mod encoding;

pub use batch::{AutoAdvance, Batch, BatchError, Options, Outcome, auto_advance};
pub use encoding::{ACTIONS, EncodingError, Section, decode, encode, find, indices, slots, write_mask};
