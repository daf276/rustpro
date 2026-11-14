// D/D Savant Kepler
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND, MZONE, PZONE};

const ID: u32 = 11609969;

pub const LISTED_SERIES: &[u16] = &[SET_DD, SET_DARK_CONTRACT];
// In a Pendulum Zone and the hand, with D/D Orthros on the field to return and Dark Contract with the Witch in the Deck to
// search.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, PZONE),
    (ID, HAND),
    (ID, HAND),
    (72181263, MZONE),
    (9765723, DECK),
    (9765723, DECK),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Pendulum::add_procedure(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(
        duel,
        EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_CANNOT_NEGATE,
    );
    e1.set_code(duel, EFFECT_CANNOT_SPECIAL_SUMMON);
    e1.set_range(duel, LOCATION_PZONE);
    e1.set_target_range(duel, 1, 0);
    e1.set_target_spsummon(duel, |duel, _, c, _, sumtp, _, _, _| {
        !c.is_set_card(duel, &[SET_DD]) && sumtp & SUMMON_TYPE_PENDULUM == SUMMON_TYPE_PENDULUM
    });
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_DESTROY);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_F);
    e2.set_code(duel, EVENT_PHASE | PHASE_STANDBY);
    e2.set_range(duel, LOCATION_PZONE);
    e2.set_count_limit(duel, 1, 0);
    e2.set_condition(duel, |duel, a| duel.is_turn_player(a.tp));
    e2.set_target(duel, sctg);
    e2.set_operation(duel, scop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e3.set_property(duel, EFFECT_FLAG_DELAY);
    e3.set_code(duel, EVENT_SUMMON_SUCCESS);
    e3.set_count_limit(duel, 1, ID);
    e3.set_target(duel, thtg);
    e3.set_operation(duel, thop);
    c.register_effect(duel, e3);
    let e4 = e3.clone_effect(duel);
    e4.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    c.register_effect(duel, e4);
}

fn desfilter(c: CardId, duel: &Duel, lvl: u32) -> bool {
    c.is_faceup(duel) && !c.is_set_card(duel, &[SET_DD]) && c.is_level_above(duel, lvl)
}

fn sctg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(true);
    }
    let scale = a.e.get_handler(duel).get_left_scale(duel);
    let lvl = scale.saturating_sub(2).max(1);
    let g = duel.get_matching_group(|c, duel| desfilter(c, duel, lvl), tp, LOCATION_MZONE, 0, None);
    if scale > 1 {
        duel.set_operation_info(0, CATEGORY_DESTROY, Some(g), g.len() as u8, tp, 0);
    }
    Ok(false)
}

fn scop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if c.get_left_scale(duel) <= 1 {
        return Ok(());
    }
    if c.update_scale(duel, -2, None, None) != 0 {
        let scale = c.get_left_scale(duel);
        let g = duel.get_matching_group(|c, duel| desfilter(c, duel, scale), tp, LOCATION_MZONE, 0, None);
        if !g.is_empty() {
            duel.break_effect()?;
            duel.destroy(g, REASON_EFFECT)?;
        }
    }
    Ok(())
}

fn rthfilter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_set_card(duel, &[SET_DD]) && c.is_able_to_hand(duel)
}

fn athfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_DARK_CONTRACT]) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_on_field(duel) && rthfilter(chkc, duel) && chkc != c);
    }
    let b1 = duel.is_existing_target(rthfilter, tp, LOCATION_ONFIELD, 0, 1, Some(c.into()));
    let b2 = duel.is_existing_matching_card(athfilter, tp, LOCATION_DECK, 0, 1, None);
    if chk == 0 {
        return Ok(b1 || b2);
    }
    let op = duel.select_effect(tp, &[(b1, aux::stringid(ID, 2)), (b2, aux::stringid(ID, 3))])?;
    e.set_label(duel, op.map_or(0, |op| op as i64));
    if op == Some(1) {
        e.set_category(duel, CATEGORY_TOHAND);
        e.set_property(duel, EFFECT_FLAG_CARD_TARGET + EFFECT_FLAG_DELAY);
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_RTOHAND);
        let g = duel.select_target(tp, rthfilter, tp, LOCATION_ONFIELD, 0, 1, 1, Some(c.into()))?;
        duel.set_operation_info(0, CATEGORY_TOHAND, g, 1, 0, 0);
    } else {
        e.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
        e.set_property(duel, EFFECT_FLAG_DELAY);
        duel.set_operation_info(0, CATEGORY_TOHAND, None, 1, tp, LOCATION_DECK as i32);
    }
    Ok(false)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if e.get_label(duel) == 1 {
        if let Some(tc) = duel.get_first_target()
            && tc.is_relate_to_effect(duel, e)
        {
            duel.send_to_hand(tc, PLAYER_NONE, REASON_EFFECT)?;
            duel.confirm_cards(1 - tp, tc)?;
        }
    } else {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
        let g = duel.select_matching_card(tp, athfilter, tp, LOCATION_DECK, 0, 1, 1, None)?;
        if !g.is_empty() {
            duel.send_to_hand(g, PLAYER_NONE, REASON_EFFECT)?;
            duel.confirm_cards(1 - tp, g)?;
        }
    }
    Ok(())
}
