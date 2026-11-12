// Ursarctic Polar Star
use super::c27693363::{sprcon, sprop, sprtg};
use crate::prelude::*;
use crate::testing::{Check, EXTRA, MZONE};

const ID: u32 = 62714453;

// On the field with Ursarctic Megabilis (Level 8) to Tribute and Ursarctic Septentrion and Grand Chariot (Level 7) in the Extra Deck;
// also in the Extra Deck, Special Summoned by sending Gusto Egul (a Level 1 Tuner) and Gusto Griffin (Level 2) to the Graveyard.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (ID, EXTRA),
    (81108658, MZONE),
    (53087962, EXTRA),
    (80086070, EXTRA),
    (91662792, MZONE),
    (42082363, MZONE),
    (42082363, MZONE),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0)), Check::SpecialSummoned];

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    c.enable_revive_limit(duel);
    c.add_must_be_special_summoned(duel);
    let e0 = duel.create_effect(c);
    e0.set_type(duel, EFFECT_TYPE_FIELD);
    e0.set_property(duel, EFFECT_FLAG_CANNOT_DISABLE + EFFECT_FLAG_UNCOPYABLE);
    e0.set_code(duel, EFFECT_SPSUMMON_PROC);
    e0.set_range(duel, LOCATION_EXTRA);
    e0.set_condition(duel, sprcon);
    e0.set_target(duel, sprtg);
    e0.set_operation(duel, sprop);
    c.register_effect(duel, e0);
    let e1 = duel.create_effect(c);
    e1.set_description(duel, aux::stringid(ID, 0));
    e1.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e1.set_type(duel, EFFECT_TYPE_IGNITION);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_cost(duel, |duel, a, chk| Cost::replaceable_full(duel, a, chk, spcost, extracon));
    e1.set_target(duel, stock::operation_info_target::<CATEGORY_SPECIAL_SUMMON, LOCATION_EXTRA>);
    e1.set_operation(duel, spop);
    c.register_effect(duel, e1);
}

fn spcheck(duel: &Duel, sg: CardSet, tp: u8, e: EffectId) -> (bool, bool) {
    let c = e.get_handler(duel);
    let res = sg.is_exists(duel, ursfilter, 1, Some(c.into()))
        && sg.is_contains(c)
        && duel.is_existing_matching_card(
            |sc, duel| spfilter(sc, duel, e, tp, Some(sg.union(c.into()))),
            tp,
            LOCATION_EXTRA,
            0,
            1,
            None,
        );
    (res, false)
}

fn ursfilter(c: CardId, duel: &Duel) -> bool {
    c.is_set_card(duel, &[SET_URSARCTIC]) && c.is_level(duel, &[8])
}

fn spfilter(c: CardId, duel: &Duel, e: EffectId, tp: u8, mustg: Option<CardSet>) -> bool {
    c.is_level(duel, &[7])
        && c.is_set_card(duel, &[SET_URSARCTIC])
        && c.is_type(duel, TYPE_SYNCHRO)
        && c.is_can_be_special_summoned(duel, e, 0, tp, true, false)
        && duel.get_location_count_from_ex(tp, tp, mustg, c) > 0
}

fn spcost(duel: &mut Duel, a: &Args, chk: i32) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let check = |duel: &Duel, sg: CardSet, tp: u8, _: CardSet| spcheck(duel, sg, tp, e);
    if chk == 0 {
        return Ok(duel.check_release_group_cost(tp, aux::TRUE, (2, 2), true, Some(&check), None));
    }
    let sg = duel.select_release_group_cost(tp, aux::TRUE, (2, 2), true, Some(&check), None)?;
    duel.release(sg, REASON_COST)?;
    Ok(false)
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
    let g = duel.select_matching_card(tp, |c, duel| spfilter(c, duel, e, tp, None), tp, LOCATION_EXTRA, 0, 1, 1, None)?;
    let Some(sc) = g.get_first() else { return Ok(()) };
    if duel.special_summon(sc, 0, tp, tp, true, false, POS_FACEUP)? > 0 {
        let e1 = duel.create_effect(sc);
        e1.set_type(duel, EFFECT_TYPE_FIELD);
        e1.set_property(duel, EFFECT_FLAG_PLAYER_TARGET);
        e1.set_code(duel, EFFECT_CANNOT_ACTIVATE);
        e1.set_range(duel, LOCATION_MZONE);
        e1.set_target_range(duel, 0, 1);
        e1.set_value(duel, Value::EffectPlayer(aclimit));
        e1.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
        sc.register_effect_full(duel, e1, true);
        if !sc.is_type(duel, TYPE_EFFECT) {
            let e2 = duel.create_effect(e.get_handler(duel));
            e2.set_type(duel, EFFECT_TYPE_SINGLE);
            e2.set_code(duel, EFFECT_ADD_TYPE);
            e2.set_value(duel, TYPE_EFFECT as i32);
            e2.set_reset(duel, RESET_EVENT | RESETS_STANDARD);
            sc.register_effect_full(duel, e2, true);
        }
        sc.register_flag_effect_full(
            duel,
            ID,
            RESET_EVENT | RESETS_STANDARD,
            EFFECT_FLAG_CLIENT_HINT,
            1,
            0,
            aux::stringid(ID, 1),
        );
    }
    Ok(())
}

fn aclimit(duel: &Duel, _: EffectId, re: EffectId, _: u8) -> bool {
    let rc = re.get_handler(duel);
    re.is_monster_effect(duel) && rc.has_level(duel) && rc.is_summon_location(duel, LOCATION_EXTRA) && rc.is_location(duel, LOCATION_MZONE)
}

fn extracon(duel: &Duel, _: EffectId, a: &Args) -> bool {
    let (e, tp) = (a.e, a.tp);
    duel.is_existing_matching_card(|c, duel| spfilter(c, duel, e, tp, None), tp, LOCATION_EXTRA, 0, 1, None)
}
