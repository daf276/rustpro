// Piri Reis Map
use crate::prelude::*;
use crate::testing::{DECK, HAND};

const ID: u32 = 33907039;

// In the hand, with Ojama Green (0 ATK) in the Deck to search and Normal Summon.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (12482652, DECK), (12482652, DECK), (12482652, DECK)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOHAND + CATEGORY_SEARCH);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_condition(duel, |duel, _| duel.is_phase(PHASE_MAIN1) && !duel.check_phase_activity());
    e1.set_target(duel, target);
    e1.set_operation(duel, activate);
    c.register_effect(duel, e1);
}

fn thfilter(c: CardId, duel: &Duel) -> bool {
    c.is_monster(duel) && c.is_attack(duel, &[0]) && c.is_able_to_hand(duel)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(thfilter, tp, LOCATION_DECK, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_TOHAND, None, 1, tp, LOCATION_DECK as i32);
    Ok(false)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_ATOHAND);
    let g = duel.select_matching_card(tp, thfilter, tp, LOCATION_DECK, 0, 1, 1, None)?;
    let Some(tc) = g.get_first() else { return Ok(()) };
    duel.send_to_hand(g, PLAYER_NONE, REASON_EFFECT)?;
    duel.confirm_cards(1 - tp, g)?;
    if !tc.is_location(duel, LOCATION_HAND) {
        return Ok(());
    }
    let lp = duel.get_lp(tp);
    duel.set_lp(tp, (lp + 1) / 2);
    let c = a.e.get_handler(duel);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e1.set_code(duel, EVENT_SUMMON_SUCCESS);
    e1.set_label(duel, i64::from(tc.get_code(duel)));
    e1.set_operation(duel, checkop);
    e1.set_reset_full(duel, RESET_PHASE | PHASE_END, 2);
    duel.register_effect(e1, tp);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD);
    e2.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e2.set_code(duel, EFFECT_CANNOT_ACTIVATE);
    e2.set_target_range(duel, 1, 0);
    e2.set_value(duel, Value::EffectPlayer(aclimit));
    e2.set_label_object(duel, Some(e1));
    e2.set_reset_full(duel, RESET_PHASE | PHASE_END, 2);
    duel.register_effect(e2, tp);
    Ok(())
}

fn checkop(duel: &mut Duel, a: &Args) -> Script<()> {
    let Some(sc) = a.eg.and_then(|eg| eg.get_first()) else {
        return Ok(());
    };
    if !sc.is_summon_player(duel, 1 - a.tp) && u32::try_from(a.e.get_label(duel)).is_ok_and(|code| sc.is_code(duel, code)) {
        a.e.set_label(duel, -1);
    }
    Ok(())
}

fn aclimit(duel: &Duel, e: EffectId, re: EffectId, _: u8) -> bool {
    let label = e.get_label_object(duel).map_or(0, |e1| e1.get_label(duel));
    u32::try_from(label).is_ok_and(|code| re.get_handler(duel).is_code(duel, code))
}
