// Number C62: Neo Galaxy-Eyes Prime Photon Dragon
use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE};

const ID: u32 = 48348921;
const PRIME_PHOTON: u32 = 31801517;

pub const LISTED_NAMES: &[u32] = &[PRIME_PHOTON, CARD_GALAXYEYES_P_DRAGON];

// In the Extra Deck, Xyz Summoned with three Level 8 LIGHT monsters or over Number 62: Galaxy-Eyes Prime Photon Dragon.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (PRIME_PHOTON, MZONE),
    (CARD_GALAXYEYES_P_DRAGON, MZONE),
    (89631139, MZONE),
    (89631139, MZONE),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::ResolvesEffect(aux::stringid(ID, 1))];

struct Procedure;

impl XyzProcedure for Procedure {
    const F: Option<XyzFilter> = Some(|c, duel, scard, sumtype, tp| c.is_attribute_full(duel, ATTRIBUTE_LIGHT, Some(scard), sumtype, tp));
    const LV: Option<u32> = Some(8);
    const CT: usize = 3;
    const ALTERF: Option<AlterFilter> = Some(ovfilter);
    const DESC: u64 = aux::stringid(ID, 0);
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Xyz::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 1));
    e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e1.set_code(duel, EVENT_PHASE | PHASE_BATTLE_START);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, 0);
    e1.set_condition(duel, |duel, a| duel.is_turn_player(a.tp));
    e1.set_cost(duel, Cost::detach_from_self::<1, 1>);
    e1.set_operation(duel, operation);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e2.set_code(duel, EFFECT_IMMUNE_EFFECT);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_condition(duel, |duel, a| has_photon_material(duel, a.e.get_handler(duel)));
    e2.set_value(duel, Value::Effect(efilter));
    c.register_effect(duel, e2);
    let e3 = e2.clone_effect(duel);
    e3.set_code(duel, EFFECT_UPDATE_ATTACK);
    e3.set_value(duel, Value::Card(atkval));
    c.register_effect(duel, e3);
}

fn ovfilter(c: CardId, duel: &Duel, tp: u8, lc: CardId) -> bool {
    c.is_faceup(duel) && c.is_summon_code(duel, Some(lc), SUMMON_TYPE_XYZ.into(), tp, &[PRIME_PHOTON])
}

fn has_photon_material(duel: &Duel, c: CardId) -> bool {
    c.get_overlay_group(duel)
        .is_exists(duel, |mc, duel| mc.is_code(duel, CARD_GALAXYEYES_P_DRAGON), 1, None)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    if !(c.is_relate_to_effect(duel, a.e) && c.is_faceup(duel)) {
        return Ok(());
    }
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 1));
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_CLIENT_HINT);
    e1.set_code(duel, EFFECT_EXTRA_ATTACK_MONSTER);
    e1.set_value(duel, 2);
    e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD | RESET_PHASE | PHASE_BATTLE);
    c.register_effect(duel, e1);
    Ok(())
}

fn efilter(duel: &Duel, e: EffectId, te: EffectId) -> bool {
    te.is_monster_effect(duel) && te.get_owner_player(duel) == 1 - e.get_handler_player(duel)
}

fn atkval(duel: &Duel, _: EffectId, c: CardId) -> i32 {
    let g = c.get_overlay_group(duel);
    let sum = g.get_sum(duel, |tc, duel| i64::from(tc.get_level(duel))) + g.get_sum(duel, |tc, duel| i64::from(tc.get_rank(duel)));
    sum as i32 * 100
}
