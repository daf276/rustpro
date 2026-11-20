#![forbid(unsafe_code)]

mod batch;
mod encoding;
mod features;
mod mask;
mod tensor;
mod tracker;

pub use batch::{AutoAdvance, Batch, BatchError, Options, Outcome, auto_advance};
pub use encoding::{ACTIONS, EncodingError, Section, decode, encode, find, indices, slots, write_mask};
pub use features::{Group, HIDDEN, OTHER, Place, Vocabulary, features};
pub use mask::{frames, mask};
pub use tensor::{
    ACTION_FEATURES, Buffers, CARD_FEATURES, CHAIN_FEATURES, GLOBAL_FEATURES, MAX_CHAIN, MAX_OBS_CARDS, PROMPTS, SHAPES, Tensors,
};
pub use tracker::Tracker;
