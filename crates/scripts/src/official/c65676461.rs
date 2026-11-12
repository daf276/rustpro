// Number 32: Shark Drake
use crate::prelude::*;
use crate::testing::{MZONE, OPPONENT};

const ID: u32 = 65676461;
pub const XYZ_NUMBER: u32 = 32;

// On the field with weak opponent's monsters (Key Mace) to destroy by battle.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, MZONE),
    (ID, MZONE),
    (1929294, MZONE | OPPONENT),
    (1929294, MZONE | OPPONENT),
    (1929294, MZONE | OPPONENT),
];

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(4);
    const CT: usize = 3;
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Xyz::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_count_limit(duel, 1, 0);
    e1.set_code(duel, EVENT_BATTLE_DESTROYING);
    e1.set_condition(duel, atcon);
    e1.set_cost(duel, Cost::detach_from_self::<1, 1>);
    e1.set_target(duel, attg);
    e1.set_operation(duel, atop);
    c.register_effect(duel, e1);
}

fn atcon(duel: &Duel, a: &Args) -> bool {
    let c = a.e.get_handler(duel);
    if !(duel.get_attacker() == Some(c) && c.is_relate_to_battle(duel) && c.is_status(duel, STATUS_OPPO_BATTLE)) {
        return false;
    }
    let bc = c.get_battle_target(duel);
    unported!(duel, bc.is_none());
    bc.is_some_and(|bc| bc.is_location(duel, LOCATION_GRAVE) && bc.is_monster(duel))
}

fn attg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let bc = duel.get_attack_target();
    unported!(duel, bc.is_none());
    let Some(bc) = bc else { return Ok(false) };
    if chk == 0 {
        return Ok(duel.get_location_count(1 - tp, LOCATION_MZONE) > 0
            && bc.is_can_be_special_summoned_full(duel, e, 0, tp, false, false, POS_FACEUP_ATTACK, 1 - tp, 0xff));
    }
    bc.create_effect_relation(duel, e);
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, Some(bc.into()), 1, 0, 0);
    Ok(false)
}

fn atop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp, c) = (a.e, a.tp, a.e.get_handler(duel));
    let bc = duel.get_attack_target();
    unported!(duel, bc.is_none());
    let Some(bc) = bc else { return Ok(()) };
    if !bc.is_relate_to_effect(duel, e) || !duel.special_summon_step(bc, 0, tp, 1 - tp, false, false, POS_FACEUP_ATTACK)? {
        return Ok(());
    }
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_UPDATE_ATTACK);
    e1.set_value(duel, -1000);
    e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
    bc.register_effect(duel, e1);
    duel.special_summon_complete()?;
    if c.is_faceup(duel) && c.is_relate_to_effect(duel, e) {
        duel.break_effect()?;
        let e1 = duel.create_effect(c);
        e1.set_description(duel, 3201);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_CLIENT_HINT);
        e1.set_code(duel, EFFECT_EXTRA_ATTACK);
        e1.set_value(duel, 1);
        e1.set_reset(duel, RESETS_STANDARD_PHASE_END);
        c.register_effect(duel, e1);
    }
    Ok(())
}
