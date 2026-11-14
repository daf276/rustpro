// Gem-Knight Master Diamond
use super::c3113836::{gem_knight, register_splimit};
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, HAND, MZONE};

const ID: u32 = 39512984;

pub const LISTED_SERIES: &[u16] = &[SET_GEM, SET_GEM_KNIGHT];
pub const MATERIAL_SETCODE: &[u16] = &[SET_GEM, SET_GEM_KNIGHT];

const RUBY: u32 = 76614340;

struct Materials;

impl FusionMix for Materials {
    const SUB: bool = true;
    const INSF: bool = true;
    const MATERIALS: &'static [FusionMaterial] = &[FusionMaterial::Filter(gem_knight); 3];
}

// In the Extra Deck, Fusion Summoned by Polymerization with three Gem-Knight Garnet; on the field with Gem-Knight Ruby in the GY to
// copy, and Garnet beside it to release for Ruby's effect.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, MZONE),
    (CARD_POLYMERIZATION, HAND),
    (91731841, HAND),
    (91731841, HAND),
    (91731841, HAND),
    (91731841, MZONE),
    (RUBY, GRAVE),
    (RUBY, GRAVE),
];
pub const CHECKS: &[Check] = &[
    Check::SpecialSummoned,
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(RUBY, 0)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Fusion::add_proc_mix_n::<Materials>(duel, c);
    register_splimit(duel, c);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_code(duel, EFFECT_UPDATE_ATTACK);
    e2.set_value(duel, Value::Card(atkup));
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 0));
    e3.set_type(duel, EFFECT_TYPE_IGNITION);
    e3.set_count_limit(duel, 1, 0);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_cost(duel, cost);
    e3.set_operation(duel, operation);
    c.register_effect(duel, e3);
}

fn atkfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_GEM]) && c.is_monster(duel)
}

fn atkup(duel: &Duel, _: EffectId, c: CardId) -> i32 {
    duel.get_matching_group_count(atkfilter, c.get_controler(duel), LOCATION_GRAVE, 0, None) as i32 * 100
}

fn filter(c: CardId, duel: &Duel) -> bool {
    c.is_level_below(duel, 7)
        && c.is_set_card(duel, &[SET_GEM_KNIGHT])
        && c.is_type(duel, TYPE_FUSION)
        && c.is_able_to_remove_as_cost(duel)
        && aux::sp_elim_filter(c, duel, true, false)
}

fn cost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let handler = Some(e.get_handler(duel).into());
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(filter, tp, LOCATION_MZONE | LOCATION_GRAVE, 0, 1, handler));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_REMOVE);
    let g = duel.select_matching_card(tp, filter, tp, LOCATION_MZONE | LOCATION_GRAVE, 0, 1, 1, handler)?;
    duel.remove(g, POS_FACEUP, REASON_COST)?;
    let code = g.get_first().map_or(0, |tc| tc.get_original_code(duel));
    e.set_label(duel, i64::from(code));
    Ok(false)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let e = a.e;
    let c = e.get_handler(duel);
    let code = e.get_label(duel) as u32;
    if c.is_relate_to_effect(duel, e) && c.is_faceup(duel) {
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
        e1.set_reset(duel, RESETS_STANDARD_PHASE_END);
        e1.set_code(duel, EFFECT_CHANGE_CODE);
        e1.set_value(duel, code as i32);
        c.register_effect(duel, e1);
        c.copy_effect(duel, code, RESETS_STANDARD_PHASE_END, 1);
    }
    Ok(())
}
