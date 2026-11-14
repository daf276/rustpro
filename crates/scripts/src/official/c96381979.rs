// Brotherhood of the Fire Fist - Tiger King
use crate::prelude::*;
use crate::testing::{Check, DECK, EXTRA, HAND, MZONE, OPPONENT, SZONE};

const ID: u32 = 96381979;

pub const LISTED_SERIES: &[u16] = &[SET_FIRE_FORMATION];
// In the Extra Deck, Xyz Summoned from Vorse Raider (Level 4 Beast-Warrior Normal Monster, also two in the Deck to Special Summon with the same ATK),
// with Fire Formation - Tenki to cost and Set, and the opponent's Raigeki, Ash Blossom and Elfnote Lucina as targets.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, EXTRA),
    (ID, EXTRA),
    (14898066, MZONE),
    (14898066, MZONE),
    (14898066, MZONE),
    (14898066, DECK),
    (14898066, DECK),
    (14898066, DECK),
    (57103969, SZONE),
    (57103969, SZONE),
    (57103969, SZONE),
    (57103969, SZONE),
    (57103969, DECK),
    (57103969, DECK),
    (12580477, HAND | OPPONENT),
    (12580477, HAND | OPPONENT),
    (14558127, MZONE | OPPONENT),
    (13597785, MZONE | OPPONENT),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::ResolvesEffect(aux::stringid(ID, 2)),
];

struct Procedure;

impl XyzProcedure for Procedure {
    const F: Option<XyzFilter> = Some(|c, duel, scard, sumtype, tp| c.is_race_full(duel, RACE_BEASTWARRIOR, Some(scard), sumtype, tp));
    const LV: Option<u32> = Some(4);
    const CT: usize = 2;
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Xyz::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SET);
    e1.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e1.set_code(duel, EVENT_SPSUMMON_SUCCESS);
    e1.set_condition(duel, |duel, a| a.e.get_handler(duel).is_xyz_summoned(duel));
    e1.set_target(duel, settg);
    e1.set_operation(duel, setop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_DISABLE);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, 0);
    e2.set_cost(duel, Cost::detach_from_self::<1, 1>);
    e2.set_target(duel, distg);
    e2.set_operation(duel, disop);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 2));
    e3.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e3.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e3.set_property(duel, EFFECT_FLAG_DAMAGE_STEP);
    e3.set_code(duel, EVENT_TO_GRAVE);
    e3.set_condition(duel, |duel, a| a.e.get_handler(duel).is_previous_location(duel, LOCATION_ONFIELD));
    e3.set_cost(duel, |duel, a, chk| {
        Cost::replaceable_full(duel, a, chk, spcost, |duel, _, a| duel.get_location_count(a.tp, LOCATION_MZONE) > 1)
    });
    e3.set_target(duel, sptg);
    e3.set_operation(duel, spop);
    c.register_effect(duel, e3);
}

fn filter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_FIRE_FORMATION]) && c.is_spell_trap(duel) && c.is_ssetable(duel)
}

fn settg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    Ok(chk == 0 && duel.is_existing_matching_card(filter, a.tp, LOCATION_DECK, 0, 1, None))
}

fn setop(duel: &mut Duel, a: &Args) -> Script<()> {
    let tp = a.tp;
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SET);
    let g = duel.select_matching_card(tp, filter, tp, LOCATION_DECK, 0, 1, 1, None)?;
    if let Some(tc) = g.get_first() {
        duel.sset(tp, tc)?;
    }
    Ok(())
}

fn dfilter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_type(duel, TYPE_EFFECT) && !c.is_race(duel, RACE_BEASTWARRIOR)
}

fn distg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    Ok(chk == 0 && duel.is_existing_matching_card(dfilter, a.tp, LOCATION_MZONE, LOCATION_MZONE, 1, None))
}

fn disop(duel: &mut Duel, a: &Args) -> Script<()> {
    let c = a.e.get_handler(duel);
    for tc in duel.get_matching_group(dfilter, a.tp, LOCATION_MZONE, LOCATION_MZONE, None) {
        for code in [EFFECT_DISABLE, EFFECT_DISABLE_EFFECT] {
            let e = duel.create_effect(c);
            e.set_type(duel, EFFECT_TYPE_SINGLE);
            e.set_code(duel, code);
            e.set_reset_full(duel, RESETS_STANDARD_PHASE_END, 2);
            tc.register_effect(duel, e);
        }
    }
    Ok(())
}

fn spcostfilter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_set_card(duel, &[SET_FIRE_FORMATION]) && c.is_spell_trap(duel) && c.is_able_to_grave_as_cost(duel)
}

fn spcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let tp = a.tp;
    let g = duel.get_matching_group(spcostfilter, tp, LOCATION_ONFIELD, 0, None);
    if chk == 0 {
        return Ok(aux::select_unselect_group_check(duel, g, 3, 3, &aux::chkf_mmz(tp, 2)));
    }
    let sg = aux::select_unselect_group(duel, g, 3, 3, &aux::chkf_mmz(tp, 2), tp, HINTMSG_TOGRAVE)?;
    duel.send_to_grave(sg, REASON_COST)?;
    Ok(false)
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    c.is_level_below(duel, 4)
        && c.is_race(duel, RACE_BEASTWARRIOR)
        && c.is_can_be_special_summoned_full(duel, e, 0, tp, false, false, POS_FACEUP_DEFENSE, tp, 0xff)
}

fn afilter1(c: CardId, duel: &Duel, g: CardSet) -> bool {
    let atk = c.get_attack(duel);
    g.is_exists(duel, |c, duel| afilter2(c, duel, atk), 1, Some(c.into()))
}

fn afilter2(c: CardId, duel: &Duel, atk: i32) -> bool {
    c.get_attack(duel) == atk
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        let g = duel.get_matching_group(|c, duel| spfilter(c, duel, e, tp), tp, LOCATION_DECK, 0, None);
        return Ok(duel.is_player_affected_by_effect(tp, CARD_BLUEEYES_SPIRIT).is_none()
            && g.is_exists(duel, |c, duel| afilter1(c, duel, g), 1, None));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 2, tp, LOCATION_DECK as i32);
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    if duel.is_player_affected_by_effect(tp, CARD_BLUEEYES_SPIRIT).is_some() || duel.get_location_count(tp, LOCATION_MZONE) < 2 {
        return Ok(());
    }
    let g = duel.get_matching_group(|c, duel| spfilter(c, duel, e, tp), tp, LOCATION_DECK, 0, None);
    let dg = g.filter(duel, |c, duel| afilter1(c, duel, g), None);
    if dg.is_empty() {
        return Ok(());
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let tc1 = dg.select(duel, tp, 1, 1, None)?.get_first();
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let atk = tc1.map_or(0, |tc| tc.get_attack(duel));
    let tc2 = dg
        .filter_select(duel, tp, |c, duel| afilter2(c, duel, atk), 1, 1, tc1.map(Into::into))?
        .get_first();
    for tc in [tc1, tc2].into_iter().flatten() {
        duel.special_summon_step(tc, 0, tp, tp, false, false, POS_FACEUP_DEFENSE)?;
    }
    duel.special_summon_complete()?;
    Ok(())
}
