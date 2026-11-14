// Gem-Knight Lady Rose Diamond
use super::c3113836::gem_knight;
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, HAND, MZONE};

const ID: u32 = 55610199;

pub const LISTED_SERIES: &[u16] = &[SET_GEM_KNIGHT];
pub const MATERIAL_SETCODE: &[u16] = &[SET_GEM, SET_GEM_KNIGHT];

struct Materials;

impl FusionMix for Materials {
    const SUB: bool = true;
    const INSF: bool = true;
    const MATERIALS: &'static [FusionMaterial] = &[
        FusionMaterial::Filter(gem_knight),
        FusionMaterial::Filter(|c, duel, fc, sumtype, tp| c.is_race_full(duel, RACE_FAIRY, Some(fc), sumtype, tp)),
    ];
}

// In the Extra Deck, Fusion Summoned by Polymerization with Gem-Knight Nepyrim (Fairy) and Garnet; on the field with Garnet in the
// GY to banish for the destruction.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, MZONE),
    (CARD_POLYMERIZATION, HAND),
    (51831560, HAND),
    (91731841, HAND),
    (91731841, GRAVE),
    (91731841, GRAVE),
];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Fusion::add_proc_mix::<Materials>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_INDESTRUCTABLE_COUNT);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_target_range(duel, LOCATION_MZONE, 0);
    e1.set_target_card(duel, |duel, _, c| c.is_set_card(duel, &[SET_GEM_KNIGHT]));
    e1.set_value(duel, Value::Reason(indval));
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_DESTROY);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_code(duel, EVENT_CHAINING);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_condition(duel, descon);
    e2.set_cost(duel, descost);
    e2.set_target(duel, destg);
    e2.set_operation(duel, stock::destroy_first_target);
    c.register_effect(duel, e2);
}

fn indval(duel: &Duel, e: EffectId, _: Option<EffectId>, r: u32, _: u8) -> bool {
    duel.is_turn_player(1 - e.get_handler_player(duel)) && r & REASON_EFFECT != 0
}

fn descon(duel: &Duel, a: &Args) -> bool {
    a.ep == 1 - a.tp && duel.is_turn_player(a.tp) && a.re.is_some_and(|re| re.is_monster_effect(duel))
}

fn descfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_GEM_KNIGHT]) && c.is_able_to_remove_as_cost(duel) && aux::sp_elim_filter(c, duel, true, false)
}

fn descost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    stock::banish_cost(duel, a, chk, descfilter, LOCATION_MZONE | LOCATION_GRAVE)
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_on_field(duel) && chkc.is_controler(duel, 1 - tp) && chkc.is_faceup(duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(CardId::is_faceup, tp, 0, LOCATION_ONFIELD, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
    let g = duel.select_target(tp, CardId::is_faceup, tp, 0, LOCATION_ONFIELD, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_DESTROY, g, 1, 0, 0);
    Ok(false)
}
