#![forbid(unsafe_code)]

// Ports of CardScripts: the Lua the oracle loads, as Rust against the engine's mirror of the Lua API.

// Marks a path the oracle takes that comes with the cards that reach it, as the engine's own macro does.
macro_rules! unported {
    ($duel:expr, $reached:expr $(,)?) => {
        if $reached {
            $duel.reach_unported(concat!(file!(), ":", line!()));
        }
    };
}

pub mod chain;
mod constant;
mod official;
mod proc_fusion;
mod proc_gemini;
mod proc_link;
mod proc_normal;
mod proc_pendulum;
mod proc_persistent;
mod proc_spirit;
mod proc_synchro;
mod proc_workaround;
mod proc_xyz;
mod utility;

use engine::{Duel, Library};

pub use official::{CARD_SCRIPTS, CLASSIC_CARDS, card_script};

pub mod prelude {
    pub use engine::{Args, CardId, CardSet, CounterFilter, Duel, EffectId, FusionCheck, Script, Value};

    pub use crate::chain::{self, DuelChain, EffectChain};
    pub use crate::constant::*;
    pub use crate::proc_fusion::{ContactFusion, DuelFusion, Fusion, FusionMaterial, FusionMix, FusionMixRep, FusionSummonParams};
    pub use crate::proc_gemini::Gemini;
    pub use crate::proc_link::{Link, LinkProcedure};
    pub use crate::proc_normal::{Normal, NormalProcedure};
    pub use crate::proc_pendulum::Pendulum;
    pub use crate::proc_persistent::{Persistent, PersistentProcedure};
    pub use crate::proc_spirit::Spirit;
    pub use crate::proc_synchro::{Synchro, SynchroProcedure};
    pub use crate::proc_workaround::{CardWorkaround, DuelWorkaround};
    pub use crate::proc_xyz::{AlterFilter, Xyz, XyzFilter, XyzProcedure};
    pub use crate::utility::{CardUtility, Cost, DuelUtility, EffectUtility, aux};
}

// What loading utility.lua registers. proc_fusion_spell.lua, proc_skill.lua and proc_maximum.lua also register global
// effects on load; they act only on materials in Fusion.ExtraGroup that are not monsters (see proc_fusion.rs), skills
// and Maximum monsters, none of which is ported.
fn load(duel: &mut Duel) {
    proc_workaround::load(duel);
}

pub static LIBRARY: Library = Library { load, card_script };
