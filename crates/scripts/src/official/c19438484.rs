// Ilios the Black Sun Dragon
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND, MZONE, OPPONENT};

const ID: u32 = 19438484;

// In the hand with three Dragons of 3000 or more ATK in the Extra Deck to reveal, one of them on the field to protect once
// Ilios is sent to the Graveyard, and the opponent's Dark Hole for that.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (ID, MZONE),
    (23995346, EXTRA),
    (11443677, EXTRA),
    (11901678, EXTRA),
    (11901678, MZONE),
    (53129443, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_count_limit(duel, 1, (ID, 0));
    e1.set_cost(duel, spcost);
    e1.set_target(duel, stock::special_summon_self_target);
    e1.set_operation(duel, stock::special_summon_self::<POS_FACEUP>);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DELAY + EFFECT_FLAG_CARD_TARGET);
    e2.set_code(duel, EVENT_TO_GRAVE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_target(duel, efftg);
    e2.set_operation(duel, effop);
    c.register_effect(duel, e2);
}

fn spcostfilter(c: CardId, duel: &Duel) -> bool {
    c.is_race(duel, RACE_DRAGON) && c.is_attack_above(duel, 3000) && !c.is_public(duel)
}

fn spcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    let g = duel.get_matching_group(spcostfilter, tp, LOCATION_EXTRA, 0, None);
    if chk == 0 {
        return Ok(g.len() >= 3 && aux::select_unselect_group_check(duel, g, 3, 3, &aux::dncheck));
    }
    let sg = aux::select_unselect_group(duel, g, 3, 3, &aux::dncheck, tp, HINTMSG_CONFIRM)?;
    duel.confirm_cards(1 - tp, sg)?;
    duel.shuffle_extra(tp)?;
    Ok(false)
}

fn tgfilter(c: CardId, duel: &Duel) -> bool {
    c.is_race(duel, RACE_DRAGON) && c.is_type(duel, TYPE_EXTRA) && c.is_faceup(duel)
}

fn efftg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_controler(duel, tp) && chkc.is_location(duel, LOCATION_MZONE) && tgfilter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(tgfilter, tp, LOCATION_MZONE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TARGET);
    duel.select_target(tp, tgfilter, tp, LOCATION_MZONE, 0, 1, 1, None)?;
    duel.set_chain_limit(|_, _, _, _, _| false, 0);
    Ok(false)
}

fn effop(duel: &mut Duel, a: &Args) -> Script<()> {
    let e = a.e;
    if let Some(tc) = duel.get_first_target().filter(|tc| tc.is_relate_to_effect(duel, e)) {
        let e1 = duel.create_effect(e.get_handler(duel));
        e1.set_description(duel, 3061);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_CLIENT_HINT);
        e1.set_code(duel, EFFECT_CANNOT_BE_EFFECT_TARGET);
        e1.set_value(duel, Value::EffectPlayer(aux::tgoval));
        e1.set_reset(duel, RESETS_STANDARD_PHASE_END);
        tc.register_effect(duel, e1);
    }
    Ok(())
}
