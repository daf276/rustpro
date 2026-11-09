// Silvera, Wolf Tamer of the White Forest
use crate::prelude::*;
use crate::testing::{EXTRA, MZONE};

const ID: u32 = 41924516;

struct Procedure;

impl SynchroProcedure for Procedure {
    const MIN2: usize = 1;
    const MAX2: usize = 99;

    fn f2(c: CardId, duel: &Duel, sc: CardId, _: u64, tp: u8) -> bool {
        Synchro::non_tuner(c, duel, sc, tp, aux::TRUE)
    }
}

// Two copies in the Extra Deck, Synchro Summoned from Silvy of the White Forest and Astellar of the White Forest on the field.
pub const SUPPORT: &[(u32, u16)] = &[(ID, EXTRA), (ID, EXTRA), (98385955, MZONE), (25592142, MZONE)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Synchro::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_POSITION + CATEGORY_SET);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_DELAY);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_target(duel, postg);
    e1.set_operation(duel, posop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD);
    e2.set_code(duel, EFFECT_UPDATE_ATTACK);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_target_range(duel, LOCATION_MZONE, 0);
    e2.set_target_card(duel, |duel, _, c| {
        c.is_race(duel, RACE_ILLUSION | RACE_SPELLCASTER) && c.is_type(duel, TYPE_SYNCHRO)
    });
    e2.set_value(duel, 500);
    c.register_effect(duel, e2);
    let e3 = e2.clone_effect(duel);
    e3.set_code(duel, EFFECT_PIERCE);
    e3.set_value(duel, DOUBLE_DAMAGE as i32);
    c.register_effect(duel, e3);
}

fn postg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(CardId::is_can_turn_set, tp, 0, LOCATION_MZONE, 1, None));
    }
    let g = duel.get_matching_group(CardId::is_can_turn_set, tp, 0, LOCATION_MZONE, None);
    duel.set_operation_info(0, CATEGORY_POSITION, Some(g), g.len() as u8, tp, POS_FACEDOWN_DEFENSE as i32);
    Ok(false)
}

fn posop(duel: &mut Duel, a: &Args) -> Script<()> {
    let g = duel.get_matching_group(CardId::is_can_turn_set, a.tp, 0, LOCATION_MZONE, None);
    if !g.is_empty() {
        duel.change_position(g, POS_FACEDOWN_DEFENSE)?;
    }
    Ok(())
}
