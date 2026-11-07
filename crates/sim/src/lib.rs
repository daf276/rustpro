#![forbid(unsafe_code)]

mod batch;
mod encoding;
mod tensor;

pub use batch::{AutoAdvance, Batch, BatchError, Options, Outcome, auto_advance};
pub use encoding::{ACTIONS, EncodingError, Section, decode, encode, find, indices, slots, write_mask};
pub use tensor::{ACTION_FEATURES, CARD_FEATURES, CHAIN_FEATURES, GLOBAL_FEATURES, MAX_CHAIN, MAX_OBS_CARDS, PROMPTS, Tensors};
