// Blackwing - Silverwind the Ascendant
use super::c76913983::Procedure;
use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE, OPPONENT};

const ID: u32 = 33236860;

pub const LISTED_SERIES: &[u16] = &[SET_BLACKWING];
// In the Extra Deck, Synchro Summoned from Blizzard (a Level 2 Tuner), Zephyros (Level 4) and Souls of the Forgotten (Level 2),
// with low-DEF monsters of the opponent's to destroy.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (22835145, MZONE),
    (22835145, MZONE),
    (14785765, MZONE),
    (14785765, MZONE),
    (4920010, MZONE),
    (4920010, MZONE),
    (1929294, MZONE | OPPONENT),
    (1929294, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Synchro::add_procedure::<Procedure<2>>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DESTROY);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_condition(duel, descon);
    e1.set_cost(duel, descost);
    e1.set_target(duel, destg);
    e1.set_operation(duel, desop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD);
    e2.set_code(duel, EFFECT_INDESTRUCTABLE_COUNT);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_target_range(duel, LOCATION_MZONE, 0);
    e2.set_count_limit(duel, 1, 0);
    e2.set_condition(duel, |duel, a| duel.is_turn_player(1 - a.e.get_handler_player(duel)));
    e2.set_target_card(duel, |duel, _, c| c.is_set_card(duel, &[SET_BLACKWING]));
    e2.set_value(duel, Value::Reason(|_, _, _, r, _| r & REASON_BATTLE > 0));
    c.register_effect(duel, e2);
}

fn descon(duel: &Duel, a: &Args) -> bool {
    a.e.get_handler(duel).is_synchro_summoned(duel)
}

fn descost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.get_activity_count(tp, ACTIVITY_BATTLE_PHASE) == 0);
    }
    let e1 = duel.create_effect(a.e.get_handler(duel));
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_CANNOT_BP);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_OATH);
    e1.set_target_range(duel, 1, 0);
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e1, tp);
    Ok(false)
}

fn filter(c: CardId, duel: &Duel, atk: i32) -> bool {
    c.is_faceup(duel) && c.is_defense_below(duel, atk - 1)
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let atk = a.e.get_handler(duel).get_attack(duel);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE) && filter(chkc, duel, atk));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(|c, duel| filter(c, duel, atk), tp, LOCATION_MZONE, LOCATION_MZONE, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
    let g = duel.select_target(tp, |c, duel| filter(c, duel, atk), tp, LOCATION_MZONE, LOCATION_MZONE, 1, 2, None)?;
    let count = g.map_or(0, |g| g.len());
    duel.set_operation_info(0, CATEGORY_DESTROY, g, count as u8, 0, 0);
    Ok(false)
}

fn desfilter(c: CardId, duel: &Duel, e: EffectId, atk: i32) -> bool {
    c.is_faceup(duel) && c.is_relate_to_effect(duel, e) && c.is_defense_below(duel, atk - 1)
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, c) = (a.e, a.e.get_handler(duel));
    if !c.is_relate_to_effect(duel, e) {
        return Ok(());
    }
    let g: Option<CardSet> = duel.get_chain_info(0, CHAININFO_TARGET_CARDS);
    let atk = c.get_attack(duel);
    let sg = g.unwrap_or_default().filter(duel, |c, duel| desfilter(c, duel, e, atk), None);
    duel.destroy(sg, REASON_EFFECT)?;
    Ok(())
}
