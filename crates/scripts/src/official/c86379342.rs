// Tri-Brigade Oath
use crate::prelude::*;
use crate::testing::{Check, GRAVE, HAND, MZONE, SZONE};

const ID: u32 = 86379342;

// Set copies and one in the GY, Tri-Brigade Ferrijit, Rugal and Shuraig (a Beast, Beast-Warrior and Winged Beast Link) on the
// field, and Tri-Brigade Nervall and Fraktall in the GY and hand to Special Summon.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, SZONE),
    (ID, SZONE),
    (ID, GRAVE),
    (26847978, MZONE),
    (52331012, MZONE),
    (99726621, MZONE),
    (14816857, GRAVE),
    (87209160, HAND),
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
    e1.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e1.set_count_limit(duel, 1, ID);
    e1.set_target(duel, sptg);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_DISABLE);
    e2.set_type(duel, EFFECT_TYPE_QUICK_O);
    e2.set_range(duel, LOCATION_GRAVE);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_code(duel, EVENT_FREE_CHAIN);
    e2.set_count_limit(duel, 1, ID);
    e2.set_condition(duel, negcon);
    e2.set_cost(duel, Cost::self_banish);
    e2.set_target(duel, negtg);
    e2.set_operation(duel, negop);
    c.register_effect(duel, e2);
}

fn lkfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    let race = c.get_race(duel);
    c.is_type(duel, TYPE_LINK)
        && c.is_faceup(duel)
        && duel.is_existing_matching_card(
            |sc, duel| spfilter(sc, duel, e, tp, race),
            tp,
            LOCATION_HAND | LOCATION_GRAVE,
            0,
            1,
            None,
        )
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8, race: u64) -> bool {
    c.is_race(duel, RACES_BEAST_BWARRIOR_WINGB)
        && c.is_monster(duel)
        && !c.is_race(duel, race)
        && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let filter = |c, duel: &Duel| lkfilter(c, duel, e, tp);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_MZONE) && chkc.is_controler(duel, tp) && filter(chkc, duel));
    }
    if chk == 0 {
        return Ok(duel.get_location_count(tp, LOCATION_MZONE) > 0 && duel.is_existing_target(filter, tp, LOCATION_MZONE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_FACEUP);
    duel.select_target(tp, filter, tp, LOCATION_MZONE, 0, 1, 1, None)?;
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, (LOCATION_HAND | LOCATION_GRAVE) as i32);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if !(tc.is_faceup(duel) && tc.is_relate_to_effect(duel, e)) {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let race = tc.get_race(duel);
    let filter = aux::necro_valley_filter(|c, duel| spfilter(c, duel, e, tp, race));
    let g = duel.select_matching_card(tp, filter, tp, LOCATION_HAND | LOCATION_GRAVE, 0, 1, 1, None)?;
    if !g.is_empty() {
        duel.special_summon(g, 0, tp, tp, false, false, POS_FACEUP)?;
    }
    Ok(())
}

fn negcon(duel: &Duel, a: &Args) -> bool {
    [RACE_BEAST, RACE_BEASTWARRIOR, RACE_WINGEDBEAST].into_iter().all(|race| {
        duel.is_existing_matching_card(
            aux::faceup_filter(|c, duel| c.is_race(duel, race)),
            a.tp,
            LOCATION_MZONE,
            0,
            1,
            None,
        )
    })
}

fn negtg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let tp = a.tp;
    if let Some(chkc) = chkc {
        return Ok(chkc.is_on_field(duel) && chkc.is_controler(duel, 1 - tp) && chkc.is_negatable_spell_trap(duel));
    }
    if chk == 0 {
        return Ok(duel.is_existing_target(CardId::is_negatable_spell_trap, tp, 0, LOCATION_ONFIELD, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_NEGATE);
    duel.select_target(tp, CardId::is_negatable_spell_trap, tp, 0, LOCATION_ONFIELD, 1, 1, None)?;
    Ok(false)
}

fn negop(duel: &mut Duel, a: &Args) -> Script<()> {
    let Some(tc) = duel.get_first_target() else { return Ok(()) };
    if !(tc.is_relate_to_effect(duel, a.e) && tc.is_faceup(duel) && !tc.is_disabled(duel)) {
        return Ok(());
    }
    duel.negate_related_chain(tc, RESET_TURN_SET);
    let c = a.e.get_handler(duel);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_DISABLE);
    e1.set_reset(duel, RESETS_STANDARD_PHASE_END);
    tc.register_effect(duel, e1);
    let e2 = e1.clone_effect(duel);
    e2.set_code(duel, EFFECT_DISABLE_EFFECT);
    e2.set_value(duel, RESET_TURN_SET as i32);
    tc.register_effect(duel, e2);
    if tc.is_type(duel, TYPE_TRAPMONSTER) {
        let e3 = e1.clone_effect(duel);
        e3.set_code(duel, EFFECT_DISABLE_TRAPMONSTER);
        tc.register_effect(duel, e3);
    }
    Ok(())
}
