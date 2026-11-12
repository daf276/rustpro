// Gunkan Suship Shirauo
use crate::prelude::*;
use crate::testing::{Check, DECK, GRAVE, HAND, MZONE};

const ID: u32 = 78362751;
pub const LISTED_NAMES: &[u32] = &[ID, CARD_SUSHIP_SHARI];

// One in the hand with Gunkan Suship Shari on the field, one on the field, and Ikura and Uni in the hand to Special Summon, with Shari
// in the Deck and Graveyard to put on top of the Deck.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, MZONE),
    (CARD_SUSHIP_SHARI, MZONE),
    (CARD_SUSHIP_SHARI, DECK),
    (CARD_SUSHIP_SHARI, DECK),
    (CARD_SUSHIP_SHARI, GRAVE),
    (61027400, HAND),
    (42377643, HAND),
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
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, selfspcon);
    e1.set_target(duel, stock::special_summon_self_target_player_0);
    e1.set_operation(duel, stock::special_summon_self::<POS_FACEUP>);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_TODECK);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_target(duel, sptg);
    e2.set_operation(duel, spop);
    c.register_effect(duel, e2);
}

fn selfspconfilter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel)
        && (c.is_code(duel, CARD_SUSHIP_SHARI)
            || c.get_overlay_group(duel)
                .is_exists(duel, |c, duel| c.is_code(duel, CARD_SUSHIP_SHARI), 1, None))
}

fn selfspcon(duel: &Duel, a: &Args) -> bool {
    duel.is_existing_matching_card(selfspconfilter, a.tp, LOCATION_MZONE, 0, 1, None)
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_GUNKAN]) && !c.is_code(duel, ID) && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0
            && duel.is_existing_matching_card(|c, duel| spfilter(c, duel, e, tp), tp, LOCATION_HAND, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_HAND as i32);
    duel.set_possible_operation_info(0, CATEGORY_TODECK, None, 1, tp, LOCATION_GRAVE as i32);
    Ok(false)
}

fn tdfilter(c: CardId, duel: &Duel) -> bool {
    c.is_code(duel, CARD_SUSHIP_SHARI) && (c.is_location(duel, LOCATION_DECK) || c.is_able_to_deck(duel))
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if duel.get_location_count(tp, LOCATION_MZONE) <= 0 {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_matching_card(tp, |c, duel| spfilter(c, duel, e, tp), tp, LOCATION_HAND, 0, 1, 1, None)?;
    if g.is_empty() || duel.special_summon(g, 0, tp, tp, false, false, POS_FACEUP)? == 0 {
        return Ok(());
    }
    let dg = duel.get_matching_group(aux::necro_valley_filter(tdfilter), tp, LOCATION_DECK | LOCATION_GRAVE, 0, None);
    if dg.is_empty() || !duel.select_yes_no(tp, aux::stringid(ID, 2))? {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TODECK);
    let sg = dg.select(duel, tp, 1, dg.len() as u8, None)?;
    if !sg.is_empty() {
        duel.break_effect()?;
        duel.hint_selection(sg);
        duel.send_to_deck(sg, PLAYER_NONE, SEQ_DECKTOP, REASON_EFFECT)?;
        duel.move_to_deck_top(sg)?;
        if sg.len() > 1 {
            duel.sort_decktop(tp, tp, sg.len() as u16)?;
        }
    }
    Ok(())
}
