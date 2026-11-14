// Etude of the Branded
use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE, SZONE};

const ID: u32 = 45675980;

pub const LISTED_SERIES: &[u16] = &[SET_BYSTIAL];
// On the field with The Bystial Aluber (a Level 4 Tuner) and Bystial Magnamhut (Level 6) as materials for Bystial Dis Pater.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, SZONE),
    (45005708, MZONE),
    (45005708, MZONE),
    (33854624, MZONE),
    (33854624, MZONE),
    (27572350, EXTRA),
    (27572350, EXTRA),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0)), Check::SpecialSummonedCode(27572350)];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_code(duel, EVENT_FREE_CHAIN);
    e2.set_range(duel, LOCATION_SZONE);
    e2.set_hint_timing(duel, 0, TIMINGS_CHECK_MONSTER_E);
    e2.set_count_limit(duel, 1, ID);
    e2.set_target(duel, sctg);
    e2.set_operation(duel, scop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_type(duel, EFFECT_TYPE_FIELD);
    e3.set_property(
        duel,
        EFFECT_FLAG_SET_AVAILABLE + EFFECT_FLAG_IGNORE_RANGE + EFFECT_FLAG_IGNORE_IMMUNE,
    );
    e3.set_code(duel, EFFECT_TO_GRAVE_REDIRECT);
    e3.set_range(duel, LOCATION_SZONE);
    e3.set_target_range(duel, 0, LOCATION_ALL);
    e3.set_condition(duel, |duel, a| {
        let filter = aux::faceup_filter(|c, duel| c.is_set_card(duel, &[SET_BYSTIAL]));
        duel.is_existing_matching_card(filter, a.e.get_handler_player(duel), LOCATION_MZONE, 0, 1, None)
    });
    e3.set_target_card(duel, rmtg);
    e3.set_value(duel, LOCATION_REMOVED as i32);
    c.register_effect(duel, e3);
}

fn checkaddition(duel: &Duel, _: u8, sg: CardSet, _: CardId) -> bool {
    sg.is_exists(duel, |c, duel| c.is_race(duel, RACE_DRAGON), 1, None)
}

fn synchro_summonable(c: CardId, duel: &Duel) -> bool {
    c.is_synchro_summonable(duel, None, None)
}

fn sctg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if chk == 0 {
        duel.set_synchro_check_additional(Some(checkaddition));
        let res = duel.is_existing_matching_card(synchro_summonable, tp, LOCATION_EXTRA, 0, 1, None);
        duel.set_synchro_check_additional(None);
        return Ok(res);
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_EXTRA as i32);
    Ok(false)
}

fn scop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.set_synchro_check_additional(Some(checkaddition));
    let g = duel.get_matching_group(synchro_summonable, tp, LOCATION_EXTRA, 0, None);
    if g.is_empty() {
        duel.set_synchro_check_additional(None);
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    if let Some(sc) = g.select(duel, tp, 1, 1, None)?.get_first() {
        duel.synchro_summon(tp, sc, None, None)?;
    }
    Ok(())
}

fn rmtg(duel: &Duel, e: EffectId, c: CardId) -> bool {
    let tp = e.get_handler_player(duel);
    c.get_owner(duel) == 1 - tp
        && duel.is_player_can_remove(tp, Some(c))
        && c.get_reason_player(duel) == 1 - tp
        && (c.get_reason(duel) & (REASON_RELEASE | REASON_RITUAL) == REASON_RELEASE | REASON_RITUAL
            || c.is_reason(duel, REASON_FUSION | REASON_SYNCHRO | REASON_LINK))
}
