// Salamangreat Jack Jaguar
use crate::prelude::*;
use crate::testing::{GRAVE, MZONE};

const ID: u32 = 56003780;

pub const LISTED_SERIES: &[u16] = &[SET_SALAMANGREAT];
pub const LISTED_NAMES: &[u32] = &[ID];

// Salamangreat Pyro Phoenix on the field points at the Main Monster Zones beside it; Salamangreat Gazelle in the GY to shuffle.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, GRAVE),
    (31313405, MZONE),
    (31313405, MZONE),
    (26889158, GRAVE),
    (26889158, GRAVE),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_PIERCE);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_TODECK);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_count_limit(duel, 1, ID);
    e2.set_condition(duel, spcon);
    e2.set_target(duel, sptg);
    e2.set_operation(duel, spop);
    c.register_effect(duel, e2);
}

fn cfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_SALAMANGREAT]) && c.is_link_monster(duel) && c.is_faceup(duel)
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    duel.is_existing_matching_card(cfilter, a.tp, LOCATION_MZONE, 0, 1, None)
}

fn tdfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_SALAMANGREAT]) && c.is_monster(duel) && !c.is_code(duel, ID) && c.is_able_to_deck(duel)
}

fn zone(duel: &Duel, tp: u8) -> u32 {
    let salamangreat = |c: CardId, duel: &Duel| c.is_set_card(duel, &[SET_SALAMANGREAT]);
    aux::get_mm_zones_pointed_to_full(duel, tp, salamangreat, LOCATION_MZONE, 0, tp)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_controler(duel, tp) && chkc.is_location(duel, LOCATION_GRAVE) && tdfilter(chkc, duel));
    }
    let c = e.get_handler(duel);
    let zone = zone(duel, tp);
    if chk == 0 {
        return Ok(zone > 0
            && duel.is_existing_target(tdfilter, tp, LOCATION_GRAVE, 0, 1, Some(c.into()))
            && c.is_can_be_special_summoned_full(duel, e, 0, tp, false, false, POS_FACEUP, tp, zone));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TODECK);
    let g = duel.select_target(tp, tdfilter, tp, LOCATION_GRAVE, 0, 1, 1, Some(c.into()))?;
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(c.into()), 1, 0, 0);
    duel.set_operation_info(0, CATEGORY_TODECK, g, 1, tp, LOCATION_GRAVE as i32);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    let zone = zone(duel, tp);
    if tc.is_relate_to_effect(duel, e)
        && tdfilter(tc, duel)
        && duel.send_to_deck(tc, PLAYER_NONE, SEQ_DECKSHUFFLE, REASON_EFFECT)? > 0
        && tc.is_location(duel, LOCATION_DECK | LOCATION_EXTRA)
        && c.is_relate_to_effect(duel, e)
        && zone > 0
    {
        duel.special_summon_full(c, 0, tp, tp, false, false, POS_FACEUP, zone)?;
    }
    Ok(())
}
