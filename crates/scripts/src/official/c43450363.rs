// R.B. Last Stand
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, GRAVE, HAND, MZONE, OPPONENT, SZONE};

const ID: u32 = 43450363;

pub const LISTED_SERIES: &[u16] = &[SET_RB];
// R.B. monsters to summon, one face-up to protect, a copy in the Graveyard, and Mind Control for the opponent to target it.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, SZONE),
    (ID, GRAVE),
    (81794107, DECK),
    (81794107, GRAVE),
    (32216688, EXTRA),
    (6821579, MZONE),
    (80071619, MZONE),
    (37520316, HAND | OPPONENT),
    (37520316, HAND | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    e1.set_count_limit(duel, 1, ID);
    e1.set_target(duel, sptg);
    e1.set_operation(duel, spop);
    e1.set_hint_timing(duel, 0, TIMING_STANDBY_PHASE | TIMING_MAIN_END | TIMINGS_CHECK_MONSTER_E);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_DISABLE);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_code(duel, EVENT_CHAINING);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_count_limit(duel, 1, ID);
    e2.set_condition(duel, discon);
    e2.set_cost(duel, Cost::self_banish);
    e2.set_target(duel, distg);
    e2.set_operation(duel, |duel, a| {
        duel.negate_effect(a.ev as u8);
        Ok(())
    });
    c.register_effect(duel, e2);
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    let code = c.get_code(duel);
    if !(c.is_set_card(duel, &[SET_RB])
        && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
        && !duel.is_existing_matching_card(
            |tc, duel| tc.is_faceup(duel) && tc.is_code(duel, code),
            tp,
            LOCATION_ONFIELD,
            0,
            1,
            None,
        ))
    {
        return false;
    }
    if c.is_location(duel, LOCATION_DECK | LOCATION_GRAVE) {
        duel.get_mzone_count(tp, None) > 0
    } else {
        c.is_location(duel, LOCATION_EXTRA) && duel.get_location_count_from_ex(tp, tp, None, c) > 0
    }
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let location = LOCATION_DECK | LOCATION_EXTRA | LOCATION_GRAVE;
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(|c, duel| spfilter(c, duel, e, tp), tp, location, 0, 1, None));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, location as i32);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let filter = aux::necro_valley_filter(|c, duel| spfilter(c, duel, e, tp));
    let g = duel.select_matching_card(tp, filter, tp, LOCATION_DECK | LOCATION_EXTRA | LOCATION_GRAVE, 0, 1, 1, None)?;
    if !g.is_empty() {
        duel.special_summon(g, 0, tp, tp, false, false, POS_FACEUP)?;
    }
    if !e.is_has_type(duel, EFFECT_TYPE_ACTIVATE) {
        return Ok(());
    }
    let e1 = duel.create_effect(e.get_handler(duel));
    e1.set_description(duel, aux::stringid(ID, 2));
    e1.set_type(duel, EFFECT_TYPE_FIELD);
    e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_CLIENT_HINT);
    e1.set_code(duel, EFFECT_CANNOT_SPECIAL_SUMMON);
    e1.set_target_range(duel, 1, 0);
    e1.set_target_card(duel, |duel, _, c| {
        c.is_location(duel, LOCATION_EXTRA) && !(c.is_race(duel, RACE_MACHINE) && c.is_attack_below(duel, 1500))
    });
    e1.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e1, tp);
    Ok(())
}

fn disconfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_set_card(duel, &[SET_RB]) && c.is_faceup(duel) && c.is_controler(duel, tp) && c.is_location(duel, LOCATION_MZONE)
}

fn discon(duel: &Duel, a: &Args) -> bool {
    let tp = a.tp;
    if !a.re.is_some_and(|re| re.is_has_property(duel, EFFECT_FLAG_CARD_TARGET)) || a.rp == tp {
        return false;
    }
    let tg: Option<CardSet> = duel.get_chain_info(a.ev as u8, CHAININFO_TARGET_CARDS);
    tg.is_some_and(|tg| tg.is_exists(duel, |c, duel| disconfilter(c, duel, tp), 1, None)) && duel.is_chain_disablable(a.ev as u8)
}

fn distg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    if chk == 0 {
        return Ok(true);
    }
    duel.set_operation_info(0, CATEGORY_DISABLE, a.eg, 1, a.tp, 0);
    Ok(false)
}
