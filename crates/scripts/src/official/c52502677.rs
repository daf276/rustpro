// Gladiator Beast Attorix
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, HAND, MZONE, OPPONENT};

const ID: u32 = 52502677;

pub const LISTED_SERIES: &[u16] = &[SET_GLADIATOR_BEAST];
pub const LISTED_NAMES: &[u32] = &[ID];
// Summoned from the hand with Gladiator Beast Gistel or by a tag-out from the Deck, with Hoplomus and Heraklinos to send to
// the GY and Hoplomus to summon.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, HAND),
    (ID, DECK),
    (26582143, HAND),
    (26582143, HAND),
    (4253484, MZONE),
    (4253484, DECK),
    (4253484, DECK),
    (27346636, EXTRA),
    (27125110, MZONE | OPPONENT),
    (27125110, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_DELAY);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_condition(duel, aux::gbspcon);
    e1.set_cost(duel, lvnamecost);
    e1.set_operation(duel, lvnameop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e2.set_code(duel, EVENT_PHASE + PHASE_BATTLE);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_condition(duel, |duel, a| a.e.get_handler(duel).get_battled_group_count(duel) > 0);
    e2.set_cost(duel, Cost::self_to_deck);
    e2.set_target(duel, |duel, a, chk, _| {
        super::c57731460::sptg(duel, a, chk, |c, duel| spfilter(c, duel, a.e, a.tp))
    });
    e2.set_operation(duel, |duel, a| {
        super::c57731460::spop(duel, a, |c, duel| spfilter(c, duel, a.e, a.tp), 102, true, 0)
    });
    c.register_effect(duel, e2);
}

fn lvnamecostfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_GLADIATOR_BEAST])
        && c.is_monster(duel)
        && c.has_level(duel)
        && !c.is_code(duel, ID)
        && c.is_able_to_grave_as_cost(duel)
}

fn lvnamecost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(lvnamecostfilter, tp, LOCATION_DECK | LOCATION_EXTRA, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOGRAVE);
    let g = duel.select_matching_card(tp, lvnamecostfilter, tp, LOCATION_DECK | LOCATION_EXTRA, 0, 1, 1, None)?;
    if let Some(sc) = g.get_first() {
        duel.send_to_grave(sc, REASON_COST)?;
        let labels = [i64::from(sc.get_level(duel)), i64::from(sc.get_code(duel))];
        a.e.set_labels(duel, &labels);
    }
    Ok(false)
}

fn lvnameop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    if c.is_relate_to_effect(duel, a.e) && c.is_faceup(duel) {
        let &[lv, code] = a.e.get_labels(duel) else { return Ok(()) };
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_CHANGE_LEVEL);
        e1.set_value(duel, lv as i32);
        e1.set_reset(duel, RESETS_STANDARD_DISABLE_PHASE_END);
        c.register_effect(duel, e1);
        let e2 = duel.create_effect(c);
        e2.set_type(duel, EFFECT_TYPE_SINGLE);
        e2.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
        e2.set_code(duel, EFFECT_CHANGE_CODE);
        e2.set_value(duel, code as i32);
        e2.set_reset(duel, RESETS_STANDARD_PHASE_END);
        c.register_effect(duel, e2);
    }
    Ok(())
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_GLADIATOR_BEAST]) && !c.is_code(duel, ID) && c.is_can_be_special_summoned(duel, e, 102, tp, false, false)
}
