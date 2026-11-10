// Wheel Synchron
use crate::prelude::*;
use crate::testing::{Check, GRAVE, HAND, MZONE};

const ID: u32 = 60283232;

// In the hand, on the field and in the Graveyard, with Junk Synchron in the hand to Normal Summon and Junk Warrior on the field
// to lower the Level of.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, MZONE),
    (ID, GRAVE),
    (CARD_JUNK_SYNCHRON, HAND),
    (CARD_JUNK_SYNCHRON, HAND),
    (CARD_JUNK_WARRIOR, MZONE),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e1.set_code(duel, EFFECT_NONTUNER);
    e1.set_range(duel, LOCATION_MZONE);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, ID);
    e2.set_target(duel, nstg);
    e2.set_operation(duel, nsop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_LVCHANGE);
    e3.set_type(duel, EFFECT_TYPE_IGNITION);
    e3.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e3.set_range(duel, LOCATION_GRAVE);
    e3.set_count_limit(duel, 1, (ID, 1));
    e3.set_cost(duel, Cost::self_banish);
    e3.set_target(duel, lvltg);
    e3.set_operation(duel, lvlop);
    c.register_effect(duel, e3);
}

fn nsfilter(c: CardId, duel: &Duel) -> bool {
    c.is_level_below(duel, 4) && c.is_summonable(duel, true, None)
}

fn nstg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(nsfilter, tp, LOCATION_HAND | LOCATION_MZONE, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_SUMMON, None, 1, 0, (LOCATION_HAND | LOCATION_MZONE) as i32);
    Ok(false)
}

fn nsop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let c = a.e.get_handler(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 2));
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_CLIENT_HINT);
    e1.set_code(duel, EFFECT_CANNOT_SPECIAL_SUMMON);
    e1.set_target_range(duel, 1, 0);
    e1.set_target_spsummon(duel, |duel, _, c, _, _, _, _, _| {
        c.is_location(duel, LOCATION_EXTRA) && !c.is_type(duel, TYPE_SYNCHRO)
    });
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e1, tp);
    aux::add_temp_lizard_check_full(duel, c, tp, |duel, _, c| !c.is_original_type(duel, TYPE_SYNCHRO));
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SUMMON);
    if let Some(sc) = duel
        .select_matching_card(tp, nsfilter, tp, LOCATION_HAND | LOCATION_MZONE, 0, 1, 1, None)?
        .get_first()
    {
        duel.summon(tp, sc, true, None)?;
    }
    Ok(())
}

fn synchfilter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_type(duel, TYPE_SYNCHRO) && c.is_level_above(duel, 2)
}

fn lvltg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE) && chkc.is_controler(duel, tp) && synchfilter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(synchfilter, tp, LOCATION_MZONE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_APPLYTO);
    duel.select_target(tp, synchfilter, tp, LOCATION_MZONE, 0, 1, 1, None)?;
    Ok(false)
}

fn lvlop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if !(tc.is_faceup(duel) && tc.is_relate_to_effect(duel, e)) {
        return Ok(());
    }
    let lvl = tc.get_level(duel);
    if lvl == 1 {
        return Ok(());
    }
    unported!(duel, lvl == 0);
    let value = duel.announce_number_range(tp, 1, 4.min(lvl.saturating_sub(1)))?;
    let e1 = duel.create_effect(e.get_handler(duel));
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
    e1.set_code(duel, EFFECT_UPDATE_LEVEL);
    e1.set_value(duel, -(value as i32));
    e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
    tc.register_effect(duel, e1);
    Ok(())
}
