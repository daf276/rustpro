// Krawler Spine
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND, MZONE, OPPONENT};

const ID: u32 = 88316955;

// On the field face-down, flipped by Elfnote Lucina's attack, and destroyed by Raigeki, with Krawler Axon and Dendrite (two other
// names) in the Deck to summon.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, MZONE),
    (ID, MZONE),
    (ID, MZONE),
    (15710054, DECK),
    (15710054, DECK),
    (46083111, DECK),
    (46083111, DECK),
    (13597785, MZONE | OPPONENT),
    (12580477, HAND | OPPONENT),
    (12580477, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DESTROY);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_FLIP + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET + EFFECT_FLAG_DELAY);
    e1.set_count_limit(duel, 1, ID);
    e1.set_target(duel, target);
    e1.set_operation(duel, stock::destroy_first_target);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_SET);
    e2.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DELAY + EFFECT_FLAG_DAMAGE_STEP);
    e2.set_code(duel, EVENT_LEAVE_FIELD);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, spcon);
    e2.set_target(duel, sptg::<ID>);
    e2.set_operation(duel, spop::<ID>);
    c.register_effect(duel, e2);
}

fn target(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(|_, _| true, tp, LOCATION_MZONE, LOCATION_MZONE, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
    let g = duel.select_target(tp, |_, _| true, tp, LOCATION_MZONE, LOCATION_MZONE, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_DESTROY, g, 1, 0, 0);
    Ok(false)
}

pub fn spcon(duel: &Duel, a: &Args) -> bool {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    c.is_previous_position(duel, POS_FACEUP)
        && c.get_location(duel) != LOCATION_DECK
        && c.is_reason(duel, REASON_EFFECT)
        && c.get_reason_player(duel) == 1 - tp
        && c.is_previous_controler(duel, tp)
}

fn filter1<const CODE: u32>(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_KRAWLER]) && !c.is_code(duel, CODE) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

pub fn sptg<const CODE: u32>(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        if duel.is_player_affected_by_effect(tp, CARD_BLUEEYES_SPIRIT).is_some() || duel.get_location_count(tp, LOCATION_MZONE) < 2 {
            return Ok(false);
        }
        let g = duel.get_matching_group(|c, duel| filter1::<CODE>(c, duel, e, tp), tp, LOCATION_DECK, 0, None);
        return Ok(g.get_class_count(duel, |c, duel| i64::from(c.get_code(duel))) >= 2);
    }
    e.get_handler(duel).create_effect_relation(duel, e);
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 2, tp, LOCATION_DECK as i32);
    Ok(false)
}

pub fn spop<const CODE: u32>(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if duel.is_player_affected_by_effect(tp, CARD_BLUEEYES_SPIRIT).is_some() || duel.get_location_count(tp, LOCATION_MZONE) < 2 {
        return Ok(());
    }
    let g = duel.get_matching_group(|c, duel| filter1::<CODE>(c, duel, e, tp), tp, LOCATION_DECK, 0, None);
    let sg = aux::select_unselect_group(duel, g, 2, 2, &aux::dncheck, tp, HINTMSG_SPSUMMON)?;
    if !sg.is_empty() {
        duel.special_summon(sg, 0, tp, tp, false, false, POS_FACEDOWN_DEFENSE)?;
        duel.confirm_cards(1 - tp, sg)?;
    }
    Ok(())
}
