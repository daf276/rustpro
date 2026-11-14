// Earthbound Prison
use crate::prelude::*;
use crate::testing::{Check, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 71089030;

pub const LISTED_SERIES: &[u16] = &[SET_EARTHBOUND];
// In the hand with Earthbound Prisoner Stone Sweeper on the field and in the Graveyard, Sangan (an effect monster) on the opponent's
// field to negate, and the opponent's Mystical Space Typhoon to destroy it.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (72323266, MZONE),
    (72323266, GRAVE),
    (26202165, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
    (5318639, HAND | OPPONENT),
    (5318639, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(0), Check::ResolvesEffect(aux::stringid(ID, 2))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_target(duel, target);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_type(duel, EFFECT_TYPE_FIELD);
    e2.set_code(duel, EFFECT_EXTRA_SUMMON_COUNT);
    e2.set_range(duel, LOCATION_FZONE);
    e2.set_target_range(duel, LOCATION_HAND | LOCATION_MZONE, 0);
    e2.set_target_card(duel, |duel, _, c| c.is_set_card(duel, &[SET_EARTHBOUND]));
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 2));
    e3.set_category(duel, CATEGORY_DISABLE);
    e3.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_F);
    e3.set_code(duel, EVENT_DESTROYED);
    e3.set_condition(duel, discon);
    e3.set_target(duel, distg);
    e3.set_operation(duel, disop);
    c.register_effect(duel, e3);
}

fn target(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_controler(duel, 1 - tp)
            && chkc.is_location(duel, LOCATION_MZONE)
            && chkc.is_faceup(duel)
            && chkc.is_type(duel, TYPE_EFFECT));
    }
    if chk == 0 {
        return Ok(true);
    }
    let filter = aux::faceup_filter(|c, duel| c.is_type(duel, TYPE_EFFECT));
    if duel.is_existing_target(&filter, tp, 0, LOCATION_MZONE, 1, None) && duel.select_yes_no(tp, aux::stringid(ID, 0))? {
        e.set_property(duel, EFFECT_FLAG_CARD_TARGET);
        e.set_operation(duel, activate);
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_FACEUP);
        duel.select_target(tp, &filter, tp, 0, LOCATION_MZONE, 1, 1, None)?;
    } else {
        e.set_property(duel, 0);
        e.clear_operation(duel);
    }
    Ok(false)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if tc.is_faceup(duel) && tc.is_relate_to_effect(duel, a.e) {
        c.set_card_target(duel, tc);
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_FIELD);
        e1.set_code(duel, EFFECT_DISABLE);
        e1.set_range(duel, LOCATION_FZONE);
        e1.set_target_range(duel, LOCATION_MZONE, LOCATION_MZONE);
        e1.set_label_object_group(duel, Some(tc.into()));
        e1.set_target_card(duel, |duel, e, c| {
            e.get_label_object_group(duel).and_then(|g| g.get_first()) == Some(c) && e.get_handler(duel).is_has_card_target(duel, c)
        });
        e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
        c.register_effect(duel, e1);
    }
    Ok(())
}

fn discon(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    a.rp == 1 - a.tp && c.is_previous_controler(duel, a.tp) && c.is_reason(duel, REASON_EFFECT)
}

fn disconfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_EARTHBOUND]) && c.is_faceup(duel) && c.is_monster(duel)
}

fn distg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    Ok(chk == 0 && duel.is_existing_matching_card(disconfilter, a.tp, LOCATION_MZONE | LOCATION_GRAVE, 0, 1, None))
}

fn disop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.set_lp(1 - tp, (duel.get_lp(1 - tp) + 1) / 2);
    let g = duel.get_matching_group(CardId::is_negatable, tp, 0, LOCATION_ONFIELD, None);
    let c = a.e.get_handler(duel);
    for tc in g {
        tc.negate_effects(duel, c, RESET_PHASE | PHASE_END, true);
    }
    Ok(())
}
