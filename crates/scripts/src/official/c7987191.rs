// Borreload Riot Dragon
use crate::prelude::*;
use crate::testing::{Check, GRAVE, MZONE};

const ID: u32 = 7987191;

pub const LISTED_NAMES: &[u32] = &[20071842];

// On the field with Rokket monsters to destroy and in the Graveyard with Rokket monsters to add.
pub const SUPPORT: &[(u32, u16)] = &[(ID, MZONE), (ID, GRAVE), (25554552, MZONE), (25554552, GRAVE), (68464358, GRAVE)];

pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DISABLE_SUMMON + CATEGORY_DESTROY);
    e1.set_type(duel, EFFECT_TYPE_QUICK_O);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_code(duel, EVENT_SPSUMMON);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, |duel, a| a.tp != a.ep && duel.get_current_chain() == 0);
    e1.set_target(duel, target);
    e1.set_operation(duel, operation);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_DESTROY + CATEGORY_TOHAND);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_target(duel, thtg);
    e2.set_operation(duel, thop);
    c.register_effect(duel, e2);
}

fn rokket_faceup(duel: &Duel, tp: u8) -> CardSet {
    duel.get_matching_group(
        aux::faceup_filter(|c, duel| c.is_set_card(duel, &[SET_ROKKET])),
        tp,
        LOCATION_MZONE,
        0,
        None,
    )
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let mut g = rokket_faceup(duel, a.tp);
    g.insert(a.e.get_handler(duel));
    if chk == 0 {
        return Ok(!g.is_empty());
    }
    let eg = a.eg.unwrap_or_default();
    g.merge(eg);
    duel.set_operation_info(0, CATEGORY_DISABLE_SUMMON, Some(eg), eg.len() as u8, 0, 0);
    duel.set_operation_info(0, CATEGORY_DESTROY, Some(g), g.len() as u8, 0, 0);
    Ok(false)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp, c) = (a.e, a.tp, a.e.get_handler(duel));
    let eg = a.eg.unwrap_or_default();
    duel.negate_summon(eg);
    duel.destroy(eg, REASON_EFFECT)?;
    let mut g = rokket_faceup(duel, tp);
    if c.is_relate_to_effect(duel, e) {
        g.insert(c);
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
    let sg = g.select(duel, tp, 1, 1, None)?;
    if !sg.is_empty() {
        duel.break_effect()?;
        duel.hint_selection(sg);
        duel.destroy(sg, REASON_EFFECT)?;
    }
    Ok(())
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    (c.is_set_card(duel, &[SET_ROKKET]) || c.is_set_card(duel, &[SET_BORREL])) && c.is_monster(duel) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_GRAVE) && chkc.is_controler(duel, tp) && thfilter(chkc, duel));
    }
    if chk == 0 {
        return Ok(
            duel.is_existing_matching_card(CardId::is_destructable, tp, LOCATION_HAND | LOCATION_ONFIELD, 0, 1, None)
                && duel.is_existing_target(thfilter, tp, LOCATION_GRAVE, 0, 1, None),
        );
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_RTOHAND);
    let g = duel.select_target(tp, thfilter, tp, LOCATION_GRAVE, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_DESTROY, None, 1, tp, (LOCATION_HAND | LOCATION_ONFIELD) as i32);
    duel.set_operation_info(0, CATEGORY_TOHAND, g, 1, tp, LOCATION_GRAVE as i32);
    Ok(false)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let tc = duel.get_first_target();
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
    let g = duel.select_matching_card(tp, CardId::is_destructable, tp, LOCATION_HAND | LOCATION_ONFIELD, 0, 1, 1, None)?;
    if !g.is_empty()
        && duel.destroy(g, REASON_EFFECT)? > 0
        && let Some(tc) = tc
        && tc.is_relate_to_effect(duel, e)
    {
        duel.send_to_hand(tc, PLAYER_NONE, REASON_EFFECT)?;
        duel.confirm_cards(1 - tp, tc)?;
    }
    Ok(())
}
