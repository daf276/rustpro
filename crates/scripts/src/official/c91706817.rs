// Angel of Blue Tears
use crate::prelude::*;
use crate::testing::{Check, DECK, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 91706817;

// A copy in the hand and one in the Graveyard, which shares its once per turn with the Graveyard effect, so Trickstar Bella Madonna deals
// the damage that triggers it; the opponent's monster to negate and Waboku in the Deck to Set.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, GRAVE),
    (30655537, MZONE | OPPONENT),
    (12607053, DECK),
    (12607053, DECK),
    (41302052, MZONE),
    (298846, GRAVE),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DAMAGE + CATEGORY_DISABLE);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET + EFFECT_FLAG_PLAYER_TARGET);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_count_limit(duel, 1, ID);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SET);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e2.set_property(duel, EFFECT_FLAG_DELAY);
    e2.set_code(duel, EVENT_DAMAGE);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_count_limit(duel, 1, ID);
    e2.set_cost(duel, Cost::self_banish);
    e2.set_condition(duel, |_, a| a.r & REASON_EFFECT != 0);
    e2.set_target(duel, settg);
    e2.set_operation(duel, setop);
    c.register_effect(duel, e2);
}

fn tgfilter(c: CardId, duel: &Duel, ac: CardId) -> bool {
    c.is_negatable_monster(duel) && duel.get_matching_group_count(aux::TRUE, c.get_controler(duel), 0, LOCATION_HAND, Some(ac.into())) > 0
}

fn target(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE) && tgfilter(chkc, duel, c));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(|tc, duel| tgfilter(tc, duel, c), tp, LOCATION_MZONE, LOCATION_MZONE, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_FACEUP);
    let g = duel.select_target(tp, |tc, duel| tgfilter(tc, duel, c), tp, LOCATION_MZONE, LOCATION_MZONE, 1, 1, None)?;
    let p = g.and_then(|g| g.get_first()).map_or(0, |tc| tc.get_controler(duel));
    duel.set_operation_info(0, CATEGORY_DISABLE, g, 1, 0, 0);
    let dam = duel.get_field_group_count(1 - p, LOCATION_HAND, 0) * 200;
    duel.set_operation_info(0, CATEGORY_DAMAGE, None, 0, 1 - p, dam as i32);
    Ok(false)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let e = a.e;
    if let Some(tc) = duel.get_first_target()
        && tc.is_relate_to_effect(duel, e)
    {
        let p = tc.get_controler(duel);
        let dam = duel.get_field_group_count(1 - p, LOCATION_HAND, 0) * 200;
        if duel.damage(1 - p, dam as u32, REASON_EFFECT)? > 0 && tc.is_faceup(duel) && !tc.is_disabled(duel) {
            duel.break_effect()?;
            duel.negate_related_chain(tc, RESET_TURN_SET);
            let c = e.get_handler(duel);
            let e1 = duel.create_effect(c);
            e1.set_type(duel, EFFECT_TYPE_SINGLE);
            e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
            e1.set_code(duel, EFFECT_DISABLE);
            e1.set_reset(duel, RESETS_STANDARD_PHASE_END);
            tc.register_effect(duel, e1);
            let e2 = duel.create_effect(c);
            e2.set_type(duel, EFFECT_TYPE_SINGLE);
            e2.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
            e2.set_code(duel, EFFECT_DISABLE_EFFECT);
            e2.set_value(duel, RESET_TURN_SET as i32);
            e2.set_reset(duel, RESETS_STANDARD_PHASE_END);
            tc.register_effect(duel, e2);
        }
    }
    Ok(())
}

fn setfilter(c: CardId, duel: &Duel) -> bool {
    c.is_normal_trap(duel) && c.is_ssetable(duel)
}

fn settg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    Ok(chk != 0 || duel.is_existing_matching_card(setfilter, a.tp, LOCATION_HAND | LOCATION_DECK, 0, 1, None))
}

fn setop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    if duel.get_location_count(tp, LOCATION_SZONE) < 1 {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SET);
    let g = duel.select_matching_card(tp, setfilter, tp, LOCATION_HAND | LOCATION_DECK, 0, 1, 1, None)?;
    let Some(tc) = g.get_first() else { return Ok(()) };
    let hand_chk = tc.is_location(duel, LOCATION_HAND);
    if duel.sset(tp, g)? > 0 && hand_chk {
        let e1 = duel.create_effect(a.e.get_handler(duel));
        e1.set_description(duel, aux::stringid(ID, 3));
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_TRAP_ACT_IN_SET_TURN);
        e1.set_property(duel, EFFECT_FLAG_SET_AVAILABLE);
        e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
        tc.register_effect(duel, e1);
    }
    Ok(())
}
