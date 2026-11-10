// Bystial Saronir
use crate::prelude::*;
use crate::testing::{Check, DECK, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 60242223;

pub const LISTED_NAMES: &[u32] = &[ID];
// In the hand and on the field (Dark Hole sends it to the Graveyard), Bystial Magnamhut (a DARK monster) on the field and in the Graveyard to banish
// and in the Deck to send, and a monster of the opponent's to make the effect a Quick Effect.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, MZONE),
    (33854624, MZONE),
    (33854624, GRAVE),
    (33854624, DECK),
    (33854624, DECK),
    (53129443, HAND),
    (26202165, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_REMOVE + CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, |duel, a| !spquickcon(duel, a));
    e1.set_target(duel, sptg);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
    let e2 = e1.clone_effect(duel);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_code(duel, EVENT_FREE_CHAIN);
    e2.set_hint_timing(duel, 0, TIMINGS_CHECK_MONSTER_E);
    e2.set_condition(duel, spquickcon);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_TOGRAVE);
    e3.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e3.set_property(duel, EFFECT_FLAG_DELAY);
    e3.set_code(duel, EVENT_TO_GRAVE);
    e3.set_count_limit(duel, 1, (ID, 1));
    e3.set_target(duel, tgtg);
    e3.set_operation(duel, tgop);
    c.register_effect(duel, e3);
}

fn spfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_attribute(duel, ATTRIBUTE_LIGHT | ATTRIBUTE_DARK)
        && c.is_able_to_remove(duel)
        && aux::sp_elim_filter(c, duel, true, false)
        && c.is_faceup(duel)
        && duel.get_mzone_count(tp, Some(c.into())) > 0
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE | LOCATION_GRAVE) && spfilter(chkc, duel, tp));
    }
    let c = e.get_handler(duel);
    if chk == 0 {
        return Ok(duel.is_existing_target(
            |c, duel| spfilter(c, duel, tp),
            tp,
            LOCATION_MZONE | LOCATION_GRAVE,
            LOCATION_GRAVE,
            1,
            None,
        ) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_REMOVE);
    let g = duel
        .select_target(
            tp,
            |c, duel| spfilter(c, duel, tp),
            tp,
            LOCATION_MZONE | LOCATION_GRAVE,
            LOCATION_GRAVE,
            1,
            1,
            None,
        )?
        .unwrap_or_default();
    duel.set_operation_info(0, CATEGORY_REMOVE, Some(g), 1, tp, 0);
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(c.into()), 1, tp, LOCATION_HAND as i32);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if tc.is_relate_to_effect(duel, e)
        && duel.remove(tc, POS_FACEUP, REASON_EFFECT)? > 0
        && tc.is_location(duel, LOCATION_REMOVED)
        && c.is_relate_to_effect(duel, e)
    {
        duel.special_summon(c, 0, tp, tp, false, false, POS_FACEUP)?;
    }
    Ok(())
}

fn spquickcon(duel: &Duel, a: &Args) -> bool {
    duel.get_field_group_count(a.tp, 0, LOCATION_MZONE) > 0
}

fn tgfilter(c: CardId, duel: &Duel) -> bool {
    ((c.is_set_card(duel, &[SET_BYSTIAL]) && c.is_monster(duel) && !c.is_code(duel, ID))
        || (c.is_set_card(duel, &[SET_BRANDED]) && c.is_spell_trap(duel)))
        && c.is_able_to_grave(duel)
}

fn tgtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(tgfilter, tp, LOCATION_DECK, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_TOGRAVE, None, 1, tp, LOCATION_DECK as i32);
    Ok(false)
}

fn tgop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOGRAVE);
    let g = duel.select_matching_card(tp, tgfilter, tp, LOCATION_DECK, 0, 1, 1, None)?;
    if !g.is_empty() {
        duel.send_to_grave(g, REASON_EFFECT)?;
    }
    Ok(())
}
