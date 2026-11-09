// Songs of the Dominators
use crate::prelude::*;
use crate::testing::{Check, DECK, GRAVE, HAND, MZONE, OPPONENT, SZONE};

const ID: u32 = 58053438;

// Set or in the hand, to negate the effect of Snipe Hunter on the opponent's field, with a Trap in the Graveyard and Dominus Impulse
// in the Deck to add.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, SZONE),
    (ID, HAND),
    (84290642, MZONE | OPPONENT),
    (84290642, HAND | OPPONENT),
    (97045737, GRAVE),
    (40366667, DECK),
    (40366667, DECK),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DISABLE + CATEGORY_TOHAND + CATEGORY_SEARCH);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_CHAINING);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    e1.set_condition(duel, condition);
    e1.set_target(duel, target);
    e1.set_operation(duel, operation);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_code(duel, EFFECT_TRAP_ACT_IN_HAND);
    e2.set_condition(duel, |duel, a| {
        !duel.is_existing_matching_card(CardId::is_monster, a.e.get_handler_player(duel), LOCATION_GRAVE, 0, 1, None)
    });
    c.register_effect(duel, e2);
}

fn condition(duel: &Duel, a: &Args) -> bool {
    let ev = a.ev as u8;
    let trig_p: u8 = duel.get_chain_info(ev, CHAININFO_TRIGGERING_PLAYER);
    let trig_typ: u32 = duel.get_chain_info(ev, CHAININFO_TRIGGERING_TYPE);
    let trig_loc: u32 = duel.get_chain_info(ev, CHAININFO_TRIGGERING_LOCATION);
    trig_p == 1 - a.tp && trig_typ & TYPE_MONSTER > 0 && trig_loc & LOCATION_MZONE > 0
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    let (e, tp) = (a.e, a.tp);
    let act_from_hand_chk = e.is_has_type(duel, EFFECT_TYPE_ACTIVATE) && e.get_handler(duel).is_status(duel, STATUS_ACT_FROM_HAND);
    e.set_label(duel, i64::from(act_from_hand_chk));
    duel.set_operation_info(0, CATEGORY_DISABLE, a.eg, 1, tp, 0);
    duel.set_possible_operation_info(0, CATEGORY_TOHAND, None, 1, tp, LOCATION_DECK as i32);
    Ok(false)
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_DOMINUS]) && c.is_able_to_hand(duel)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if duel.negate_effect(a.ev as u8)
        && duel.is_existing_matching_card(CardId::is_trap, tp, LOCATION_GRAVE, 0, 1, None)
        && duel.is_existing_matching_card(thfilter, tp, LOCATION_DECK, 0, 1, None)
        && duel.select_yes_no(tp, aux::stringid(ID, 2))?
    {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
        let g = duel.select_matching_card(tp, thfilter, tp, LOCATION_DECK, 0, 1, 1, None)?;
        if !g.is_empty() {
            duel.break_effect()?;
            duel.send_to_hand(g, PLAYER_NONE, REASON_EFFECT)?;
            duel.confirm_cards(1 - tp, g)?;
        }
    }
    if e.is_has_type(duel, EFFECT_TYPE_ACTIVATE) && e.get_label(duel) == 1 {
        let e1 = duel.create_effect(e.get_handler(duel));
        e1.set_description(duel, aux::stringid(ID, 3));
        e1.set_type(duel, EFFECT_TYPE_FIELD);
        e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_CLIENT_HINT);
        e1.set_code(duel, EFFECT_CANNOT_ACTIVATE);
        e1.set_target_range(duel, 1, 0);
        e1.set_value(duel, Value::EffectPlayer(aclimit));
        e1.set_reset_full(duel, RESET_PHASE | PHASE_END, 2);
        duel.register_effect(e1, tp);
    }
    Ok(())
}

fn aclimit(duel: &Duel, _: EffectId, re: EffectId, _: u8) -> bool {
    re.is_monster_effect(duel)
        && re
            .get_handler(duel)
            .is_location(duel, LOCATION_HAND | LOCATION_GRAVE | LOCATION_REMOVED)
}
