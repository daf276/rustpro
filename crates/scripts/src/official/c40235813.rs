// Dark Time Wizard
use crate::prelude::*;
use crate::testing::{Check, DECK, GRAVE, HAND, MZONE, OPPONENT};
use engine::constants::MSG_TOSS_COIN;

const ID: u32 = 40235813;

pub const LISTED_NAMES: &[u32] = &[ID];
pub const TOSS_COIN: bool = true;

// In the hand and Graveyard, with the opponent's Sangan and its own Normal Monsters to destroy, and cards that mention it to search.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (ID, GRAVE),
    (4881365, DECK),
    (4881365, DECK),
    (77482666, DECK),
    (77482666, DECK),
    (26202165, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
    (1784619, MZONE),
    (1784619, MZONE),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0)), Check::Message(MSG_TOSS_COIN)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    e1.set_hint_timing(
        duel,
        TIMING_END_PHASE,
        TIMING_STANDBY_PHASE | TIMING_MAIN_END | TIMINGS_CHECK_MONSTER_E,
    );
    c.register_effect(duel, e1);
}

fn deckthfilter(c: CardId, duel: &Duel) -> bool {
    c.lists_code(duel, ID) && !c.is_code(duel, ID) && c.is_able_to_hand(duel)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let b1 = !duel.has_flag_effect(tp, ID) && duel.is_existing_matching_card(deckthfilter, tp, LOCATION_DECK, 0, 1, None);
    let g = duel.get_field_group(tp, LOCATION_MZONE, LOCATION_MZONE);
    let b2 = !duel.has_flag_effect(tp, ID + 1) && !g.is_empty();
    if chk == 0 {
        return Ok(b1 || b2);
    }
    let op = duel.select_effect(tp, &[(b1, aux::stringid(ID, 1)), (b2, aux::stringid(ID, 2))])?;
    e.set_label(duel, op.map_or(0, |op| op as i64));
    if op == Some(1) {
        duel.register_flag_effect(tp, ID, RESET_PHASE | PHASE_END, 0, 1);
        e.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
        duel.set_operation_info(0, CATEGORY_TOHAND, None, 1, tp, LOCATION_DECK as i32);
        duel.set_possible_operation_info(0, CATEGORY_TOHAND, None, 1, tp, LOCATION_GRAVE as i32);
    } else {
        duel.register_flag_effect(tp, ID + 1, RESET_PHASE | PHASE_END, 0, 1);
        e.set_category(duel, CATEGORY_COIN + CATEGORY_DESTROY + CATEGORY_DAMAGE);
        duel.set_operation_info(0, CATEGORY_COIN, None, 0, tp, 1);
        duel.set_operation_info(0, CATEGORY_DESTROY, Some(g), 1, tp, 0);
        duel.set_possible_operation_info(0, CATEGORY_DAMAGE, None, 1, 1 - tp, 0);
    }
    Ok(false)
}

fn gythfilter(c: CardId, duel: &Duel) -> bool {
    c.is_code(duel, ID) && c.is_able_to_hand(duel)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let op = e.get_label(duel);
    if op == 1 {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
        let g = duel.select_matching_card(tp, deckthfilter, tp, LOCATION_DECK, 0, 1, 1, None)?;
        if !g.is_empty() {
            duel.send_to_hand(g, PLAYER_NONE, REASON_EFFECT)?;
            duel.confirm_cards(1 - tp, g)?;
        }
        let c = e.get_handler(duel);
        aux::register_client_hint(duel, c, 0, tp, 1, 0, Some(aux::stringid(ID, 3)));
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
        e1.set_code(duel, EVENT_PHASE + PHASE_END);
        e1.set_count_limit(duel, 1, 0);
        e1.set_operation(duel, gyop);
        e1.set_reset(duel, RESET_PHASE | PHASE_END);
        duel.register_effect(e1, tp);
    } else if op == 2 {
        if duel.call_coin(tp)? {
            let g = duel.get_field_group(tp, 0, LOCATION_MZONE);
            if !g.is_empty() && duel.destroy(g, REASON_EFFECT)? > 0 {
                let dam = duel
                    .get_operated_group()
                    .get_sum(duel, |c, duel| i64::from(c.get_base_attack(duel)));
                if dam > 0 {
                    duel.damage(1 - tp, (dam as f64 / 2.0).round() as u32, REASON_EFFECT)?;
                }
            }
        } else {
            let g = duel.get_field_group(tp, LOCATION_MZONE, 0);
            if !g.is_empty() {
                duel.destroy(g, REASON_EFFECT)?;
            }
        }
    }
    Ok(())
}

fn gyop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_CARD, 0, ID.into());
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let g = duel.select_matching_card(tp, aux::necro_valley_filter(gythfilter), tp, LOCATION_GRAVE, 0, 1, 1, None)?;
    if !g.is_empty() {
        duel.hint_selection(g);
        duel.send_to_hand(g, PLAYER_NONE, REASON_EFFECT)?;
    }
    Ok(())
}
