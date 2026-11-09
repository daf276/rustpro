// Maliss <C> MTP-07
use crate::prelude::*;
use crate::testing::{Check, DECK, MZONE};

const ID: u32 = 94722358;

// With Maliss <P> March Hare in the Deck to add and on the field to banish for the turn it is Set.
pub const SUPPORT: &[(u32, u16)] = &[(ID, 0), (20938824, DECK), (20938824, DECK), (96676583, DECK), (20938824, MZONE)];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH + CATEGORY_REMOVE);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    e1.set_hint_timing(duel, 0, TIMING_STANDBY_PHASE | TIMING_MAIN_END | TIMINGS_CHECK_MONSTER_E);
    e1.set_cost(duel, thcost);
    e1.set_target(duel, thtg);
    e1.set_operation(duel, thop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_type(duel, EFFECT_TYPE_SINGLE);
    e2.set_property(duel, EFFECT_FLAG_SET_AVAILABLE);
    e2.set_code(duel, EFFECT_TRAP_ACT_IN_SET_TURN);
    e2.set_value(duel, Value::Activation(|duel, e, _| e.set_label(duel, 1)));
    e2.set_condition(duel, |duel, a| {
        let tp = a.e.get_handler_player(duel);
        duel.is_existing_matching_card(|c, duel| thcostfilter(c, duel, tp), tp, LOCATION_MZONE, 0, 1, None)
    });
    c.register_effect(duel, e2);
    e1.set_label_object(duel, Some(e2));
}

fn thcostfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_faceup(duel) && c.is_set_card(duel, &[SET_MALISS]) && c.is_able_to_remove_full(duel, Some(tp), POS_FACEUP, REASON_EFFECT)
}

fn thcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    let Some(label_obj) = a.e.get_label_object(duel) else {
        return Ok(chk == 0);
    };
    if chk == 0 {
        label_obj.set_label(duel, 0);
        return Ok(true);
    }
    if label_obj.get_label(duel) > 0 {
        label_obj.set_label(duel, 0);
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_REMOVE);
        let g = duel.select_matching_card(tp, |c, duel| thcostfilter(c, duel, tp), tp, LOCATION_MZONE, 0, 1, 1, None)?;
        duel.remove(g, POS_FACEUP, REASON_COST)?;
    }
    Ok(false)
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_MALISS]) && c.is_monster(duel) && c.is_able_to_hand(duel)
}

fn thtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(thfilter, tp, LOCATION_DECK, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_TOHAND, None, 1, tp, LOCATION_DECK as i32);
    duel.set_possible_operation_info(0, CATEGORY_REMOVE, None, 1, PLAYER_EITHER, LOCATION_ONFIELD as i32);
    Ok(false)
}

fn linkfilter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_set_card(duel, &[SET_MALISS]) && c.is_type(duel, TYPE_LINK)
}

fn thop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let g = duel.select_matching_card(tp, thfilter, tp, LOCATION_DECK, 0, 1, 1, None)?;
    if g.is_empty()
        || duel.send_to_hand(g, PLAYER_NONE, REASON_EFFECT)? == 0
        || !g.get_first().is_some_and(|tc| tc.is_location(duel, LOCATION_HAND))
    {
        return Ok(());
    }
    duel.confirm_cards(1 - tp, g)?;
    duel.shuffle_hand(tp)?;
    let c = e.get_handler(duel);
    let exc = c.is_relate_to_effect(duel, e).then(|| c.into());
    let rg = duel.get_matching_group(CardId::is_able_to_remove, tp, LOCATION_ONFIELD, LOCATION_ONFIELD, exc);
    if !rg.is_empty()
        && duel.is_existing_matching_card(linkfilter, tp, LOCATION_MZONE, 0, 1, None)
        && duel.select_yes_no(tp, aux::stringid(ID, 2))?
    {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_REMOVE);
        let sg = rg.select(duel, tp, 1, 1, None)?;
        if !sg.is_empty() {
            duel.hint_selection(sg);
            duel.break_effect()?;
            duel.remove(sg, POS_FACEUP, REASON_EFFECT)?;
        }
    }
    Ok(())
}
