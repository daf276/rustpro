// Gold Pride - Eradicator
use super::c22390469::{spop, sptg};
use super::c37675907::Procedure;
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, MZONE, OPPONENT};

const ID: u32 = 45464587;
const GOLD_PRIDE_ASSASSINATOR: u32 = 60203670;

pub const LISTED_NAMES: &[u32] = &[GOLD_PRIDE_ASSASSINATOR];

// In the Extra Deck, Synchro Summoned with Gold Pride - Leon (a Level 3 Tuner) and Level 5 Eliminator, with Extra Deck monsters on both
// sides (Iron Chain Dragon, several so that it seldom picks itself) to destroy on the opponent's turn and Eliminator in the Deck to
// Special Summon.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (23512906, MZONE),
    (23512906, MZONE),
    (GOLD_PRIDE_ASSASSINATOR, MZONE),
    (GOLD_PRIDE_ASSASSINATOR, DECK),
    (GOLD_PRIDE_ASSASSINATOR, DECK),
    (19974580, MZONE | OPPONENT),
    (19974580, MZONE | OPPONENT),
    (19974580, MZONE | OPPONENT),
    (19974580, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::SpecialSummonedCode(GOLD_PRIDE_ASSASSINATOR),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    Synchro::add_procedure::<Procedure>(duel, c);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_DESTROY);
    e1.set_type(duel, EFFECT_TYPE_QUICK_O);
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_hint_timing(duel, 0, TIMINGS_CHECK_MONSTER_E);
    e1.set_count_limit(duel, 1, ID);
    e1.set_condition(duel, |duel, a| duel.is_turn_player(1 - a.tp));
    e1.set_target(duel, destg);
    e1.set_operation(duel, desop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_TOEXTRA + CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_F);
    e2.set_code(duel, EVENT_PHASE + PHASE_END);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, 0);
    e2.set_condition(duel, |duel, a| a.e.get_handler(duel).has_flag_effect(duel, ID));
    e2.set_target(duel, sptg);
    e2.set_operation(duel, spop::<GOLD_PRIDE_ASSASSINATOR>);
    c.register_effect(duel, e2);
}

fn extra_faceup(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_type(duel, TYPE_EXTRA)
}

fn destg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE) && extra_faceup(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(extra_faceup, tp, LOCATION_MZONE, LOCATION_MZONE, 1, None));
    }
    let ct = if duel.get_lp(tp) < duel.get_lp(1 - tp) { 2 } else { 1 };
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_DESTROY);
    let g = duel.select_target(tp, extra_faceup, tp, LOCATION_MZONE, LOCATION_MZONE, 1, ct, None)?;
    duel.set_operation_info(0, CATEGORY_DESTROY, g, g.map_or(0, |g| g.len() as u8), 0, 0);
    a.e.get_handler(duel)
        .register_flag_effect(duel, ID, RESETS_STANDARD_PHASE_END, EFFECT_FLAG_OATH, 1);
    Ok(false)
}

fn desop(duel: &mut Duel, a: &Args) -> Script<()> {
    let g = duel.get_target_cards(a.e).unwrap_or_default();
    if !g.is_empty() {
        duel.destroy(g, REASON_EFFECT)?;
    }
    Ok(())
}
