// Spiritual Beast Tamer Winda
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, HAND, MZONE, OPPONENT};

const ID: u32 = 65193366;

pub const LISTED_SERIES: &[u16] = &[SET_RITUAL_BEAST];
// Copies on the field for the opponent's Raigeki to destroy, with Ritual Beast monsters in the Deck and the Extra Deck to summon.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, MZONE),
    (ID, MZONE),
    (88123329, DECK),
    (14513016, DECK),
    (58811192, EXTRA),
    (12580477, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.set_sp_summon_once(duel, ID);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_DELAY);
    e1.set_code(duel, EVENT_DESTROYED);
    e1.set_condition(duel, |duel, a| {
        a.rp == 1 - a.tp && a.e.get_handler(duel).is_previous_controler(duel, a.tp)
    });
    e1.set_target(duel, sptg);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    if c.is_location(duel, LOCATION_EXTRA) && duel.get_location_count_from_ex(tp, tp, None, c) == 0 {
        return false;
    }
    c.is_set_card(duel, &[SET_RITUAL_BEAST]) && c.is_monster(duel) && c.is_can_be_special_summoned(duel, e, 0, tp, true, false)
}

fn summon_locations(duel: &Duel, tp: u8) -> u32 {
    match duel.get_location_count(tp, LOCATION_MZONE) > 0 {
        true => LOCATION_EXTRA | LOCATION_DECK,
        false => LOCATION_EXTRA,
    }
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let loc = summon_locations(duel, tp);
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(|c, duel| spfilter(c, duel, e, tp), tp, loc, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, loc as i32);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let loc = summon_locations(duel, tp);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_matching_card(tp, |c, duel| spfilter(c, duel, e, tp), tp, loc, 0, 1, 1, None)?;
    if !g.is_empty() {
        duel.special_summon(g, 0, tp, tp, true, false, POS_FACEUP)?;
    }
    Ok(())
}
