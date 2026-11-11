// Conduction Warrior Plasma Magnum
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND};

const ID: u32 = 47247792;

struct Materials;

impl FusionMix for Materials {
    const SUB: bool = true;
    const INSF: bool = true;
    const MATERIALS: &'static [FusionMaterial] = &[FusionMaterial::Filter(|c, duel, fc, sumtype, tp| {
        c.is_attribute_full(duel, ATTRIBUTE_EARTH, Some(fc), sumtype, tp) && c.is_race_full(duel, RACE_ROCK, Some(fc), sumtype, tp)
    }); 2];
}

// In the Extra Deck, Fusion Summoned by Polymerization with Alpha and Beta. No Magnet Spell or Level 8 Magna Warrior is
// ported, so its Deck effects are unreachable.
pub const SUPPORT: &[(u32, u16)] = &[(ID, EXTRA), (CARD_POLYMERIZATION, HAND), (99785935, HAND), (39256679, HAND)];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Fusion::add_proc_mix_n::<Materials>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_CHANGE_ATTRIBUTE);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_target_range(duel, LOCATION_MZONE, LOCATION_MZONE);
    e1.set_value(duel, ATTRIBUTE_EARTH as i32);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DELAY);
    e2.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e2.set_count_limit(duel, 1, ID);
    e2.set_condition(duel, |duel, a| a.e.get_handler(duel).is_fusion_summoned(duel));
    e2.set_target(duel, thtg);
    e2.set_operation(duel, thop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_SUMMON);
    e3.set_type(duel, EFFECT_TYPE_IGNITION);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_count_limit(duel, 1, (ID, 1));
    e3.set_cost(duel, nscost);
    e3.set_target(duel, nstg);
    e3.set_operation(duel, nsop);
    c.register_effect(duel, e3);
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_MAGNET])
        && (c.is_normal_spell(duel) || c.is_quick_play_spell(duel) || c.is_field_spell(duel))
        && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, thfilter, LOCATION_DECK, CATEGORY_TOHAND)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::search(duel, a, thfilter, LOCATION_DECK)
}

fn nscostfilter(c: CardId, duel: &Duel) -> bool {
    c.is_level(duel, &[8]) && c.is_set_card(duel, &[SET_MAGNA_WARRIOR]) && c.is_able_to_grave_as_cost(duel)
}

fn nscost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    stock::send_to_grave_cost(duel, a, chk, nscostfilter, LOCATION_DECK)
}

fn nsfilter(c: CardId, duel: &Duel) -> bool {
    c.is_level_below(duel, 4) && c.is_set_card(duel, &[SET_MAGNET_WARRIOR]) && c.is_summonable(duel, true, None)
}

fn nstg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, nsfilter, LOCATION_HAND | LOCATION_MZONE, CATEGORY_SUMMON)
}

fn nsop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SUMMON);
    let g = duel.select_matching_card(tp, nsfilter, tp, LOCATION_HAND | LOCATION_MZONE, 0, 1, 1, None)?;
    if let Some(sc) = g.get_first() {
        duel.summon(tp, sc, true, None)?;
    }
    Ok(())
}
