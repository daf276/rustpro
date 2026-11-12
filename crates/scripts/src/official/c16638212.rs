// D.D. Sprite
use crate::prelude::*;
use crate::testing::{Check, HAND, MZONE};

const ID: u32 = 16638212;

// Copies in the hand to Special Summon, banishing a monster on the field (Key Mace, a Normal Monster) that returns at the next Standby Phase.
pub const SUPPORT: &[(u32, u16)] = &[(ID, HAND), (ID, HAND), (1929294, MZONE), (1929294, MZONE)];
pub const CHECKS: &[Check] = &[Check::SpecialSummoned, Check::ResolvesEffect(aux::stringid(ID, 0))];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_code(duel, EFFECT_SPSUMMON_PROC);
    e1.set_property(duel, EFFECT_FLAG_UNCOPYABLE);
    e1.set_range(duel, LOCATION_HAND);
    e1.set_condition(duel, spcon);
    e1.set_target(duel, sptg);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    if a.c.is_none() {
        return true;
    }
    let filter = |c: CardId, duel: &Duel| c.is_able_to_remove_as_cost(duel) && c.is_faceup(duel);
    duel.is_existing_matching_card(filter, a.e.get_handler_player(duel), LOCATION_MZONE, 0, 1, None)
}

fn sptg(duel: &mut Duel, a: &Args, _: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let rg = duel
        .get_matching_group(CardId::is_able_to_remove_as_cost, tp, LOCATION_MZONE, 0, None)
        .filter(duel, CardId::is_faceup, None);
    let g = aux::select_unselect_group_full(duel, rg, 1, 1, &aux::chkf_mmz(tp, 1), tp, HINTMSG_REMOVE, None, true)?;
    if g.is_empty() {
        return Ok(false);
    }
    e.set_label_object_group(duel, Some(g));
    Ok(true)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let Some(g) = a.e.get_label_object_group(duel) else { return Ok(()) };
    duel.remove(g, POS_FACEUP, REASON_COST + REASON_TEMPORARY)?;
    let (Some(tc), Some(c)) = (g.get_first(), a.c) else { return Ok(()) };
    tc.register_flag_effect(duel, ID, RESET_EVENT | RESETS_STANDARD, 0, 0);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_F);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_code(duel, EVENT_PHASE | PHASE_STANDBY);
    e1.set_count_limit(duel, 1, 0);
    e1.set_reset(
        duel,
        RESET_EVENT | (RESETS_STANDARD_DISABLE & !RESET_TOFIELD) | RESET_PHASE | PHASE_STANDBY,
    );
    e1.set_operation(duel, retop);
    e1.set_label_object_group(duel, Some(tc.into()));
    c.register_effect(duel, e1);
    Ok(())
}

fn retop(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(tc) = a.e.get_label_object_group(duel).and_then(|g| g.get_first())
        && tc.get_flag_effect(duel, ID) != 0
    {
        duel.return_to_field(tc)?;
    }
    Ok(())
}
