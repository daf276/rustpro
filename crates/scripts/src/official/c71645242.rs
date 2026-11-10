// Black Garden
use crate::prelude::*;
use crate::testing::{Check, GRAVE, HAND, MZONE};

const ID: u32 = 71645242;
const SUMMONED_BY_BLACK_GARDEN: u32 = 0x20;

pub const LISTED_NAMES: &[u32] = &[ID, TOKEN_ROSE];
// In the hand, with Dark Plant (a Plant of ATK 300) in the hand to summon and on the field and in the Graveyard to revive.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, HAND),
    (ID, HAND),
    (13193642, MZONE),
    (13193642, HAND),
    (13193642, HAND),
    (13193642, GRAVE),
];
pub const CHECKS: &[Check] = &[
    Check::ResolvesEffect(aux::stringid(ID, 0)),
    Check::ResolvesEffect(aux::stringid(ID, 1)),
    Check::SpecialSummonedCode(TOKEN_ROSE),
];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    let e0 = duel.create_effect(c);
    e0.set_type(duel, EFFECT_TYPE_ACTIVATE);
    e0.set_code(duel, EVENT_FREE_CHAIN);
    c.register_effect(duel, e0);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_ATKCHANGE + CATEGORY_SPECIAL_SUMMON + CATEGORY_TOKEN);
    e1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_TRIGGER_F);
    e1.set_range(duel, LOCATION_FZONE);
    e1.set_code(duel, EVENT_CUSTOM + ID);
    e1.set_target(duel, atksptg);
    e1.set_operation(duel, atkspop);
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_category(duel, CATEGORY_DESTROY + CATEGORY_SPECIAL_SUMMON);
    e2.set_property(duel, EFFECT_FLAG_CARD_TARGET);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_FZONE);
    e2.set_target(duel, dessptg);
    e2.set_operation(duel, desspop);
    c.register_effect(duel, e2);
    aux::global_check(duel, ID, |duel| {
        let ge1 = duel.create_effect(c);
        ge1.set_type(duel, EFFECT_TYPE_FIELD + EFFECT_TYPE_CONTINUOUS);
        ge1.set_code(duel, EVENT_SUMMON_SUCCESS);
        ge1.set_condition_mut(duel, regcon);
        ge1.set_operation(duel, regop);
        duel.register_effect(ge1, 0);
        let ge2 = ge1.clone_effect(duel);
        ge2.set_code(duel, EVENT_SPSUMMON_SUCCESS);
        duel.register_effect(ge2, 0);
    });
}

fn cfilter(c: CardId, duel: &Duel, tp: u8) -> bool {
    c.is_controler(duel, tp) && c.get_summon_type(duel) != SUMMON_TYPE_SPECIAL + SUMMONED_BY_BLACK_GARDEN
}

fn regcon(duel: &mut Duel, a: &Args) -> bool {
    let eg = a.eg.unwrap_or_default();
    let mut sf = 0;
    if eg.is_exists(duel, |c, duel| cfilter(c, duel, 0), 1, None) {
        sf += 1;
    }
    if eg.is_exists(duel, |c, duel| cfilter(c, duel, 1), 1, None) {
        sf += 2;
    }
    a.e.set_label(duel, sf);
    sf != 0
}

fn regop(duel: &mut Duel, a: &Args) -> Script<()> {
    let label = a.e.get_label(duel) as u32;
    duel.raise_event(a.eg.unwrap_or_default(), EVENT_CUSTOM + ID, Some(a.e), a.r, a.rp, a.ep, label)
}

fn atksptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if chk == 0 {
        return Ok(e.get_handler(duel).is_relate_to_effect(duel, e));
    }
    duel.set_target_card(a.eg.unwrap_or_default());
    duel.set_operation_info(0, CATEGORY_TOKEN, None, 1, 0, 0);
    let (mine, theirs) = ((a.ev >> tp) & 1 != 0, (a.ev >> (1 - tp)) & 1 != 0);
    let p = if mine && theirs {
        PLAYER_ALL
    } else if mine {
        tp
    } else {
        1 - tp
    };
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, p, 0);
    Ok(false)
}

fn atkfilter(c: CardId, duel: &Duel, e: EffectId) -> bool {
    c.is_location(duel, LOCATION_MZONE) && c.is_faceup(duel) && c.is_relate_to_effect(duel, e) && !c.is_immune_to_effect(duel, e)
}

fn summon_token(duel: &mut Duel, p: u8) -> Script<()> {
    let stats = MonsterStats::new(SET_ROSE, TYPES_TOKEN, 800, 800, 2, RACE_PLANT, ATTRIBUTE_DARK);
    if duel.get_location_count(1 - p, LOCATION_MZONE) > 0
        && duel.is_player_can_special_summon_monster_full(p, TOKEN_ROSE, stats, POS_FACEUP_ATTACK, 1 - p, 0)
        && let Some(token) = duel.create_token(p, TOKEN_ROSE)
    {
        duel.special_summon_step(token, SUMMONED_BY_BLACK_GARDEN, p, 1 - p, false, false, POS_FACEUP_ATTACK)?;
    }
    Ok(())
}

fn atkspop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp, ev) = (a.e, a.tp, a.ev);
    let g = a.eg.unwrap_or_default().filter(duel, |c, duel| atkfilter(c, duel, e), None);
    if g.is_empty() {
        return Ok(());
    }
    let mut change = false;
    for tc in g {
        let preatk = tc.get_attack(duel);
        let e1 = duel.create_effect(e.get_handler(duel));
        e1.set_type(duel, EFFECT_TYPE_SINGLE);
        e1.set_code(duel, EFFECT_SET_ATTACK_FINAL);
        e1.set_value(duel, (tc.get_attack(duel) + 1) / 2);
        e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
        tc.register_effect(duel, e1);
        if !tc.is_immune_to_effect(duel, e1) && (preatk + 1) / 2 == tc.get_attack(duel) {
            change = true;
        }
    }
    if !change {
        return Ok(());
    }
    if (ev >> tp) & 1 != 0 {
        summon_token(duel, tp)?;
    }
    if (ev >> (1 - tp)) & 1 != 0 {
        summon_token(duel, 1 - tp)?;
    }
    duel.special_summon_complete()?;
    Ok(())
}

fn spfilter(c: CardId, duel: &Duel, atk: i32, e: EffectId, tp: u8) -> bool {
    c.is_attack(duel, &[atk]) && c.is_can_be_special_summoned(duel, e, SUMMONED_BY_BLACK_GARDEN, tp, false, false)
}

fn plantfilter(c: CardId, duel: &Duel) -> bool {
    c.is_faceup(duel) && c.is_race(duel, RACE_PLANT)
}

fn dessptg(duel: &mut Duel, a: &Args, chk: i32, chkc: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    if let Some(chkc) = chkc {
        return Ok(chkc.is_location(duel, LOCATION_GRAVE)
            && chkc.is_controler(duel, tp)
            && spfilter(chkc, duel, e.get_label(duel) as i32, e, tp));
    }
    let mut g = duel.get_matching_group(plantfilter, tp, LOCATION_MZONE, LOCATION_MZONE, None);
    let atk = g.get_sum(duel, |c, duel| i64::from(c.get_attack(duel))) as i32;
    if chk == 0 {
        return Ok(!g.is_empty()
            && duel.get_mzone_count(tp, Some(g)) > 0
            && duel.is_existing_target(|c, duel| spfilter(c, duel, atk, e, tp), tp, LOCATION_GRAVE, 0, 1, None));
    }
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let tg = duel.select_target(tp, |c, duel| spfilter(c, duel, atk, e, tp), tp, LOCATION_GRAVE, 0, 1, 1, None)?;
    e.set_label(duel, i64::from(atk));
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, tg, 1, 0, 0);
    g.insert(e.get_handler(duel));
    duel.set_operation_info(0, CATEGORY_DESTROY, Some(g), g.len() as u8, 0, 0);
    Ok(false)
}

fn desspop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    if !c.is_relate_to_effect(duel, e) {
        return Ok(());
    }
    let mut dg = duel.get_matching_group(plantfilter, tp, LOCATION_MZONE, LOCATION_MZONE, None);
    dg.insert(c);
    duel.destroy(dg, REASON_EFFECT)?;
    let og = duel.get_operated_group();
    if dg.len() != og.len() || og.into_iter().filter(|&oc| dg.contains(oc)).count() != og.len() {
        return Ok(());
    }
    duel.break_effect()?;
    if let Some(tc) = duel.get_first_target()
        && tc.is_relate_to_effect(duel, e)
    {
        duel.special_summon(tc, SUMMONED_BY_BLACK_GARDEN, tp, tp, false, false, POS_FACEUP)?;
    }
    Ok(())
}
