// Mementotlan Akihiron
use crate::prelude::*;
use crate::testing::{Check, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 54207171;

// In the hand, on the field and in the Graveyard, with Mementotlan Angwitch to add and to be destroyed by the opponent's
// Raigeki and Dark Hole.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, MZONE),
    (ID, GRAVE),
    (54550967, MZONE),
    (54550967, GRAVE),
    (12580477, HAND | OPPONENT),
    (53129443, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::ResolvesEffect(aux::stringid(ID, 2)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_DELAY);
    e1.set_code(duel, EVENT_SUMMON_SUCCESS);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, |duel, a| {
        (duel.is_able_to_enter_bp() || (duel.is_battle_phase() && !duel.is_phase(PHASE_BATTLE))) && !duel.has_flag_effect(a.tp, ID)
    });
    e1.set_operation(duel, diratkop);
    c.register_effect(duel, e1);
    let e2 = e1.clone_effect(duel);
    e2.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_TOHAND);
    e3.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e3.set_property(duel, EFFECT_FLAG_DELAY + EFFECT_FLAG_CARD_TARGET);
    e3.set_code(duel, EVENT_DESTROYED);
    e3.set_count_limit(duel, 1, (ID, 1));
    e3.set_condition(duel, |duel, a| a.e.get_handler(duel).is_reason(duel, REASON_EFFECT));
    e3.set_target(duel, thtg);
    e3.set_operation(duel, thop);
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_description(duel, aux::stringid(ID, 2));
    e4.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e4.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e4.set_property_full(duel, EFFECT_FLAG_DAMAGE_STEP + EFFECT_FLAG_DELAY, EFFECT_FLAG2_CHECK_SIMULTANEOUS);
    e4.set_code(duel, EVENT_DESTROYED);
    e4.set_range(duel, LOCATION_GRAVE);
    e4.set_count_limit(duel, 1, (ID, 2));
    e4.set_condition(duel, spcon);
    e4.set_target(duel, sptg);
    e4.set_operation(duel, stock::special_summon_self::<POS_FACEUP>);
    c.register_effect(duel, e4);
}

fn diratkop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    if duel.has_flag_effect(tp, ID) {
        return Ok(());
    }
    duel.register_flag_effect(tp, ID, RESET_PHASE | PHASE_END, 0, 1);
    let e1 = duel.create_effect(a.e.get_handler(duel));
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_DIRECT_ATTACK);
    e1.set_target_range(duel, LOCATION_MZONE, LOCATION_MZONE);
    e1.set_target_card(duel, |duel, _, c| c.is_set_card(duel, &[SET_MEMENTO]) && c.is_level_above(duel, 9));
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e1, tp);
    Ok(())
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_MEMENTO]) && c.is_faceup(duel) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_GRAVE | LOCATION_REMOVED)
            && chkc.is_controler(duel, tp)
            && thfilter(chkc, duel)
            && chkc != c);
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(thfilter, tp, LOCATION_GRAVE | LOCATION_REMOVED, 0, 1, Some(c.into())));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let g = duel.select_target(tp, thfilter, tp, LOCATION_GRAVE | LOCATION_REMOVED, 0, 1, 1, Some(c.into()))?;
    duel.set_operation_info(0, CATEGORY_TOHAND, g, 1, tp, 0);
    Ok(false)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(tc) = duel.get_first_target()
        && tc.is_relate_to_effect(duel, a.e)
    {
        duel.send_to_hand(tc, PLAYER_NONE, REASON_EFFECT)?;
        duel.confirm_cards(1 - a.tp, tc)?;
    }
    Ok(())
}

fn spconfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    if !(c.is_reason(duel, REASON_BATTLE | REASON_EFFECT)
        && c.is_previous_controler(duel, tp)
        && c.is_previous_location(duel, LOCATION_HAND | LOCATION_MZONE))
    {
        return false;
    }
    if c.is_previous_location(duel, LOCATION_MZONE) {
        c.is_previous_set_card(duel, &[SET_MEMENTO]) && c.is_previous_position(duel, POS_FACEUP)
    } else {
        c.is_set_card(duel, &[SET_MEMENTO]) && c.is_monster(duel)
    }
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    let eg = a.eg.unwrap_or_default();
    !eg.is_contains(a.e.get_handler(duel)) && eg.is_exists(duel, |c, duel| spconfilter(c, duel, a.tp), 1, None)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, c, tp) = (a.e, a.e.get_handler(duel), a.tp);
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0 && c.is_can_be_special_summoned(duel, e, 0, tp, false, false));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(c.into()), 1, tp, 0);
    Ok(false)
}
