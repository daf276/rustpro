// Kashtira Birth
use crate::prelude::*;
use crate::testing::{Check, GRAVE, HAND, MZONE, OPPONENT, SZONE};

const ID: u32 = 69540484;

// Kashtira Fenrir and Unicorn (Level 7) in the hand, on the field and in the Graveyard, with Raigeki and Sangan on the
// opponent's side to chain to and to banish.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, SZONE),
    (32909498, HAND),
    (68304193, MZONE),
    (32909498, GRAVE),
    (12580477, HAND | OPPONENT),
    (26202165, MZONE | OPPONENT),
    (26202165, GRAVE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::ResolvesEffect(aux::stringid(ID, 2)),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e1.set_code(duel, EVENT_FREE_CHAIN);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_type(duel, EFFECT_TYPE_FIELD);
    e2.set_code(duel, EFFECT_SUMMON_PROC);
    e2.set_range(duel, LOCATION_SZONE);
    e2.set_target_range(duel, LOCATION_HAND, 0);
    e2.set_count_limit(duel, 1, (ID, 0));
    e2.set_condition(duel, ntcon);
    e2.set_target_card(duel, |duel, _, c| c.is_level(duel, &[7]));
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e3.set_type(duel, EFFECT_TYPE_IGNITION);
    e3.set_range(duel, LOCATION_SZONE);
    e3.set_count_limit(duel, 1, (ID, 1));
    e3.set_target(duel, sptg);
    e3.set_operation(duel, spop);
    c.register_effect(duel, e3);
    let e4 = duel.create_effect(c);
    e4.set_description(duel, aux::stringid(ID, 2));
    e4.set_category(duel, CATEGORY_REMOVE);
    e4.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e4.set_property(duel, EFFECT_FLAG_CARD_TARGET + EFFECT_FLAG_DELAY);
    e4.set_code(duel, EVENT_CHAIN_SOLVED);
    e4.set_range(duel, LOCATION_SZONE);
    e4.set_count_limit(duel, 1, (ID, 2));
    e4.set_condition(duel, |duel, a| a.rp == 1 - a.tp && a.re.is_some_and(|re| re.is_spell_effect(duel)));
    e4.set_target(duel, rmtg);
    e4.set_operation(duel, rmop);
    c.register_effect(duel, e4);
}

fn ntcon(duel: &Duel, a: &Args) -> bool {
    let Some(c) = a.c else { return true };
    a.min == 0 && duel.get_location_count(c.get_controler(duel), LOCATION_MZONE) > 0
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_faceup(duel)
        && c.is_set_card(duel, &[SET_KASHTIRA])
        && !c.is_type(duel, TYPE_XYZ)
        && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    stock::special_summon_target(
        duel,
        a,
        chk,
        |c, duel| spfilter(c, duel, a.e, a.tp),
        LOCATION_GRAVE | LOCATION_REMOVED,
    )
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if duel.get_location_count(tp, LOCATION_MZONE) <= 0 {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let filter = aux::necro_valley_filter(|c, duel| spfilter(c, duel, e, tp));
    let g = duel.select_matching_card(tp, filter, tp, LOCATION_GRAVE | LOCATION_REMOVED, 0, 1, 1, None)?;
    if !g.is_empty() {
        duel.special_summon(g, 0, tp, tp, false, false, POS_FACEUP)?;
    }
    Ok(())
}

fn rmfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_able_to_remove_full(duel, Some(tp), POS_FACEDOWN, REASON_EFFECT) && aux::sp_elim_filter(c, duel, false, false)
}

fn rmtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE | LOCATION_GRAVE) && chkc.is_controler(duel, 1 - tp) && rmfilter(chkc, duel, tp));
    }
    if chk == 0 {
        return Ok(duel.is_existing_matching_card(
            aux::faceup_filter(|c, duel| c.is_set_card(duel, &[SET_KASHTIRA])),
            tp,
            LOCATION_MZONE,
            0,
            1,
            None,
        ) && duel.is_existing_target(|c, duel| rmfilter(c, duel, tp), tp, 0, LOCATION_MZONE | LOCATION_GRAVE, 3, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_REMOVE);
    let g = duel.select_target(
        tp,
        |c, duel| rmfilter(c, duel, tp),
        tp,
        0,
        LOCATION_MZONE | LOCATION_GRAVE,
        3,
        3,
        None,
    )?;
    duel.set_operation_info(0, CATEGORY_REMOVE, g, g.map_or(0, |g| g.len()) as u8, 0, 0);
    Ok(false)
}

fn rmop(duel: &mut Duel, a: &Args) -> Script<()> {
    let g = duel.get_target_cards(a.e).unwrap_or_default();
    if !g.is_empty() {
        duel.remove(g, POS_FACEDOWN, REASON_EFFECT)?;
    }
    Ok(())
}
