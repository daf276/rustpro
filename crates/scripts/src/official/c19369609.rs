// Lyrilusc - Promenade Thrush
use super::c72971064::Procedure;
use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE, OPPONENT, SZONE};

const ID: u32 = 19369609;

// In the Extra Deck, Xyz Summoned with Level 1 Normal Monsters (with another monster of mine to battle), and the opponent's Spells/Traps to
// shuffle and monsters to battle.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (487395, MZONE),
    (1929294, MZONE),
    (3557275, MZONE),
    (12057781, MZONE),
    (21239280, MZONE),
    (26202165, MZONE | OPPONENT),
    (13597785, MZONE | OPPONENT),
    (13597785, MZONE | OPPONENT),
    (18605135, SZONE | OPPONENT),
    (22359980, SZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Xyz::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_UPDATE_ATTACK);
    e1.set_value(
        duel,
        Value::Card(|duel, e, _| e.get_handler(duel).get_overlay_count(duel) as i32 * 500),
    );
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_TODECK);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, 0);
    e2.set_cost(duel, Cost::detach_from_self::<1, 1>);
    e2.set_target(duel, tdtg);
    e2.set_operation(duel, tdop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_ATKCHANGE);
    e3.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e3.set_code(duel, EVENT_BATTLE_START);
    e3.set_range(duel, LOCATION_MZONE);
    e3.set_condition(duel, atkcon);
    e3.set_cost(duel, detachcost);
    e3.set_operation(duel, atkop);
    c.register_effect(duel, e3);
}

fn tdfilter(c: CardId, duel: &Duel) -> bool {
    c.is_spell_trap(duel) && c.is_able_to_deck(duel)
}

fn tdtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_on_field(duel) && chkc.is_controler(duel, 1 - tp) && tdfilter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(tdfilter, tp, 0, LOCATION_ONFIELD, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TODECK);
    let g = duel.select_target(tp, tdfilter, tp, 0, LOCATION_ONFIELD, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_TODECK, g, 1, tp, 0);
    Ok(false)
}

fn tdop(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(tc) = duel.get_first_target()
        && tc.is_relate_to_effect(duel, a.e)
    {
        duel.send_to_deck(tc, PLAYER_NONE, SEQ_DECKSHUFFLE, REASON_EFFECT)?;
    }
    Ok(())
}

fn atkcon(duel: &Duel, a: &Args) -> bool {
    let (bc, _) = duel.get_battle_monster(a.tp);
    bc.is_some_and(|bc| bc.is_faceup(duel) && bc != a.e.get_handler(duel))
}

fn detachcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let max = a.e.get_handler(duel).get_overlay_count(duel) as u16;
    Cost::detach_from_self_with(duel, a, chk, 1, max, |duel, e, og| e.set_label(duel, og.len() as i64))
}

fn atkop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let (bc, _) = duel.get_battle_monster(tp);
    if let Some(bc) = bc
        && bc.is_faceup(duel)
        && bc.is_controler(duel, tp)
        && bc.is_relate_to_battle(duel)
    {
        let amt = a.e.get_label(duel) as i32 * 300;
        bc.update_attack(duel, amt, Some(RESETS_STANDARD_PHASE_END), Some(a.e.get_handler(duel)), None);
    }
    Ok(())
}
