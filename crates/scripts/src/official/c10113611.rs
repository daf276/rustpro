// Wattuna
use crate::prelude::*;
use crate::testing::{Check, EXTRA, HAND, MZONE};

const ID: u32 = 10113611;

pub const LISTED_SERIES: &[u16] = &[SET_WATT];
// Copies on the field to attack directly and in the hand, with Thunder Kid (Level 2) in the hand to release with it for Wattchimera
// (Level 6) in the Extra Deck.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, MZONE),
    (ID, HAND),
    (ID, HAND),
    (15510988, HAND),
    (15510988, HAND),
    (2772236, EXTRA),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::SpecialSummonedCode(2772236),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_DIRECT_ATTACK);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_O);
    e2.set_code(duel, EVENT_DAMAGE_STEP_END);
    e2.set_range(duel, LOCATION_HAND);
    e2.set_count_limit(duel, 1, ID);
    e2.set_condition(duel, |duel, a| duel.get_flag_effect(a.tp, ID) > 0);
    e2.set_target(duel, stock::special_summon_self_target);
    e2.set_operation(duel, stock::special_summon_self::<POS_FACEUP>);
    c.register_effect(duel, e2);
    let e3 = duel.create_effect(c);
    e3.set_description(duel, aux::stringid(ID, 1));
    e3.set_category(duel, CATEGORY_RELEASE + CATEGORY_SPECIAL_SUMMON);
    e3.set_type(duel, EFFECT_TYPE_SINGLE + EFFECT_TYPE_TRIGGER_O);
    e3.set_code(duel, EVENT_BATTLE_DAMAGE);
    e3.set_count_limit(duel, 1, (ID, 1));
    e3.set_condition(duel, spsynccon);
    e3.set_target(duel, spsynctg);
    e3.set_operation(duel, spsyncop);
    c.register_effect(duel, e3);
    aux::global_check(duel, ID, |duel| {
        let ge1 = duel.create_effect(c);
        ge1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
        ge1.set_code(duel, EVENT_BATTLE_DAMAGE);
        ge1.set_operation(duel, checkop);
        duel.register_effect(ge1, 0);
    });
}

fn checkop(duel: &mut Duel, a: &Args) -> Script<()> {
    let player = a.eg.and_then(|eg| eg.get_first()).map_or(0, |tc| tc.get_controler(duel));
    if a.ep == 1 - player {
        duel.register_flag_effect(player, ID, RESET_PHASE | PHASE_DAMAGE, 0, 1);
    }
    Ok(())
}

fn spsynccon(duel: &Duel, a: &Args) -> bool {
    a.ep == 1 - a.tp && duel.get_attack_target().is_none()
}

fn relfilter(c: CardId, duel: &Duel) -> bool {
    c.has_level(duel)
        && !c.is_type(duel, TYPE_TUNER)
        && c.is_releasable_by_effect(duel)
        && (c.is_location(duel, LOCATION_HAND) || c.is_faceup(duel))
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8, matg: Option<CardSet>, lv: i64) -> bool {
    c.is_set_card(duel, &[SET_WATT])
        && c.is_type(duel, TYPE_SYNCHRO)
        && c.is_level(duel, &[lv as u32])
        && duel.get_location_count_from_ex(tp, tp, matg, c) > 0
        && c.is_can_be_special_summoned(duel, e, 0, tp, false, false)
}

fn rescon(duel: &Duel, sg: CardSet, e: EffectId, tp: u8) -> (bool, bool) {
    let lv = sg.get_sum(duel, |c, duel| i64::from(c.get_level(duel)));
    (
        sg.is_contains(e.get_handler(duel))
            && duel.is_existing_matching_card(|c, duel| spfilter(c, duel, e, tp, Some(sg), lv), tp, LOCATION_EXTRA, 0, 1, None),
        false,
    )
}

fn spsynctg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let mut g = duel.get_matching_group(relfilter, tp, LOCATION_HAND | LOCATION_MZONE, 0, None);
    g.merge(c.into());
    if chk == 0 {
        return Ok(c.has_level(duel)
            && g.len() >= 2
            && aux::select_unselect_group_check(duel, g, 2, g.len(), &|duel, sg, _, _| rescon(duel, sg, e, tp)));
    }
    duel.set_operation_info(0, CATEGORY_RELEASE, Some(c.into()), 1, tp, (LOCATION_HAND | LOCATION_MZONE) as i32);
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, LOCATION_EXTRA as i32);
    Ok(false)
}

fn spsyncop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if !c.is_relate_to_effect(duel, e) {
        return Ok(());
    }
    let mut g = duel.get_matching_group(relfilter, tp, LOCATION_HAND | LOCATION_MZONE, 0, None);
    g.merge(c.into());
    if g.len() < 2 {
        return Ok(());
    }
    let rg = aux::select_unselect_group(duel, g, 2, g.len(), &|duel, sg, _, _| rescon(duel, sg, e, tp), tp, HINTMSG_RELEASE)?;
    if rg.len() < 2 {
        return Ok(());
    }
    let lv = rg.get_sum(duel, |c, duel| i64::from(c.get_level(duel)));
    if duel.release(rg, REASON_EFFECT)? > 0 {
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
        let sc = duel
            .select_matching_card(tp, |c, duel| spfilter(c, duel, e, tp, None, lv), tp, LOCATION_EXTRA, 0, 1, 1, None)?
            .get_first();
        if let Some(sc) = sc {
            duel.special_summon(sc, 0, tp, tp, false, false, POS_FACEUP)?;
        }
    }
    Ok(())
}
