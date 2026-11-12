// Time Thief Perpetua
use crate::prelude::*;
use crate::testing::{DECK, EXTRA, GRAVE, MZONE};

const ID: u32 = 59208943;

pub const LISTED_NAMES: &[u32] = &[ID];

// In the Extra Deck, Xyz Summoned from Level 4 Normal Monsters, with Time Thief Winder in the Graveyard and Deck and Time Thief Redoer on the field.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (9491461, MZONE),
    (9491461, MZONE),
    (23720856, MZONE),
    (23720856, MZONE),
    (56308388, GRAVE),
    (56308388, GRAVE),
    (56308388, DECK),
    (56308388, DECK),
    (55285840, MZONE),
];

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(4);
    const CT: usize = 2;
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Xyz::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e1.set_code(duel, EVENT_PHASE | PHASE_STANDBY);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, ID);
    e1.set_cost(duel, Cost::detach_from_self::<1, 1>);
    e1.set_target(duel, sptg);
    e1.set_operation(duel, stock::special_summon_first_target::<POS_FACEUP>);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_code(duel, EVENT_FREE_CHAIN);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_target(duel, mattg);
    e2.set_operation(duel, matop);
    c.register_effect(duel, e2);
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_TIME_THIEF]) && !c.is_code(duel, ID) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_GRAVE) && chkc.is_controler(duel, tp) && spfilter(chkc, duel, e, tp));
    }
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0
            && duel.is_existing_target(|c, duel| spfilter(c, duel, e, tp), tp, LOCATION_GRAVE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_target(tp, |c, duel| spfilter(c, duel, e, tp), tp, LOCATION_GRAVE, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, g, 1, 0, 0);
    Ok(false)
}

fn xyzfilter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_type(duel, TYPE_XYZ)
}

fn thieffilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_TIME_THIEF])
}

fn mattg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let c = a.e.get_handler(duel);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_controler(duel, tp) && chkc.is_location(duel, LOCATION_MZONE) && xyzfilter(chkc, duel) && chkc != c);
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(xyzfilter, tp, LOCATION_MZONE, 0, 1, Some(c.into()))
            && duel.is_existing_matching_card(thieffilter, tp, LOCATION_DECK, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_FACEUP);
    duel.select_target(tp, xyzfilter, tp, LOCATION_MZONE, 0, 1, 1, Some(c.into()))?;
    Ok(false)
}

fn matop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if let Some(tc) = duel.get_first_target()
        && tc.is_faceup(duel)
        && tc.is_relate_to_effect(duel, e)
        && !tc.is_immune_to_effect(duel, e)
    {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_XMATERIAL);
        let g = duel.select_matching_card(tp, thieffilter, tp, LOCATION_DECK, 0, 1, 1, None)?;
        if !g.is_empty() {
            duel.overlay(tc, g)?;
        }
    }
    Ok(())
}
