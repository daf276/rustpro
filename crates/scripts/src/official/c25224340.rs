// Nightmare Hands
use crate::prelude::*;
use crate::testing::{Check, DECK, GRAVE, HAND, MZONE, OPPONENT};

const ID: u32 = 25224340;

// Illusion Normal Monsters: Jongleur-Ghoul Illusionist in the Deck to send, Pesanta (Level 8) on the field; Psychic Kappa in the
// opponent's Graveyard to summon.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, GRAVE),
    (47894537, DECK),
    (47894537, DECK),
    (66429798, MZONE),
    (7892180, GRAVE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_TOGRAVE);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_count_limit(duel, 1, ID);
    e1.set_target(duel, tgtg);
    e1.set_operation(duel, tgop);
    e1.set_hint_timing(duel, 0, TIMING_STANDBY_PHASE | TIMING_MAIN_END | TIMINGS_CHECK_MONSTER_E);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON + CATEGORY_REMOVE);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_count_limit(duel, 1, ID);
    e2.set_condition(duel, spcon);
    e2.set_cost(duel, Cost::self_banish);
    e2.set_target(duel, sptg);
    e2.set_operation(duel, spop);
    c.register_effect(duel, e2);
}

fn tgfilter(c: CardId, duel: &Duel) -> bool {
    c.is_race(duel, RACE_ILLUSION) && c.is_able_to_grave(duel)
}

fn tgtg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::exists_target(duel, a, chk, tgfilter, LOCATION_DECK, CATEGORY_TOGRAVE)
}

fn tgop(duel: &mut Duel, a: &Args) -> Script<()> {
    stock::send_to_grave(duel, a, tgfilter, LOCATION_DECK)
}

fn spconfilter(c: CardId, duel: &Duel) -> bool {
    c.is_level_above(duel, 5) && c.is_race(duel, RACE_ILLUSION) && c.is_faceup(duel)
}

fn spcon(duel: &Duel, a: &Args) -> bool {
    duel.is_existing_matching_card(spconfilter, a.tp, LOCATION_MZONE, 0, 1, None)
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_controler(duel, 1 - tp) && chkc.is_location(duel, LOCATION_GRAVE) && spfilter(chkc, duel, e, tp));
    }
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0
            && duel.is_existing_target(|c, duel| spfilter(c, duel, e, tp), tp, 0, LOCATION_GRAVE, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_target(tp, |c, duel| spfilter(c, duel, e, tp), tp, 0, LOCATION_GRAVE, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, g, 1, tp, 0);
    duel.set_possible_operation_info(0, CATEGORY_REMOVE, None, 1, tp, LOCATION_MZONE as i32);
    Ok(false)
}

struct Banish;

impl aux::DelayedOperation for Banish {
    fn operate(duel: &mut Duel, ag: CardSet, _: &Args) -> Script<()> {
        duel.remove(ag, POS_FACEUP, REASON_EFFECT)?;
        Ok(())
    }

    fn condition(duel: &Duel, _: CardSet, a: &Args) -> bool {
        i64::from(duel.get_turn_count()) == duel.get_script_state(turn_summoned_key(a.e.get_label(duel))) + 1
    }
}

// The turn the monster was summoned in, per delayed effect (its first label is the fid).
fn turn_summoned_key(fid: i64) -> u64 {
    u64::from(ID) << 32 | fid as u64
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if let Some(tc) = duel.get_first_target()
        && tc.is_relate_to_effect(duel, e)
        && duel.special_summon(tc, 0, tp, tp, false, false, POS_FACEUP)? > 0
    {
        let turn = i64::from(duel.get_turn_count());
        if let Some(delayed) = aux::delayed_operation_full::<Banish>(
            duel,
            tc.into(),
            PHASE_END,
            RESET_PHASE | PHASE_END,
            ID,
            e,
            tp,
            aux::stringid(ID, 2),
            2,
        ) {
            duel.set_script_state(turn_summoned_key(delayed.get_label(duel)), turn);
        }
    }
    Ok(())
}
