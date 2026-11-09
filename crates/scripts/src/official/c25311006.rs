// Triple Tactics Talent
use crate::prelude::*;
use crate::testing::{HAND, MZONE, OPPONENT};

const ID: u32 = 25311006;

// In the hand with Dark Hole to destroy the opponent's Sangans, whose effects in the Main Phase enable it.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (53129443, HAND),
    (26202165, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    e1.set_condition(duel, |duel, a| duel.get_custom_activity_count(ID, 1 - a.tp, ACTIVITY_CHAIN) > 0);
    e1.set_target(duel, efftg);
    e1.set_operation(duel, effop);
    c.register_effect(duel, e1);
    duel.add_custom_activity_counter(ID, ACTIVITY_CHAIN, CounterFilter::Chain(chainfilter));
}

fn chainfilter(duel: &Duel, re: EffectId, _: u8, _: u8) -> bool {
    !(duel.is_main_phase() && re.is_monster_effect(duel))
}

fn efftg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let b1 = duel.is_player_can_draw(tp, 2);
    let b2 = duel.is_existing_matching_card(CardId::is_controler_can_be_changed, tp, 0, LOCATION_MZONE, 1, None);
    let b3 = duel.get_field_group_count(tp, 0, LOCATION_HAND) > 0;
    if chk == 0 {
        return Ok(b1 || b2 || b3);
    }
    let op = duel.select_effect(
        tp,
        &[(b1, aux::stringid(ID, 1)), (b2, aux::stringid(ID, 2)), (b3, aux::stringid(ID, 3))],
    )?;
    e.set_label(duel, op.map_or(0, |op| op as i64));
    match op {
        Some(1) => {
            e.set_category(duel, CATEGORY_DRAW);
            e.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
            duel.set_target_player(tp);
            duel.set_target_param(2);
            duel.set_operation_info(0, CATEGORY_DRAW, None, 0, tp, 2);
        }
        Some(2) => {
            e.set_category(duel, CATEGORY_CONTROL);
            e.set_property(duel, 0);
            duel.set_operation_info(0, CATEGORY_CONTROL, None, 1, tp, 0);
        }
        Some(3) => {
            e.set_category(duel, CATEGORY_TODECK);
            e.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
            duel.set_target_player(tp);
            duel.set_operation_info(0, CATEGORY_TODECK, None, 0, 1 - tp, LOCATION_HAND as i32);
        }
        _ => {}
    }
    Ok(false)
}

fn effop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    match a.e.get_label(duel) {
        1 => {
            let p: u8 = duel.get_chain_info(0, CHAININFO_TARGET_PLAYER);
            let d: u32 = duel.get_chain_info(0, CHAININFO_TARGET_PARAM);
            duel.draw(p, d, REASON_EFFECT)?;
        }
        2 => {
            duel.hint(HINT_SELECTMSG, tp, HINTMSG_CONTROL);
            let g = duel.select_matching_card(tp, CardId::is_controler_can_be_changed, tp, 0, LOCATION_MZONE, 1, 1, None)?;
            if !g.is_empty() {
                duel.hint_selection(g);
                let reason_player = duel.get_reason_player();
                duel.get_control_full(g, tp, PHASE_END as u16, 1, 0xff, reason_player)?;
            }
        }
        3 => {
            let p: u8 = duel.get_chain_info(0, CHAININFO_TARGET_PLAYER);
            let g = duel.get_field_group(p, 0, LOCATION_HAND);
            if !g.is_empty() {
                duel.confirm_cards(p, g)?;
                duel.hint(HINT_SELECTMSG, p, HINTMSG_TODECK);
                let sg = g.select(duel, p, 1, 1, None)?;
                if !sg.is_empty() {
                    duel.hint_selection(sg);
                    duel.send_to_deck(sg, PLAYER_NONE, SEQ_DECKSHUFFLE, REASON_EFFECT)?;
                }
                duel.shuffle_hand(1 - p)?;
            }
        }
        _ => {}
    }
    Ok(())
}
