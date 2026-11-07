// Crossout Designator
use engine::constants::MSG_ANNOUNCE_CARD;

use crate::prelude::*;
use crate::testing::Check;

const ID: u32 = 65681983;

pub const CHECKS: &[Check] = &[Check::Resolves, Check::Message(MSG_ANNOUNCE_CARD)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_category(duel, CATEGORY_REMOVE);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_count_limit_full(duel, 1, ID, EFFECT_COUNT_CODE_OATH);
    e1.set_target(duel, rmtg);
    e1.set_operation(duel, rmop);
    c.register_effect(duel, e1);
}

fn rmtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(CardId::is_able_to_remove, tp, LOCATION_DECK, 0, 1, None));
    }
    let g = duel.get_matching_group(CardId::is_able_to_remove, tp, LOCATION_DECK, 0, None);
    let ids = lua::pairs(g.iter().map(|tc| tc.get_code(duel)));
    let mut announce_filter = Vec::new();
    for code in ids {
        announce_filter.extend([u64::from(code), OPCODE_ISCODE]);
        if announce_filter.len() > 2 {
            announce_filter.push(OPCODE_OR);
        }
    }
    let ac = duel.announce_card(tp, &announce_filter)?;
    duel.set_target_param(ac);
    duel.set_operation_info(0, CATEGORY_REMOVE, None, 1, tp, LOCATION_DECK as i32);
    duel.set_operation_info(0, CATEGORY_ANNOUNCE, None, 0, tp, ANNOUNCE_CARD_FILTER as i32);
    Ok(false)
}

fn rmop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let ac: u32 = duel.get_chain_info(0, CHAININFO_TARGET_PARAM);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_REMOVE);
    let g = duel.select_matching_card(
        tp,
        |c, duel| c.is_code(duel, ac) && c.is_able_to_remove(duel),
        tp,
        LOCATION_DECK,
        0,
        1,
        1,
        None,
    )?;
    let Some(tc) = g.get_first() else { return Ok(()) };
    if duel.remove(tc, POS_FACEUP, REASON_EFFECT)? == 0 || !tc.is_location(duel, LOCATION_REMOVED) {
        return Ok(());
    }
    let c = a.e.get_handler(duel);
    let code = i64::from(tc.get_original_code_rule(duel));
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_DISABLE);
    e1.set_target_range(duel, LOCATION_ONFIELD, LOCATION_ONFIELD);
    e1.set_label(duel, code);
    e1.set_target_card(duel, distg);
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e1, tp);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e2.set_code(duel, EVENT_CHAIN_SOLVING);
    e2.set_label(duel, code);
    e2.set_condition(duel, discon);
    e2.set_operation(duel, disop);
    e2.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e2, tp);
    let e3 = e1.clone_effect(duel);
    e3.set_code(duel, EFFECT_DISABLE_TRAPMONSTER);
    e3.set_target_range(duel, LOCATION_MZONE, LOCATION_MZONE);
    duel.register_effect(e3, tp);
    Ok(())
}

fn distg(duel: &Duel, e: EffectId, c: CardId) -> bool {
    c.is_original_code_rule(duel, e.get_label(duel) as u32)
}

fn discon(duel: &Duel, a: &Args) -> bool {
    a.re.is_some_and(|re| re.get_handler(duel).is_original_code_rule(duel, a.e.get_label(duel) as u32))
}

fn disop(duel: &mut Duel, a: &Args) -> Script<()> {
    duel.negate_effect(a.ev as u8);
    Ok(())
}
