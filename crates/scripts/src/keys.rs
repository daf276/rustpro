// Script-state and script-group keys of the library and of the Lua globals cards share. A card's own keys are
// `ID << 16 | slot`, below bit 48; each kind here has its own tag in the top byte, so no two kinds can share a key.
// Top byte 0x40 is taken by the engine's script groups in `libeffect.rs`.
use engine::{CardId, EffectId};

#[repr(u8)]
enum Tag {
    HandShuffled = 1,
    DelayedOperationId,
    FusionCheckExact,
    FusionCheckMin,
    FusionCheckMax,
    FusionGlobalEffect,
    FusionExtraGroup,
    FusionChkf,
    FusionMaterial,
    XyzProcCancellable,
    ReincarnationChecked,
    KeptExtraMaterials,
    GlobalCheck,
    ChainData,
    Registering,
    ReplaceCount,
    ReplaceMax,
    DelayedTurn,
    SynchroSend,
}

const fn key(tag: Tag, payload: u64) -> u64 {
    (tag as u64) << 56 | payload & ((1 << 56) - 1)
}

// player_table of proc_workaround.lua: whether a player's hand was already shuffled in the current chain.
pub fn hand_shuffled(player: u8) -> u64 {
    key(Tag::HandShuffled, player.into())
}

// delayed_operation_id, the global counter of utility.lua.
pub const DELAYED_OPERATION_ID: u64 = key(Tag::DelayedOperationId, 0);
// Fusion.CheckExact, CheckMin and CheckMax, 0 for nil.
pub const FUSION_CHECK_EXACT: u64 = key(Tag::FusionCheckExact, 0);
pub const FUSION_CHECK_MIN: u64 = key(Tag::FusionCheckMin, 0);
pub const FUSION_CHECK_MAX: u64 = key(Tag::FusionCheckMax, 0);
// The field id of proc_fusion_spell's global EFFECT_EXTRA_FUSION_MATERIAL.
pub const FUSION_GLOBAL_EFFECT: u64 = key(Tag::FusionGlobalEffect, 0);
// Fusion.ExtraGroup, a script group.
pub const FUSION_EXTRA_GROUP: u64 = key(Tag::FusionExtraGroup, 0);
// Xyz.ProcCancellable, set by the target of the alternative procedure.
pub const XYZ_PROC_CANCELLABLE: u64 = key(Tag::XyzProcCancellable, 0);
// ReincarnationChecked, the upvalue guarding Auxiliary.EnableCheckReincarnation's global effect.
pub const REINCARNATION_CHECKED: u64 = key(Tag::ReincarnationChecked, 0);
// Synchro.Send, where Synchro.Operation sends the materials.
pub const SYNCHRO_SEND: u64 = key(Tag::SynchroSend, 0);
// _replace_count and _replace_max, the Lua globals of the draw-replacing cards.
pub const REPLACE_COUNT: u64 = key(Tag::ReplaceCount, 0);
pub const REPLACE_MAX: u64 = key(Tag::ReplaceMax, 0);

// The chkf upvalue of a SummonEffTG (`op` false) or SummonEffOP closure, per card that created it.
pub fn fusion_chkf(owner: CardId, op: bool) -> u64 {
    key(Tag::FusionChkf, (owner.index() as u64) << 1 | u64::from(op))
}

// mt.material: the `index`th material code of the Fusion Monster with this original code.
pub fn fusion_material(code: u32, index: u64) -> u64 {
    key(Tag::FusionMaterial, u64::from(code) << 8 | index)
}

// Link.Target's label object: the packed filter indices at `index` 0 and the emt groups, both per effect.
pub fn kept_extra_materials(effect: EffectId, index: usize) -> u64 {
    key(Tag::KeptExtraMaterials, (effect.index() as u64) << 16 | index as u64)
}

// s.global_check per script, keyed by the script's id.
pub fn global_check(id: u32) -> u64 {
    key(Tag::GlobalCheck, id.into())
}

// e:GetChainData() of a chain link, keyed by its chain id and the effect; a `slot` per field of the table.
pub fn chain_data(chain_id: u32, effect: EffectId, slot: u8) -> u64 {
    key(
        Tag::ChainData,
        u64::from(chain_id) << 24 | (effect.index() as u64) << 8 | u64::from(slot),
    )
}

// The turn a card's delayed effect was set up in, keyed by the script's id and the effect's first label (a fid).
pub fn delayed_turn(id: u32, fid: i64) -> u64 {
    key(Tag::DelayedTurn, u64::from(id) << 28 | fid as u64 & ((1 << 28) - 1))
}

// The card chain.lua's Effect.RegisterEffect override records for an effect, a script group.
pub fn registering(effect: EffectId) -> u64 {
    key(Tag::Registering, effect.index() as u64)
}
