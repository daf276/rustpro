#![forbid(unsafe_code)]

// A call site whose effect code no ported card registers yet: an effect reaching it marks the duel's result as
// untrustworthy ([Duel::unported_site]), and fails a debug assertion.
macro_rules! unported {
    ($duel:expr, $effects:expr $(,)?) => {
        if ::std::iter::IntoIterator::into_iter($effects).next().is_some() {
            $duel.reach_unported(concat!(file!(), ":", line!()));
        }
    };
}

// Clone whose clone_from goes field by field, so that copying into an existing value keeps its buffers.
macro_rules! clone_by_field {
    ($type:ty { $($field:ident),* $(,)? }) => {
        impl Clone for $type {
            fn clone(&self) -> Self {
                Self { $($field: self.$field.clone()),* }
            }

            fn clone_from(&mut self, source: &Self) {
                $(self.$field.clone_from(&source.$field);)*
            }
        }
    };
}

mod action;
mod buffer;
mod card;
mod chain;
pub mod constants;
mod duel;
mod effect;
mod field;
mod gamestate;
mod libcard;
mod libduel;
mod libeffect;
mod libgroup;
mod list;
mod operations;
mod playerop;
mod processor;
mod rng;
mod script;

pub use action::{Action, ActionError, BattleCommand, IdleCommand, Prompt};
pub use card::{CardId, CardSet, MAX_CARDS};
pub use constants::{DUEL_MODE_MR5, PHASE_DAMAGE, PHASE_DAMAGE_CAL};
pub use duel::{Duel, DuelError, DuelOptions, NewCardInfo, PlayerOptions, Status};
pub use effect::{
    ChainLimitFn, Condition, ConditionFn, CostFn, CounterFilter, EffectId, EffectSet, Event, ExtraMaterialCall, GroupOperationFn, OpTarget,
    Operation, OperationFn, SpSummonTargetFn, SynchroCheckFn, SynchroMaterialFn, Target, TargetFn, Value,
};
pub use gamestate::{DuelRecord, FormatError, PlacedCard, Position, PositionError, ReplayError};
pub use libduel::{ChainInfo, ExtraSummon, FromChainInfo, MonsterStats, ReleaseOptions};
pub use libeffect::CountCode;
pub use script::{Args, FusionCheck, InitialEffect, Interrupt, Library, Script};
