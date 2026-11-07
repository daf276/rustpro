// Pot of Prosperity
use crate::prelude::*;

const ID: u32 = 84211599;

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    e1.set_cost(duel, cost);
    e1.set_target(duel, target);
    e1.set_operation(duel, operation);
    c.register_effect(duel, e1);
    aux::global_check(duel, ID, |duel| {
        let ge1 = duel.create_effect(c);
        ge1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
        ge1.set_code(duel, EVENT_DRAW);
        ge1.set_operation(duel, checkop);
        duel.register_effect(ge1, 0);
    });
}

fn checkop(duel: &mut Duel, a: &Args) -> Script<()> {
    if a.r & REASON_EFFECT > 0 {
        duel.register_flag_effect(a.ep, ID, RESET_PHASE | PHASE_END, 0, 1);
    }
    Ok(())
}

fn cost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.get_flag_effect(tp, ID) == 0);
    }
    let e1 = duel.create_effect(a.e.get_handler(duel));
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_CANNOT_DRAW);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_CLIENT_HINT + EFFECT_FLAG_OATH);
    e1.set_description(duel, aux::stringid(ID, 2));
    e1.set_target_range(duel, 1, 0);
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e1, tp);
    Ok(false)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let g = duel.get_matching_group(
        |c, duel| c.is_able_to_remove_as_cost_full(duel, POS_FACEDOWN),
        tp,
        LOCATION_EXTRA,
        0,
        None,
    );
    let dg = duel.get_field_group_count(tp, LOCATION_DECK, 0);
    let top_to_hand = |duel: &Duel, count| {
        g.len() >= count && dg >= count && duel.get_decktop_group(tp, count).filter_count(duel, CardId::is_able_to_hand, None) > 0
    };
    let (t3, t6) = (top_to_hand(duel, 3), top_to_hand(duel, 6));
    if chk == 0 {
        return Ok(t3 || t6);
    }
    let ann: Vec<u64> = [(t3, 3), (t6, 6)]
        .into_iter()
        .filter_map(|(available, count)| available.then_some(count))
        .collect();
    duel.hint(HINT_SELECTMSG, tp, aux::stringid(ID, 0));
    let d = duel.announce_number(tp, &ann)?;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_REMOVE);
    let rg = g.select(duel, tp, d as u8, d as u8, None)?;
    duel.remove(rg, POS_FACEDOWN, REASON_COST)?;
    a.e.set_label(duel, d as i64);
    duel.set_target_player(tp);
    duel.set_operation_info(0, CATEGORY_TOHAND, None, 1, 0, LOCATION_DECK as i32);
    Ok(false)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let p: u8 = duel.get_chain_info(0, CHAININFO_TARGET_PLAYER);
    let d = e.get_label(duel) as usize;
    duel.confirm_decktop(p, d)?;
    let mut g = duel.get_decktop_group(p, d);
    if !g.is_empty() {
        duel.disable_shuffle_check(true);
        duel.hint(HINT_SELECTMSG, p, HINTMSG_ATOHAND);
        if let Some(sc) = g.select(duel, p, 1, 1, None)?.get_first() {
            if sc.is_able_to_hand(duel) {
                duel.send_to_hand(sc, PLAYER_NONE, REASON_EFFECT)?;
                duel.confirm_cards(1 - p, sc)?;
                duel.shuffle_hand(p)?;
                g.remove(sc);
                duel.move_to_deck_bottom(g)?;
                duel.sort_deckbottom(p, p, g.len() as u16)?;
            } else {
                duel.send_to_grave(sc, REASON_RULE)?;
            }
        }
    }
    let e1 = duel.create_effect(e.get_handler(duel));
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_CHANGE_DAMAGE);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e1.set_target_range(duel, 0, 1);
    e1.set_value(duel, Value::ChangeDamage(|_, _, _, val, _, _, _| val / 2));
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e1, tp);
    aux::register_client_hint(duel, e.get_handler(duel), 0, tp, 1, 0, Some(aux::stringid(ID, 1)));
    Ok(())
}
