// Megalith Unformed
use crate::prelude::*;
use crate::testing::{Check, DECK, HAND, MZONE, OPPONENT};

const ID: u32 = 69003792;

pub const LISTED_SERIES: &[u16] = &[SET_MEGALITH];

struct Params;

impl RitualParams for Params {
    const LOCATION: Option<u32> = Some(LOCATION_DECK);
    const SUMPOS: u32 = POS_FACEUP_DEFENSE;

    fn filter(c: CardId, duel: &Duel) -> bool {
        c.is_set_card(duel, &[SET_MEGALITH])
    }

    fn lv(c: CardId, duel: &Duel) -> u32 {
        c.get_level(duel) * 2
    }
}

// In the hand, with Megalith Och (Level 4) in the Deck to summon by Tributing two Level 4 Normal Monsters (Kojikocy) in the
// hand, and on the field facing an opponent's face-up monster.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (36849933, DECK),
    (36849933, DECK),
    (36849933, MZONE),
    (1184620, HAND),
    (1184620, HAND),
    (13597785, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(0), Check::SpecialSummonedCode(36849933)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_hint_timing(duel, TIMING_DAMAGE_STEP, TIMING_DAMAGE_STEP | TIMING_END_PHASE);
    e1.set_target(duel, target);
    c.register_effect(duel, e1);
}

fn target(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let ritual = aux::faceup_filter(|c, duel| c.is_type(duel, TYPE_RITUAL));
    let b1 = duel.get_matching_group_count(CardId::is_faceup, tp, 0, LOCATION_MZONE, None) > 0
        && duel.get_matching_group_count(ritual, tp, LOCATION_MZONE, LOCATION_MZONE, None) > 0;
    let b2 = Ritual::target::<Params, RITPROC_EQUAL>(duel, a, 0, None)?;
    if chk == 0 {
        return Ok(b1 || b2);
    }
    let op = duel.select_effect(tp, &[(b1, aux::stringid(ID, 0)), (b2, aux::stringid(ID, 1))])?;
    e.set_label(duel, op.map_or(0, |op| op as i64));
    if op == Some(1) {
        e.set_category(duel, CATEGORY_ATKCHANGE);
        e.set_operation(duel, atkop);
    } else if op == Some(2) {
        e.set_category(duel, CATEGORY_SPECIAL_SUMMON);
        e.set_operation(duel, Ritual::operation::<Params, RITPROC_EQUAL>);
        Ritual::target::<Params, RITPROC_EQUAL>(duel, a, 1, None)?;
    }
    Ok(false)
}

fn atkop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    let g = duel.get_matching_group(CardId::is_faceup, tp, 0, LOCATION_MZONE, None);
    let ritual = aux::faceup_filter(|c, duel| c.is_type(duel, TYPE_RITUAL));
    let atk = duel.get_matching_group_count(ritual, tp, LOCATION_MZONE, LOCATION_MZONE, None) as i32 * -500;
    if g.is_empty() || atk == 0 {
        return Ok(());
    }
    let c = a.e.get_handler(duel);
    for tc in g {
        let e1 = duel.create_effect(c);
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_UPDATE_ATTACK);
        e1.set_value(duel, atk);
        e1.set_reset(duel, RESETS_STANDARD_PHASE_END);
        tc.register_effect(duel, e1);
    }
    Ok(())
}
