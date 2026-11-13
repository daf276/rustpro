// Angello Vaalmonica
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, GRAVE, HAND, MZONE, OPPONENT};
use engine::Operation;
use engine::constants::MSG_ADD_COUNTER;

const ID: u32 = 3048768;
const DIMONNO: u32 = 30432463;
const AGATHOKAKOLOGICAL_VOICE: u32 = 39210885;

pub const LISTED_NAMES: &[u32] = &[DIMONNO];

// Copies in the hand to place with Dimonno in the Pendulum Zone (which needs both zones free), Vaalmonica Spells in the Graveyard to copy and Link Monsters
// in the Extra Deck to Link Summon.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (DIMONNO, DECK),
    (DIMONNO, DECK),
    (65496951, HAND),
    (42193638, HAND),
    (5605529, HAND),
    (78598237, HAND),
    (5605529, GRAVE),
    (78598237, GRAVE),
    (2815176, EXTRA),
    (2815176, EXTRA),
    (76821171, EXTRA),
    (76821171, EXTRA),
    (26202165, MZONE),
    (26202165, MZONE),
    (26202165, MZONE | OPPONENT),
    (26202165, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::ResolvesEffect(aux::stringid(ID, 2)),
    Check::Message(MSG_ADD_COUNTER),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Pendulum::add_procedure(duel, c);
    c.enable_counter_permit_full(duel, COUNTER_RESONANCE, LOCATION_PZONE as u16);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e1.set_code(duel, EVENT_RECOVER);
    e1.set_range(duel, LOCATION_PZONE);
    e1.set_condition(duel, |duel, a| {
        a.ep == a.tp
            && duel.is_existing_matching_card(
                |c, duel| c.is_original_race(duel, RACE_FIEND),
                a.tp,
                LOCATION_PZONE,
                0,
                1,
                Some(a.e.get_handler(duel).into()),
            )
    });
    e1.set_operation(duel, ctop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e2.set_code(duel, EVENT_ATTACK_ANNOUNCE);
    e2.set_range(duel, LOCATION_PZONE);
    e2.set_count_limit(duel, 1, 0);
    e2.set_condition(duel, |duel, a| {
        duel.get_attacker().is_some_and(|attacker| attacker.is_controler(duel, 1 - a.tp))
    });
    e2.set_target(duel, linksptg);
    e2.set_operation(duel, linkspop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_type(duel, EFFECT_TYPE_IGNITION);
    e3.set_range(duel, LOCATION_HAND);
    e3.set_count_limit(duel, 1, (ID, 0));
    e3.set_cost(duel, pzcost);
    e3.set_target(duel, pztg::<DIMONNO>);
    e3.set_operation(duel, pzop::<DIMONNO>);
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_description(duel, aux::stringid(ID, 2));
    e4.set_type(duel, EFFECT_TYPE_IGNITION);
    e4.set_range(duel, LOCATION_MZONE);
    e4.set_count_limit(duel, 1, (ID, 1));
    e4.set_condition(duel, |duel, a| a.e.get_handler(duel).has_flag_effect(duel, ID));
    e4.set_cost(duel, cpcost);
    e4.set_target(duel, cptg);
    e4.set_operation(duel, cpop::<1>);
    c.register_effect(duel, e4);
    aux::global_check(duel, ID, |duel| {
        let ge1 = duel.create_effect(c);
        ge1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
        ge1.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE);
        ge1.set_code(duel, EVENT_SUMMON_SUCCESS);
        ge1.set_label(duel, i64::from(ID));
        ge1.set_operation(duel, aux::sumreg);
        duel.register_effect(ge1, 0);
        let ge2 = ge1.clone_effect(duel);
        ge2.set_code(duel, EVENT_SPSUMMON_SUCCESS);
        duel.register_effect(ge2, 0);
    });
}

// The effects Angello and Dimonno Vaalmonica share.
pub fn ctop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if !duel.is_chain_solving() {
        return addcounter(duel, a);
    }
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e1.set_code(duel, EVENT_CHAIN_SOLVED);
    e1.set_range(duel, LOCATION_PZONE);
    e1.set_operation(duel, addcounter);
    e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD | RESET_CHAIN);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
    e2.set_code(duel, EVENT_CHAIN_SOLVED);
    e2.set_operation(duel, |duel, a| {
        if let Some(e1) = a.e.get_label_object(duel) {
            e1.reset(duel);
        }
        Ok(())
    });
    e2.set_label_object(duel, Some(e1));
    e2.set_reset(duel, RESET_CHAIN);
    duel.register_effect(e2, tp);
    Ok(())
}

fn addcounter(duel: &mut Duel, a: &Args) -> Script<()> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    c.add_counter(duel, COUNTER_RESONANCE, 1);
    duel.raise_event(c, EVENT_CUSTOM + AGATHOKAKOLOGICAL_VOICE, Some(a.e), 0, tp, tp, 1)
}

fn linkfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_VAALMONICA]) && c.is_link_summonable(duel, None, None)
}

fn linksptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, linkfilter, LOCATION_EXTRA, CATEGORY_SPECIAL_SUMMON)
}

fn linkspop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_matching_card(tp, linkfilter, tp, LOCATION_EXTRA, 0, 1, 1, None)?;
    if let Some(sc) = g.get_first() {
        duel.link_summon(tp, sc, None, None)?;
    }
    Ok(())
}

pub fn pzcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(CardId::is_discardable, tp, LOCATION_HAND, 0, 1, Some(c.into())));
    }
    duel.discard_hand_full(tp, CardId::is_discardable, 1, 1, REASON_COST | REASON_DISCARD, Some(c.into()))?;
    Ok(false)
}

fn pzfilter<const PARTNER: u32>(c: CardId, duel: &Duel) -> bool {
    c.is_code(duel, PARTNER) && !c.is_forbidden(duel)
}

pub fn pztg<const PARTNER: u32>(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    Ok(chk == 0
        && duel.check_location(tp, LOCATION_PZONE, 0)
        && duel.check_location(tp, LOCATION_PZONE, 1)
        && duel.is_existing_matching_card(pzfilter::<PARTNER>, tp, LOCATION_DECK, 0, 1, None))
}

pub fn pzop<const PARTNER: u32>(duel: &mut Duel, a: &Args) -> Script<()> {
    let (c, tp) = (a.e.get_handler(duel), a.tp);
    if !(duel.check_location(tp, LOCATION_PZONE, 0) && duel.check_location(tp, LOCATION_PZONE, 1) && c.is_relate_to_effect(duel, a.e)) {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_TOFIELD);
    let g = duel.select_matching_card(tp, pzfilter::<PARTNER>, tp, LOCATION_DECK, 0, 1, 1, None)?;
    let Some(tc) = g.get_first() else { return Ok(()) };
    duel.move_to_field(c, tp, tp, LOCATION_PZONE, POS_FACEUP, true)?;
    duel.move_to_field(tc, tp, tp, LOCATION_PZONE, POS_FACEUP, true)?;
    Ok(())
}

pub fn cpcost(duel: &mut Duel, a: &Args, _: i32) -> Script<bool> {
    a.e.set_label(duel, 1);
    Ok(true)
}

fn cpfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_VAALMONICA]) && (c.is_normal_spell(duel) || c.is_normal_trap(duel)) && c.is_able_to_remove_as_cost(duel)
}

pub fn cptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        if e.get_label(duel) == 0 {
            return Ok(false);
        }
        e.set_label(duel, 0);
        return Ok(duel.is_existing_matching_card(cpfilter, tp, LOCATION_GRAVE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_REMOVE);
    let g = duel.select_matching_card(tp, cpfilter, tp, LOCATION_GRAVE, 0, 1, 1, None)?;
    duel.remove(g, POS_FACEUP, REASON_COST)?;
    e.set_label_object_group(duel, Some(g));
    Ok(false)
}

// The operation of the Spell or Trap with `EXTRA` as its `angello_or_dimonno` parameter.
pub fn cpop<const EXTRA: u8>(duel: &mut Duel, a: &Args) -> Script<()> {
    let Some(tc) = a.e.get_label_object_group(duel).and_then(|g| g.get_first()) else {
        return Ok(());
    };
    let Some(op) = tc.get_activate_effect(duel).and_then(|eff| eff.get_operation(duel)) else {
        return Ok(());
    };
    unported!(duel, !matches!(op, Operation::Event(_) | Operation::Promptless(_)));
    if let Operation::Event(op) | Operation::Promptless(op) = op {
        op(duel, &Args { extra: EXTRA, ..*a })?;
    }
    Ok(())
}
