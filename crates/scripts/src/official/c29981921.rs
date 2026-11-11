// Legendary Six Samurai - Shi En
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND, MZONE, OPPONENT};
use engine::constants::MSG_SELECT_EFFECTYN;

const ID: u32 = 29981921;

// In the Extra Deck, Synchro Summoned with Tactical Trainer of the Six Samurai (a Warrior Tuner) and Legendary Six Samurai - Kageki,
// with the opponent's Raigeki to negate and destroy.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (ID, EXTRA),
    (16968936, MZONE),
    (16968936, MZONE),
    (2511717, MZONE),
    (2511717, MZONE),
    (12580477, HAND | OPPONENT),
    (12580477, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::SpecialSummoned,
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::Message(MSG_SELECT_EFFECTYN),
    // The hint Tactical Trainer of the Six Samurai gives the Synchro Monster it is material for.
    Check::CardHint(aux::stringid(16968936, 2)),
];

struct Procedure;

impl SynchroProcedure for Procedure {
    const MAX2: usize = 99;
    const MIN2: usize = 1;

    fn f1(c: CardId, duel: &Duel, sc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_race_full(duel, RACE_WARRIOR, Some(sc), sumtype, tp)
    }

    fn f2(c: CardId, duel: &Duel, sc: CardId, sumtype: u64, tp: u8) -> bool {
        c.is_not_tuner(duel, sc, tp) && c.is_set_card_full(duel, &[SET_SIX_SAMURAI], Some(sc), sumtype, tp)
    }
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Synchro::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_NEGATE + CATEGORY_DESTROY);
    e1.set_property(duel, EFFECT_FLAG_DAMAGE_STEP + EFFECT_FLAG_DAMAGE_CAL);
    e1.set_type(duel, EFFECT_TYPE_QUICK_O);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_count_limit(duel, 1, 0);
    e1.set_code(duel, EVENT_CHAINING);
    e1.set_condition(duel, discon);
    e1.set_target(duel, stock::negate_destroy_target);
    e1.set_operation(duel, disop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_CONTINUOUS + EFFECT_TYPE_SINGLE);
    e2.set_code(duel, EFFECT_DESTROY_REPLACE);
    e2.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_target(duel, desreptg);
    e2.set_operation(duel, desrepop);
    c.register_effect(duel, e2);
}

fn discon(duel: &Duel, a: &Args) -> bool {
    if a.e.get_handler(duel).is_status(duel, STATUS_BATTLE_DESTROYED) {
        return false;
    }
    a.rp == 1 - a.tp && a.re.is_some_and(|re| re.is_has_type(duel, EFFECT_TYPE_ACTIVATE)) && duel.is_chain_negatable(a.ev as u8)
}

fn disop(duel: &mut Duel, a: &Args) -> Script<()> {
    let Some(re) = a.re else { return Ok(()) };
    if duel.negate_activation(a.ev as u8) && re.get_handler(duel).is_relate_to_effect(duel, re) {
        duel.destroy(a.eg.unwrap_or_default(), REASON_EFFECT)?;
    }
    Ok(())
}

fn repfilter(c: CardId, duel: &Duel, e: EffectId) -> bool {
    c.is_faceup(duel)
        && c.is_set_card(duel, &[SET_SIX_SAMURAI])
        && c.is_destructable_full(duel, e)
        && !c.is_status(duel, STATUS_DESTROY_CONFIRMED + STATUS_BATTLE_DESTROYED)
}

fn desreptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if chk == 0 {
        return Ok(!c.is_reason(duel, REASON_REPLACE)
            && c.is_on_field(duel)
            && c.is_faceup(duel)
            && duel.is_existing_matching_card(|tc, duel| repfilter(tc, duel, e), tp, LOCATION_MZONE, 0, 1, Some(c.into())));
    }
    if !duel.select_effect_yes_no_full(tp, c, 96)? {
        return Ok(false);
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESREPLACE);
    let g = duel.select_matching_card(tp, |tc, duel| repfilter(tc, duel, e), tp, LOCATION_MZONE, 0, 1, 1, Some(c.into()))?;
    e.set_label_object_group(duel, Some(g));
    if let Some(tc) = g.get_first() {
        tc.set_status(duel, STATUS_DESTROY_CONFIRMED, true);
    }
    Ok(true)
}

fn desrepop(duel: &mut Duel, a: &Args) -> Script<()> {
    if let Some(tc) = a.e.get_label_object_group(duel).and_then(|g| g.get_first()) {
        tc.set_status(duel, STATUS_DESTROY_CONFIRMED, false);
        duel.destroy(tc, REASON_EFFECT | REASON_REPLACE)?;
    }
    Ok(())
}
