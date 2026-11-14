// Goblin Bikers Gone Wild
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND, MZONE, OPPONENT};

const ID: u32 = 55320758;

pub const LISTED_SERIES: &[u16] = &[SET_GOBLIN];

const GOBLIN_BIKERS_GONE_BONKERS: u32 = 46014517;

// In the hand with Goblin Zombie and Goblin Attack Force in the Deck to summon, and on the field with Goblin Bikers Gone Bonkers
// to attach to when Raigeki sends it to the Graveyard.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (ID, MZONE),
    (63665875, DECK),
    (63665875, DECK),
    (78658564, DECK),
    (GOBLIN_BIKERS_GONE_BONKERS, MZONE),
    (12580477, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::ResolvesEffect(aux::stringid(ID, 2)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_UNCOPYABLE);
    e1.set_code(duel, EFFECT_SUMMON_PROC);
    e1.set_condition(duel, sumcon);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DELAY);
    e2.set_code(duel, EVENT_SUMMON_SUCCESS);
    e2.set_count_limit(duel, 1, ID);
    e2.set_target(duel, sptg);
    e2.set_operation(duel, spop);
    c.register_effect(duel, e2);
    let e3 = e2.clone_effect(duel);
    e3.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_description(duel, aux::stringid(ID, 2));
    e4.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e4.set_property(duel, EFFECT_FLAG_CARD_TARGET + EFFECT_FLAG_DELAY);
    e4.set_code(duel, EVENT_TO_GRAVE);
    e4.set_count_limit(duel, 1, (ID, 1));
    e4.set_target(duel, attachtg);
    e4.set_operation(duel, attachop);
    c.register_effect(duel, e4);
}

fn sumconfilter(c: CardId, duel: &Duel) -> bool {
    c.is_facedown(duel) || !c.is_set_card(duel, &[SET_GOBLIN])
}

fn sumcon(duel: &Duel, a: &Args) -> bool {
    let Some(c) = a.c else { return true };
    let tp = c.get_controler(duel);
    a.min == 0
        && c.is_level_above(duel, 5)
        && duel.get_location_count_full(tp, LOCATION_MZONE, tp, LOCATION_REASON_TOFIELD, a.zone) > 0
        && (duel.get_field_group_count(tp, LOCATION_MZONE, 0) == 0
            || !duel.is_existing_matching_card(sumconfilter, tp, LOCATION_MZONE, 0, 1, None))
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_GOBLIN]) && c.is_level_below(duel, 4) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::special_summon_target(duel, a, chk, |c, duel| spfilter(c, duel, a.e, a.tp), LOCATION_DECK)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::special_summon(duel, a, |c, duel| spfilter(c, duel, a.e, a.tp), LOCATION_DECK, POS_FACEUP)
}

fn xyzfilter(c: CardId, duel: &Duel, mc: CardId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_GOBLIN])
        && c.is_type(duel, TYPE_XYZ)
        && c.is_faceup(duel)
        && mc.is_can_be_xyz_material(duel, Some(c), tp, REASON_EFFECT)
}

fn attachfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    duel.is_existing_matching_card(|xc, duel| xyzfilter(xc, duel, c, tp), tp, LOCATION_MZONE, 0, 1, Some(c.into()))
}

fn attachtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_on_field(duel) && attachfilter(chkc, duel, tp));
    }
    let filter = |c, duel: &Duel| attachfilter(c, duel, tp);
    if chk == 0 {
        return Ok(duel.is_existing_target(filter, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATTACH);
    duel.select_target(tp, filter, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, 1, 1, None)?;
    Ok(false)
}

fn attachop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if !tc.is_relate_to_effect(duel, e) {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SELF);
    let g = duel.select_matching_card(
        tp,
        |c, duel| xyzfilter(c, duel, tc, tp),
        tp,
        LOCATION_MZONE,
        0,
        1,
        1,
        Some(tc.into()),
    )?;
    let Some(xyzc) = g.get_first() else { return Ok(()) };
    duel.hint_selection(xyzc);
    if xyzc.is_immune_to_effect(duel, e) || tc.is_immune_to_effect(duel, e) {
        return Ok(());
    }
    duel.overlay_full(xyzc, tc, true)?;
    Ok(())
}
