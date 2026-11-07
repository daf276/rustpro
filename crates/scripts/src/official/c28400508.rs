// Number 97: Draglubion
use crate::prelude::*;
use crate::testing::{Check, EXTRA, GRAVE, MZONE};

const ID: u32 = 28400508;
pub const XYZ_NUMBER: u32 = 97;

pub const LISTED_NAMES: &[u32] = &[ID];

// On the field with Number 62 and Number C62 in the Extra Deck and Graveyard as the "Number" Dragon monsters.
pub const SUPPORT: &[(u32, u16)] = &[
    (ID, MZONE),
    (31801517, EXTRA),
    (48348921, EXTRA),
    (31801517, GRAVE),
    (48348921, GRAVE),
];
pub const CHECKS: &[Check] = &[Check::ResolvesEffect(aux::stringid(ID, 0))];

struct Procedure;

impl XyzProcedure for Procedure {
    const LV: Option<u32> = Some(8);
    const CT: usize = 2;
}

pub fn initial_effect(duel: &mut Duel, c: CardId) {
    Xyz::add_procedure::<Procedure>(duel, c);
    c.enable_revive_limit(duel);
    let e1 = duel.create_effect(c);
    e1.set_type(duel, EFFECT_TYPE_SINGLE);
    e1.set_code(duel, EFFECT_CANNOT_BE_EFFECT_TARGET);
    e1.set_property(duel, EFFECT_FLAG_SINGLE_RANGE);
    e1.set_range(duel, LOCATION_MZONE);
    e1.set_value(duel, Value::EffectPlayer(aux::tgoval));
    c.register_effect(duel, e1);
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 0));
    e2.set_category(duel, CATEGORY_SPECIAL_SUMMON);
    e2.set_type(duel, EFFECT_TYPE_IGNITION);
    e2.set_range(duel, LOCATION_MZONE);
    e2.set_count_limit(duel, 1, ID);
    e2.set_cost(duel, Cost::detach_from_self::<1, 1>);
    e2.set_target(duel, sptg);
    e2.set_operation(duel, spop);
    c.register_effect(duel, e2);
}

fn spfilter(c: CardId, duel: &Duel) -> bool {
    c.is_race(duel, RACE_DRAGON) && c.is_set_card(duel, &[SET_NUMBER]) && !c.is_code(duel, ID)
}

fn spchk(c: CardId, duel: &Duel, e: EffectId, tp: u8) -> bool {
    if !c.is_can_be_special_summoned(duel, e, 0, tp, false, false) {
        return false;
    }
    if c.is_location(duel, LOCATION_EXTRA) {
        return duel.get_location_count_from_ex(tp, tp, None, c) > 0;
    }
    duel.get_location_count(tp, LOCATION_MZONE) > 0
}

fn sptg(duel: &mut Duel, a: &Args, chk: i32, _: Option<CardId>) -> Script<bool> {
    let (e, tp) = (a.e, a.tp);
    let g = duel.get_matching_group(spfilter, tp, LOCATION_EXTRA | LOCATION_GRAVE, 0, None);
    if chk == 0 {
        return Ok(aux::select_unselect_group_check(duel, g, 2, 2, &|duel, sg, _, _| {
            (rescon(duel, sg, e, tp), false)
        }));
    }
    duel.set_operation_info(0, CATEGORY_SPECIAL_SUMMON, None, 1, tp, (LOCATION_EXTRA | LOCATION_GRAVE) as i32);
    Ok(false)
}

fn rescon(duel: &Duel, sg: CardSet, e: EffectId, tp: u8) -> bool {
    sg.is_exists(duel, |c, duel| spchk(c, duel, e, tp), 1, None)
        && sg.get_class_count(duel, |c, duel| i64::from(c.get_code(duel))) == sg.len()
}

fn spop(duel: &mut Duel, a: &Args) -> Script<()> {
    let (e, tp) = (a.e, a.tp);
    let c = e.get_handler(duel);
    let g = duel.get_matching_group(spfilter, tp, LOCATION_EXTRA | LOCATION_GRAVE, 0, None);
    let check = |duel: &Duel, sg, _, _| (rescon(duel, sg, e, tp), false);
    if aux::select_unselect_group_check(duel, g, 2, 2, &check) {
        let sg = aux::select_unselect_group(duel, g, 2, 2, &check, tp, HINTMSG_SELECT)?;
        duel.hint(HINT_SELECTMSG, tp, HINTMSG_SPSUMMON);
        let tg = sg
            .filter(duel, |sc, duel| spchk(sc, duel, e, tp), None)
            .select(duel, tp, 1, 1, None)?;
        if duel.special_summon(tg, 0, tp, tp, false, false, POS_FACEUP)? != 0
            && let Some(tc) = tg.get_first()
        {
            duel.overlay(tc, sg.difference(tg))?;
            let e1 = duel.create_effect(c);
            e1.set_type(duel, EFFECT_TYPE_FIELD);
            e1.set_code(duel, EFFECT_CANNOT_ATTACK);
            e1.set_target_range(duel, LOCATION_MZONE, 0);
            e1.set_label_object_group(duel, Some(tc.into()));
            e1.set_target_card(duel, atktg);
            e1.set_reset(duel, RESET_PHASE | PHASE_END);
            duel.register_effect(e1, tp);
        }
    }
    let e2 = duel.create_effect(c);
    e2.set_description(duel, aux::stringid(ID, 1));
    e2.set_type(duel, EFFECT_TYPE_FIELD);
    e2.set_code(duel, EFFECT_CANNOT_SPECIAL_SUMMON);
    e2.set_property(duel, EFFECT_FLAG_PLAYER_TARGET + EFFECT_FLAG_CLIENT_HINT);
    e2.set_target_range(duel, 1, 0);
    e2.set_reset(duel, RESET_PHASE | PHASE_END);
    duel.register_effect(e2, tp);
    Ok(())
}

fn atktg(duel: &Duel, e: EffectId, c: CardId) -> bool {
    e.get_label_object_group(duel).and_then(|g| g.get_first()) != Some(c)
}
