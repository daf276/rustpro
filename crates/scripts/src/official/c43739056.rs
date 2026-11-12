// Amanokujaki
use crate::prelude::*;

const ID: u32 = 43739056;

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_CANNOT_ACTIVATE);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_target_range(duel, 1, 0);
    e1.set_value(duel, Value::EffectPlayer(aclimit));
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_category(duel, CATEGORY_CONTROL);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_code(duel, EVENT_FREE_CHAIN);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, 0);
    e2.set_condition(duel, condition);
    e2.set_target(duel, target);
    e2.set_operation(duel, operation);
    c.register_effect(duel, e2);
}

fn aclimit(duel: &Duel, e: EffectId, re: EffectId, _: u8) -> bool {
    let c = e.get_handler(duel);
    let rc = re.get_handler(duel);
    rc != c
        && re.is_monster_effect(duel)
        && rc.is_on_field(duel)
        && rc.is_special_summoned(duel)
        && rc.is_attribute(duel, c.get_attribute(duel))
}

fn condition(duel: &Duel, a: &Args) -> bool {
    duel.is_turn_player(a.e.get_handler(duel).get_owner(duel)) && duel.is_main_phase()
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let c = a.e.get_handler(duel);
    if chk == 0 {
        return Ok(c.is_able_to_change_controler(duel));
    }
    duel.set_operation_info(0, CATEGORY_CONTROL, Some(c.into()), 1, 0, 0);
    Ok(false)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if c.is_relate_to_effect(duel, e) {
        duel.get_control(c, 1 - tp)?;
        if duel.select_yes_no(tp, aux::stringid(ID, 0))? {
            let att = c.announce_another_attribute(duel, tp)?;
            let e1 = duel.create_effect(c);
            e1.set_type(duel, EFFECT_TYPE_SINGLE);
            e1.set_code(duel, EFFECT_CHANGE_ATTRIBUTE);
            e1.set_value(duel, att as i32);
            e1.set_reset_full(duel, RESETS_STANDARD_DISABLE_PHASE_END, 2);
            c.register_effect(duel, e1);
        }
    }
    Ok(())
}
