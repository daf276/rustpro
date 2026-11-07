// Fairy Tale Tails
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE, SZONE};

const ID: u32 = 91957038;

// Fairy Tail - Luna in the hand to Normal Summon and on the field to qualify the damage prevention, and a second copy
// of this card on the field to use its effects.
pub const SUPPORT: &[(u32, u16)] = &[(86937530, HAND), (86937530, MZONE), (ID, SZONE)];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_SUMMON + CATEGORY_SET);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_SZONE);
    e2.set_count_limit(duel, 1, ID);
    e2.set_cost(duel, nscost);
    e2.set_target(duel, nstg);
    e2.set_operation(duel, nsop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_FIELD);
    e3.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e3.set_code(duel, EFFECT_CHANGE_DAMAGE);
    e3.set_range(duel, LOCATION_SZONE);
    e3.set_target_range(duel, 1, 0);
    e3.set_condition(duel, damcon);
    e3.set_value(duel, Value::ChangeDamage(damval));
    c.register_effect(duel, e3);
    let e4 = e3.clone_effect(duel);
    e4.set_code(duel, EFFECT_NO_EFFECT_DAMAGE);
    c.register_effect(duel, e4);
}

fn nsfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_attack(duel, &[1850])
        && c.is_race(duel, RACE_SPELLCASTER)
        && c.can_summon_or_set(duel, true, None, 0)
        && !c.is_public(duel)
        && !duel.is_existing_matching_card(
            aux::faceup_filter(|c2, duel| c2.is_code(duel, c.get_code(duel))),
            tp,
            LOCATION_MZONE,
            0,
            1,
            None,
        )
}

fn nscost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(|c, duel| nsfilter(c, duel, tp), tp, LOCATION_HAND, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_CONFIRM);
    let g = duel.select_matching_card(tp, |c, duel| nsfilter(c, duel, tp), tp, LOCATION_HAND, 0, 1, 1, None)?;
    duel.confirm_cards(1 - tp, g)?;
    duel.shuffle_hand(tp)?;
    duel.set_target_card(g);
    Ok(false)
}

fn nstg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    duel.set_operation_info(0, CATEGORY_SUMMON, None, 1, a.tp, LOCATION_HAND as i32);
    Ok(false)
}

fn nsop(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(tc) = duel.get_first_target()
        && tc.is_relate_to_effect(duel, a.e)
    {
        duel.summon_or_set(a.tp, tc, true, None, 0)?;
    }
    Ok(())
}

fn damfilter(c: CardId, duel: &Duel) -> bool {
    c.is_base_attack(duel, 1850) && c.is_race(duel, RACE_SPELLCASTER) && c.is_faceup(duel)
}

fn damcon(duel: &Duel, a: &Args) -> bool {
    let e = a.e;
    !e.get_handler(duel).has_flag_effect(duel, ID)
        && duel.is_existing_matching_card(damfilter, e.get_handler_player(duel), LOCATION_MZONE, 0, 1, None)
}

fn damval(duel: &mut Duel, e: EffectId, _: Option<EffectId>, val: u32, r: u32, _: u8, _: Option<CardId>) -> u32 {
    if r & (REASON_BATTLE | REASON_EFFECT) == 0 {
        return val;
    }
    e.get_handler(duel).register_flag_effect(duel, ID, RESETS_STANDARD_PHASE_END, 0, 1);
    duel.hint(HINT_CARD, 0, u64::from(ID));
    0
}
