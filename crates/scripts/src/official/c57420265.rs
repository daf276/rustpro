// Fishborg Harpooner
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND, MZONE, OPPONENT};

const ID: u32 = 57420265;

// In the hand with Slime Toad, and in the Graveyard as material for Brionac, Dragon of the Ice Barrier against Sangan.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (ID, MZONE),
    (68638985, HAND),
    (68638985, MZONE),
    (68638985, MZONE),
    (50321796, EXTRA),
    (50321796, EXTRA),
    (26202165, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_HANDES);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_count_limit(duel, 1, ID);
    e1.set_target(duel, sptg);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_DISABLE);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DELAY);
    e2.set_code(duel, EVENT_BE_MATERIAL);
    e2.set_condition(duel, discon);
    e2.set_target(duel, distg);
    e2.set_operation(duel, disop);
    c.register_effect(duel, e2);
}

fn spcostfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8, dc_chk: bool, sp_chk: bool) -> bool {
    c.is_attribute(duel, ATTRIBUTE_WATER)
        && !c.is_public(duel)
        && ((sp_chk && c.is_discardable_full(duel, REASON_EFFECT, None))
            || (dc_chk && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)))
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let dc_chk = c.is_discardable_full(duel, REASON_EFFECT, None);
    let sp_chk = c.is_can_be_special_summoned(duel, e, 0, tp, false, false);
    let filter = |c, duel: &Duel| spcostfilter(c, duel, e, tp, dc_chk, sp_chk);
    if chk == 0 {
        return Ok((dc_chk || sp_chk)
            && !c.is_public(duel)
            && duel.get_location_count(tp, LOCATION_MZONE) > 0
            && duel.is_existing_matching_card(filter, tp, LOCATION_HAND, 0, 1, Some(c.into())));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_CONFIRM);
    let mut g = duel.select_matching_card(tp, filter, tp, LOCATION_HAND, 0, 1, 1, Some(c.into()))?;
    g.insert(c);
    duel.confirm_cards(1 - tp, g)?;
    duel.shuffle_hand(tp)?;
    duel.set_target_card(g);
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(g), 1, 0, 0);
    duel.set_operation_info(0, CATEGORY_HANDES, Some(g), 1, 0, 0);
    Ok(false)
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8, g: CardSet) -> bool {
    c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
        && g.is_exists(duel, |c, duel| c.is_discardable_full(duel, REASON_EFFECT, None), 1, Some(c.into()))
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if duel.get_location_count(tp, LOCATION_MZONE) == 0 {
        return Ok(());
    }
    let g = duel.get_target_cards(e).unwrap_or_default();
    if g.len() != 2 {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let sg = g.filter_select(duel, tp, |c, duel| spfilter(c, duel, e, tp, g), 1, 1, None)?;
    if sg.len() == 1 {
        duel.special_summon(sg, 0, tp, tp, false, false, POS_FACEUP)?;
        duel.send_to_grave(g.difference(sg), REASON_EFFECT | REASON_DISCARD)?;
    }
    Ok(())
}

fn discon(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    c.is_location(duel, LOCATION_GRAVE)
        && a.r == REASON_SYNCHRO
        && c.get_reason_card(duel).is_some_and(|rc| rc.is_attribute(duel, ATTRIBUTE_WATER))
}

fn disfilter(c: CardId, duel: &Duel) -> bool {
    c.is_negatable_monster(duel) && c.is_type(duel, TYPE_EFFECT)
}

fn distg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(disfilter, tp, 0, LOCATION_MZONE, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_DISABLE, None, 1, 1 - tp, LOCATION_MZONE as i32);
    Ok(false)
}

fn disop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_NEGATE);
    let g = duel.select_matching_card(tp, disfilter, tp, 0, LOCATION_MZONE, 1, 1, None)?;
    if let Some(tc) = g.get_first() {
        duel.hint_selection_full(tc, true);
        tc.negate_effects(duel, a.e.get_handler(duel), RESET_PHASE | PHASE_END, false);
    }
    Ok(())
}
