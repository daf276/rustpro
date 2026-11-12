// Laval the Greater
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, HAND, MZONE, OPPONENT};
use engine::constants::MSG_SELECT_EFFECTYN;

const ID: u32 = 12986807;

struct Procedure;

impl SynchroProcedure for Procedure {
    const MIN2: usize = 1;
    const MAX2: usize = 99;

    fn f2(c: CardId, duel: &Duel, sc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_not_tuner(duel, sc, tp) && c.is_attribute_full(duel, ATTRIBUTE_FIRE, Some(sc), sumtype, tp)
    }
}

// Laval Coatl (Level 2 Tuner) with Darkfire Soldier #1 for a Synchro Summon from the Extra Deck, a Laval monster in the Graveyard
// to banish, and the opponent's Dark Holes to destroy it.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (45439263, MZONE),
    (45439263, MZONE),
    (5388481, MZONE),
    (5388481, MZONE),
    (2407147, GRAVE),
    (53129443, HAND | OPPONENT),
    (53129443, HAND | OPPONENT),
];
// The destruction replacement is no chain link and moves no card; its prompt is the MSG_SELECT_EFFECTYN.
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0)), Check::Message(MSG_SELECT_EFFECTYN)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Synchro::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_HANDES);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_F);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_condition(duel, condition);
    e1.set_target(duel, stock::operation_info_target::<CATEGORY_TOGRAVE, LOCATION_HAND>);
    e1.set_operation(duel, operation);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_CONTINUOUS + EFFECT_TYPE_SINGLE);
    e2.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_code(duel, EFFECT_DESTROY_REPLACE);
    e2.set_target(duel, desreptg);
    c.register_effect(duel, e2);
}

fn condition(duel: &Duel, a: &Args) -> bool {
    a.e.get_handler(duel).is_synchro_summoned(duel)
}

fn operation(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOGRAVE);
    let g = duel.select_matching_card(tp, aux::TRUE, tp, LOCATION_HAND, 0, 1, 1, None)?;
    duel.send_to_grave(g, REASON_EFFECT)?;
    Ok(())
}

fn repfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_LAVAL]) && c.is_able_to_remove_as_cost(duel) && aux::sp_elim_filter(c, duel, true, false)
}

fn desreptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (tp, c) = (a.tp, a.e.get_handler(duel));
    if chk == 0 {
        return Ok(!c.is_reason(duel, REASON_REPLACE)
            && c.is_reason(duel, REASON_EFFECT)
            && duel.is_existing_matching_card(repfilter, tp, LOCATION_MZONE | LOCATION_GRAVE, 0, 1, None));
    }
    if duel.select_effect_yes_no_full(tp, c, 96)? {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_REMOVE);
        let g = duel.select_matching_card(tp, repfilter, tp, LOCATION_MZONE | LOCATION_GRAVE, 0, 1, 1, None)?;
        duel.remove(g, POS_FACEUP, REASON_COST)?;
        return Ok(true);
    }
    Ok(false)
}
