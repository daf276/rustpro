// Uni-Horned Familiar
use crate::prelude::*;
use crate::testing::{MZONE, OPPONENT};

const ID: u32 = 22318971;

// Copies on the field, one to banish as the cost, with Elfnote Lucina on the opponent's side to attack them.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, MZONE),
    (ID, MZONE),
    (13597785, MZONE | OPPONENT),
    (13597785, MZONE | OPPONENT),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_REMOVE);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_code(duel, EVENT_BE_BATTLE_TARGET);
    e1.set_condition(duel, condition);
    e1.set_cost(duel, cost);
    e1.set_target(duel, target);
    e1.set_operation(duel, operation);
    c.register_effect(duel, e1);
}

fn condition(duel: &Duel, a: &Args) -> bool {
    a.e.get_handler(duel).is_position(duel, POS_FACEUP_DEFENSE)
}

fn cost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let (tp, handler) = (a.tp, Some(a.e.get_handler(duel).into()));
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(CardId::is_able_to_remove, tp, LOCATION_MZONE, 0, 1, handler));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_REMOVE);
    let g = duel.select_matching_card(tp, CardId::is_able_to_remove, tp, LOCATION_MZONE, 0, 1, 1, handler)?;
    duel.remove(g, POS_FACEUP, REASON_COST)?;
    Ok(false)
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let c = a.e.get_handler(duel);
    if chk == 0 {
        return Ok(c.is_able_to_remove(duel));
    }
    let attacker = duel.get_attacker();
    unported!(duel, attacker.is_none());
    if let Some(attacker) = attacker {
        duel.set_target_card(attacker);
    }
    duel.set_operation_info(0, CATEGORY_REMOVE, Some(c.into()), 1, 0, 0);
    Ok(false)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if c.is_relate_to_effect(duel, e) && c.is_faceup(duel) && duel.remove(c, 0, REASON_EFFECT | REASON_TEMPORARY)? != 0 {
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
        e1.set_code(duel, EVENT_PHASE + PHASE_STANDBY);
        e1.set_reset(duel, RESET_PHASE | PHASE_STANDBY | RESET_SELF_TURN);
        e1.set_count_limit(duel, 1, 0);
        e1.set_condition(duel, retcon);
        e1.set_operation(duel, retop);
        duel.register_effect(e1, tp);
        let ac = duel.get_first_target();
        unported!(duel, ac.is_none());
        if let Some(ac) = ac
            && ac.is_relate_to_effect(duel, e)
            && ac.is_faceup(duel)
        {
            let e2 = duel.create_effect(c);
            e2.set_type(duel, EFFECT_TYPE_SINGLE);
            e2.set_code(duel, EFFECT_MUST_ATTACK);
            e2.set_reset(duel, RESETS_STANDARD_PHASE_END);
            ac.register_effect(duel, e2);
        }
    }
    Ok(())
}

fn retcon(duel: &Duel, a: &Args) -> bool {
    duel.is_turn_player(a.tp)
}

fn retop(duel: &mut Duel, a: &Args) -> Script<()> {
    let owner = a.e.get_owner(duel);
    duel.return_to_field(owner)?;
    Ok(())
}
