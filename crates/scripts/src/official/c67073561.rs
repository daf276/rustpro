// Baba Barber
use crate::prelude::*;
use crate::testing::{EXTRA, MZONE};

const ID: u32 = 67073561;

struct Procedure;

impl LinkProcedure for Procedure {
    const MIN: usize = 2;
    const MAX: Option<usize> = Some(2);
}

// In the Extra Deck, Link Summoned with two Protrons.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (24154052, MZONE),
    (24154052, MZONE),
    (24154052, MZONE),
    (24154052, MZONE),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Link::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_category(duel, CATEGORY_REMOVE);
    e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_code(duel, EVENT_PHASE | PHASE_BATTLE_START);
    e1.set_count_limit(duel, 1, ID);
    e1.set_target(duel, rmtg);
    e1.set_operation(duel, rmop);
    c.register_effect(duel, e1);
}

fn filter(c: CardId, duel: &Duel, e: EffectId) -> bool {
    let handler = e.get_handler(duel);
    (handler.get_linked_group(duel).is_contains(c) || c == handler) && c.is_able_to_remove(duel)
}

fn rmtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE) && filter(chkc, duel, e));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(|c, duel| filter(c, duel, e), tp, LOCATION_MZONE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TARGET);
    let g = duel.select_target(tp, |c, duel| filter(c, duel, e), tp, LOCATION_MZONE, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_REMOVE, g, 1, 0, 0);
    Ok(false)
}

fn rmop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if tc.is_relate_to_effect(duel, e) && duel.remove(tc, 0, REASON_EFFECT | REASON_TEMPORARY)? != 0 {
        tc.register_flag_effect(duel, ID, RESETS_STANDARD_PHASE_END, 0, 1);
        let e1 = duel.create_effect(e.get_handler(duel));
        e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
        e1.set_code(duel, EVENT_PHASE + PHASE_END);
        e1.set_reset(duel, RESET_PHASE | PHASE_END);
        e1.set_label_object_group(duel, Some(tc.into()));
        e1.set_count_limit(duel, 1, 0);
        e1.set_condition(duel, retcon);
        e1.set_operation(duel, retop);
        duel.register_effect(e1, tp);
    }
    Ok(())
}

fn retcon(duel: &Duel, a: &Args) -> bool {
    a.e.get_label_object_group(duel)
        .and_then(|g| g.get_first())
        .is_some_and(|tc| tc.get_flag_effect(duel, ID) != 0)
}

fn retop(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(tc) = a.e.get_label_object_group(duel).and_then(|g| g.get_first()) {
        duel.return_to_field(tc)?;
    }
    Ok(())
}
