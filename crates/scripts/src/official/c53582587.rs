// Torrential Tribute
use crate::prelude::*;

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    for code in [EVENT_SUMMON_SUCCESS, EVENT_FLIP_SUMMON_SUCCESS, EVENT_SPSUMMON_SUCCESS] {
        let e = duel.create_effect(c);
        e.set_category(duel, CATEGORY_DESTROY);
        e.set_type(duel, EFFECT_TYPE_ACTIVATE);
        e.set_code(duel, code);
        e.set_target(duel, target);
        e.set_operation(duel, activate);
        c.register_effect(duel, e);
    }
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(aux::TRUE, a.tp, LOCATION_MZONE, LOCATION_MZONE, 1, None));
    }
    let g = duel.get_matching_group(aux::TRUE, a.tp, LOCATION_MZONE, LOCATION_MZONE, None);
    duel.set_operation_info(0, CATEGORY_DESTROY, Some(g), g.len() as u8, 0, 0);
    Ok(false)
}

fn activate(duel: &mut Duel, a: &Args) -> Script<()> {
    let g = duel.get_matching_group(aux::TRUE, a.tp, LOCATION_MZONE, LOCATION_MZONE, None);
    if !g.is_empty() {
        duel.destroy(g, REASON_EFFECT)?;
    }
    Ok(())
}
