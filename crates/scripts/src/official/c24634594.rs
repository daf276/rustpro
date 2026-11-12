// Diabolantis the Menacing Mantis
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, MZONE};

const ID: u32 = 24634594;

// In the Extra Deck, Synchro Summoned with Zeta (Level 4 Tuner) and Gokibore (Level 4 Insect), with more Gokibore on the field and in the Deck.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (42026772, MZONE),
    (15367030, MZONE),
    (15367030, MZONE),
    (15367030, MZONE),
    (15367030, DECK),
    (15367030, DECK),
    (15367030, DECK),
];

pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

struct Procedure;

impl SynchroProcedure for Procedure {
    const MIN2: usize = 1;
    const MAX2: usize = 99;

    fn f2(c: CardId, duel: &Duel, sc: CardId, _: u64, tp: u8) -> bool {
        Synchro::non_tuner(c, duel, sc, tp, aux::TRUE)
    }
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Synchro::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOGRAVE);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_property(duel, EFFECT_FLAG_DELAY);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_count_limit(duel, 1, (ID, 0));
    e1.set_condition(duel, syncon);
    e1.set_target(duel, gytg);
    e1.set_operation(duel, gyop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, (ID, 1));
    e2.set_condition(duel, syncon);
    e2.set_target(duel, tuntg);
    e2.set_operation(duel, tunop);
    c.register_effect(duel, e2);
}

fn syncon(duel: &Duel, a: &Args) -> bool {
    a.e.get_handler(duel).is_synchro_summoned(duel)
}

fn gyfilter(c: CardId, duel: &Duel) -> bool {
    c.is_race(duel, RACE_INSECT | RACE_PLANT) && c.is_able_to_grave(duel)
}

fn gytg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    let ct = a.e.get_handler(duel).get_material_count(duel) as i64 - 1;
    if chk == 0 {
        return Ok(ct > 0 && duel.is_existing_matching_card(gyfilter, tp, LOCATION_DECK, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_TOGRAVE, None, 1, tp, LOCATION_DECK as i32);
    Ok(false)
}

fn gyop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let ct = a.e.get_handler(duel).get_material_count(duel) as i64 - 1;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOGRAVE);
    let g = duel.select_matching_card(tp, gyfilter, tp, LOCATION_DECK, 0, 1, ct as u8, None)?;
    if !g.is_empty() {
        duel.send_to_grave(g, REASON_EFFECT)?;
    }
    Ok(())
}

fn tunfilter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && !c.is_type(duel, TYPE_TUNER) && c.is_race(duel, RACE_INSECT | RACE_PLANT)
}

fn tuntg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE) && chkc.is_controler(duel, tp) && tunfilter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(tunfilter, tp, LOCATION_MZONE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TARGET);
    duel.select_target(tp, tunfilter, tp, LOCATION_MZONE, 0, 1, 1, None)?;
    Ok(false)
}

fn tunop(duel: &mut Duel, a: &Args) -> Script<()> {
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if tc.is_relate_to_effect(duel, a.e) && tc.is_faceup(duel) && !tc.is_type(duel, TYPE_TUNER) {
        let e1 = duel.create_effect(a.e.get_handler(duel));
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
        e1.set_code(duel, EFFECT_ADD_TYPE);
        e1.set_value(duel, TYPE_TUNER as i32);
        e1.set_reset(duel, RESETS_STANDARD_PHASE_END);
        tc.register_effect(duel, e1);
    }
    Ok(())
}
