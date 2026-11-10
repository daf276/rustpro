// Bazoo the Soul-Eater
use crate::prelude::*;
use crate::testing::{GRAVE, MZONE};

const ID: u32 = 40133511;

// On the field with monsters in the Graveyard and on the field to banish.
pub const SUPPORT: &[(u32, u16)] = &[(ID, MZONE), (ID, MZONE), (26202165, GRAVE), (26202165, GRAVE), (26202165, MZONE)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_ATKCHANGE);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, 0);
    e1.set_cost(duel, cost);
    e1.set_operation(duel, operation);
    c.register_effect(duel, e1);
}

fn cfilter(c: CardId, duel: &Duel) -> bool {
    c.is_monster(duel) && c.is_able_to_remove_as_cost(duel) && aux::sp_elim_filter(c, duel, false, false)
}

fn cost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(cfilter, tp, LOCATION_MZONE | LOCATION_GRAVE, 0, 1, Some(c.into())));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_REMOVE);
    let cg = duel.select_matching_card(tp, cfilter, tp, LOCATION_MZONE | LOCATION_GRAVE, 0, 1, 3, Some(c.into()))?;
    duel.remove(cg, POS_FACEUP, REASON_COST)?;
    e.set_label(duel, cg.len() as i64);
    Ok(false)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    let count = a.e.get_label(duel);
    if c.is_faceup(duel) && c.is_relate_to_effect(duel, a.e) {
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_UPDATE_ATTACK);
        e1.set_value(duel, count as i32 * 300);
        e1.set_reset_full(duel, RESETS_STANDARD_DISABLE_PHASE_END, 2);
        c.register_effect(duel, e1);
    }
    Ok(())
}
