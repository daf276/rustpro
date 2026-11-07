// Ghost Ogre & Snow Rabbit
use crate::prelude::*;

const ID: u32 = 59438930;

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_category(duel, CATEGORY_DESTROY);
    e1.set_type(duel, EFFECT_TYPE_QUICK_O);
    e1.set_code(duel, EVENT_CHAINING);
    e1.set_range(duel, LOCATION_MZONE | LOCATION_HAND);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, condition);
    e1.set_cost(duel, Cost::self_to_grave);
    e1.set_target(duel, target);
    e1.set_operation(duel, operation);
    c.register_effect(duel, e1);
}

fn condition(duel: &Duel, a: &Args) -> bool {
    let Some(re) = a.re else { return false };
    re.get_handler(duel).is_on_field(duel)
        && (re.is_monster_effect(duel) || (re.is_spell_trap_effect(duel) && !re.is_has_type(duel, EFFECT_TYPE_ACTIVATE)))
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(a.re.is_some_and(|re| re.get_handler(duel).is_destructable(duel)));
    }
    duel.set_operation_info(0, CATEGORY_DESTROY, a.eg, 1, 0, 0);
    Ok(false)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let Some(re) = a.re else { return Ok(()) };
    if re.get_handler(duel).is_relate_to_effect(duel, re)
        && let Some(eg) = a.eg
    {
        duel.destroy(eg, REASON_EFFECT)?;
    }
    Ok(())
}
